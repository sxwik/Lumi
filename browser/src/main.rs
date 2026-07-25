mod extension;

use eframe::egui;
use egui::{Color32, RichText, Vec2};
use extension::ExtensionRegistry;
use lumi_parser::LumiNode;
use lumi_protocol::tls::{self, LmpStream};
use lumi_protocol::{LmpMessage, LnsResolver, LumiPackage, LumiUri};
use lumi_renderer::RenderOptions;
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

enum PageContent {
    LumiMl(LumiNode),
    Markdown(String),
}

struct Tab {
    title: String,
    url: String,
    content: Option<PageContent>,
    raw_payload: String,
    error: Option<String>,
    history_back: Vec<String>,
    history_forward: Vec<String>,
}

impl Tab {
    fn new(url: &str) -> Self {
        Self {
            title: "New Tab".to_string(),
            url: url.to_string(),
            content: None,
            raw_payload: String::new(),
            error: None,
            history_back: Vec::new(),
            history_forward: Vec::new(),
        }
    }
}

enum NetworkResponse {
    Success {
        tab_index: usize,
        url: String,
        payload: Vec<u8>,
        page_content: Result<PageContent, String>,
    },
    Error {
        tab_index: usize,
        url: String,
        error: String,
    },
}

struct LogEntry {
    time: String,
    msg: String,
}

pub struct LumiApp {
    tabs: Vec<Tab>,
    active_tab: usize,
    url_input: String,
    show_dev_console: bool,
    show_settings: bool,
    show_bookmarks: bool,
    show_history: bool,
    show_extensions: bool,
    bookmarks: Vec<String>,
    history: Vec<String>,
    net_sender: Sender<(usize, String)>,
    net_receiver: Receiver<NetworkResponse>,
    logs: VecDeque<LogEntry>,
    extensions: ExtensionRegistry,
}

impl LumiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = Color32::from_rgb(18, 20, 26);
        visuals.panel_fill = Color32::from_rgb(24, 27, 36);
        cc.egui_ctx.set_visuals(visuals);

        let (req_tx, req_rx) = channel::<(usize, String)>();
        let (res_tx, res_rx) = channel::<NetworkResponse>();

        thread::spawn(move || {
            let lns = LnsResolver::new();
            let mut current_stream: Option<(String, LmpStream)> = None;
            let tls_config = match tls::make_dev_client_config() {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("[LMP TLS] Failed to initialize TLS client config: {}", e);
                    return;
                }
            };

            while let Ok((tab_index, url_str)) = req_rx.recv() {
                match url_str.parse::<LumiUri>() {
                    Ok(uri) => {
                        println!("[LNS] Resolving domain '{}'...", uri.host);
                        let resolved_addr = match lns.resolve(&uri.host) {
                            Ok(addr) => {
                                println!("[LNS] Resolved '{}' -> {}", uri.host, addr);
                                addr
                            }
                            Err(e) => {
                                eprintln!("[LNS] Resolution failed for '{}': {}", uri.host, e);
                                let _ = res_tx.send(NetworkResponse::Error {
                                    tab_index,
                                    url: url_str.clone(),
                                    error: format!("LNS Resolution Error: {}", e),
                                });
                                continue;
                            }
                        };

                        let need_new = match current_stream {
                            Some((ref addr, _)) => addr != &resolved_addr,
                            None => true,
                        };

                        if need_new {
                            println!(
                                "[LMP TLS] Establishing TLS connection to {}...",
                                resolved_addr
                            );
                            match tls::connect_tls(&resolved_addr, &uri.host, tls_config.clone()) {
                                Ok(stream) => {
                                    println!(
                                        "[LMP TLS] TLS Handshake successful with {}",
                                        resolved_addr
                                    );
                                    current_stream = Some((resolved_addr.clone(), stream));
                                }
                                Err(e) => {
                                    eprintln!("[LMP TLS] TLS Connection failed: {}", e);
                                    let _ = res_tx.send(NetworkResponse::Error {
                                        tab_index,
                                        url: url_str.clone(),
                                        error: format!(
                                            "TLS Connection failed to target '{}': {}",
                                            uri.host, e
                                        ),
                                    });
                                    continue;
                                }
                            }
                        }

                        if let Some((_, ref mut stream)) = current_stream {
                            println!("[LMP] Sending GET request for '{}'...", url_str);
                            let req = LmpMessage::new_request(&url_str, 1);
                            if let Err(e) = req.write_to(stream) {
                                current_stream = None;
                                let _ = res_tx.send(NetworkResponse::Error {
                                    tab_index,
                                    url: url_str,
                                    error: format!("Write error: {}", e),
                                });
                                continue;
                            }

                            println!("[LMP] Receiving LMP frame response...");
                            match LmpMessage::read_from(stream) {
                                Ok(res) => {
                                    println!("[LMP] Frame received! Payload size: {} bytes, Content-Type: {}", res.payload.len(), res.header.content_type);
                                    let raw_bytes = res.payload.clone();

                                    let page_content: Result<PageContent, String> = if res
                                        .header
                                        .content_type
                                        == "text/markdown"
                                    {
                                        let md_src =
                                            String::from_utf8_lossy(&res.payload).to_string();
                                        Ok(PageContent::Markdown(md_src))
                                    } else if res.header.content_type == "application/lpkg" {
                                        match LumiPackage::from_bytes(&res.payload) {
                                            Ok(pkg) => {
                                                if let Some(ref md_src) = pkg.index_md {
                                                    Ok(PageContent::Markdown(md_src.clone()))
                                                } else if let Some(ref lml_src) = pkg.index_lml {
                                                    lumi_parser::parse(lml_src)
                                                        .map(PageContent::LumiMl)
                                                        .map_err(|e| {
                                                            format!("LumiML Parse Error: {}", e)
                                                        })
                                                } else {
                                                    Err("Package does not contain index.md or index.lml".to_string())
                                                }
                                            }
                                            Err(e) => Err(format!("Error unpacking .lpkg: {}", e)),
                                        }
                                    } else {
                                        let lml_code =
                                            String::from_utf8_lossy(&res.payload).to_string();
                                        lumi_parser::parse(&lml_code)
                                            .map(PageContent::LumiMl)
                                            .map_err(|e| format!("LumiML Parse Error: {}", e))
                                    };

                                    let _ = res_tx.send(NetworkResponse::Success {
                                        tab_index,
                                        url: url_str,
                                        payload: raw_bytes,
                                        page_content,
                                    });
                                }
                                Err(e) => {
                                    current_stream = None;
                                    let _ = res_tx.send(NetworkResponse::Error {
                                        tab_index,
                                        url: url_str,
                                        error: format!("LMP Read Error: {}", e),
                                    });
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = res_tx.send(NetworkResponse::Error {
                            tab_index,
                            url: url_str,
                            error: format!("Invalid URI format: {}", e),
                        });
                    }
                }
            }
        });

        let mut app = Self {
            tabs: vec![Tab::new("lumi://welcome.lumi")],
            active_tab: 0,
            url_input: "lumi://welcome.lumi".to_string(),
            show_dev_console: false,
            show_settings: false,
            show_bookmarks: false,
            show_history: false,
            show_extensions: false,
            bookmarks: vec![
                "lumi://welcome.lumi".to_string(),
                "lumi://search.lumi".to_string(),
                "lumi://docs.lumi".to_string(),
                "lumi://games.lumi".to_string(),
                "lumi://wiki.lumi".to_string(),
            ],
            history: Vec::new(),
            net_sender: req_tx,
            net_receiver: res_rx,
            logs: VecDeque::new(),
            extensions: ExtensionRegistry::new(),
        };

        app.log("Lumi Browser v0.3 active (First Public Network + Search + RFC System)");
        app.navigate_current("lumi://welcome.lumi");
        app
    }

    fn log(&mut self, msg: &str) {
        if self.logs.len() > 100 {
            self.logs.pop_front();
        }
        self.logs.push_back(LogEntry {
            time: "14:02".to_string(),
            msg: msg.to_string(),
        });
    }

    fn navigate_current(&mut self, url: &str) {
        if self.tabs.is_empty() {
            return;
        }

        let tab = &mut self.tabs[self.active_tab];
        if !tab.url.is_empty() && tab.url != url {
            tab.history_back.push(tab.url.clone());
            tab.history_forward.clear();
        }
        tab.url = url.to_string();
        self.url_input = url.to_string();

        if !self.history.contains(&url.to_string()) {
            self.history.push(url.to_string());
        }

        self.log(&format!("Navigating to {}", url));
        let _ = self.net_sender.send((self.active_tab, url.to_string()));
    }

    fn process_network_responses(&mut self) {
        while let Ok(res) = self.net_receiver.try_recv() {
            match res {
                NetworkResponse::Success {
                    tab_index,
                    url,
                    payload,
                    page_content,
                } => {
                    if tab_index < self.tabs.len() {
                        let tab = &mut self.tabs[tab_index];
                        tab.raw_payload = format!(
                            "[LMP Payload: {} bytes]\n\n{}",
                            payload.len(),
                            String::from_utf8_lossy(&payload)
                        );

                        match page_content {
                            Ok(content) => {
                                match &content {
                                    PageContent::LumiMl(ast) => {
                                        if let Some(title_node) = ast.children.iter().find(|c| {
                                            c.element_type == lumi_parser::ElementType::Title
                                        }) {
                                            if let Some(val) =
                                                title_node.value.as_ref().or_else(|| {
                                                    title_node
                                                        .children
                                                        .first()
                                                        .and_then(|c| c.value.as_ref())
                                                })
                                            {
                                                tab.title = val.clone();
                                            }
                                        } else {
                                            tab.title = url.clone();
                                        }
                                    }
                                    PageContent::Markdown(md_text) => {
                                        let first_line =
                                            md_text.lines().find(|l| l.starts_with("# "));
                                        if let Some(title) = first_line {
                                            tab.title = title.trim_start_matches("# ").to_string();
                                        } else {
                                            tab.title = url.clone();
                                        }
                                    }
                                }
                                tab.content = Some(content);
                                tab.error = None;
                            }
                            Err(e) => {
                                tab.error = Some(e);
                                tab.content = None;
                            }
                        }
                    }
                    self.log(&format!("Loaded {}", url));
                }
                NetworkResponse::Error {
                    tab_index,
                    url,
                    error,
                } => {
                    if tab_index < self.tabs.len() {
                        let tab = &mut self.tabs[tab_index];
                        tab.error = Some(error.clone());
                        tab.content = None;
                    }
                    self.log(&format!("Failed to load {}: {}", url, error));
                }
            }
        }
    }
}

impl eframe::App for LumiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_network_responses();

        let mut next_nav: Option<String> = None;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);

            // Tab Bar
            ui.horizontal(|ui| {
                let mut close_tab_idx = None;

                for (idx, tab) in self.tabs.iter().enumerate() {
                    let is_active = idx == self.active_tab;
                    let title_text = format!(
                        " {} ",
                        if tab.title.is_empty() {
                            &tab.url
                        } else {
                            &tab.title
                        }
                    );

                    let btn = egui::Button::new(RichText::new(&title_text).size(13.0).color(
                        if is_active {
                            Color32::WHITE
                        } else {
                            Color32::LIGHT_GRAY
                        },
                    ))
                    .fill(if is_active {
                        Color32::from_rgb(45, 55, 75)
                    } else {
                        Color32::from_rgb(25, 28, 38)
                    })
                    .rounding(4.0);

                    if ui.add(btn).clicked() {
                        self.active_tab = idx;
                        self.url_input = self.tabs[idx].url.clone();
                    }

                    if self.tabs.len() > 1 && ui.small_button("x").clicked() {
                        close_tab_idx = Some(idx);
                    }
                }

                if let Some(idx) = close_tab_idx {
                    self.tabs.remove(idx);
                    if self.active_tab >= self.tabs.len() {
                        self.active_tab = self.tabs.len().saturating_sub(1);
                    }
                    if !self.tabs.is_empty() {
                        self.url_input = self.tabs[self.active_tab].url.clone();
                    }
                }

                if ui.button("+").clicked() {
                    self.tabs.push(Tab::new("lumi://welcome.lumi"));
                    self.active_tab = self.tabs.len() - 1;
                    self.navigate_current("lumi://welcome.lumi");
                }
            });

            ui.add_space(4.0);

            // Navigation Bar
            ui.horizontal(|ui| {
                let can_back = !self.tabs[self.active_tab].history_back.is_empty();
                if ui.add_enabled(can_back, egui::Button::new("⯇")).clicked() {
                    let tab = &mut self.tabs[self.active_tab];
                    if let Some(prev) = tab.history_back.pop() {
                        tab.history_forward.push(tab.url.clone());
                        tab.url = prev.clone();
                        self.url_input = prev.clone();
                        let _ = self.net_sender.send((self.active_tab, prev));
                    }
                }

                let can_fwd = !self.tabs[self.active_tab].history_forward.is_empty();
                if ui.add_enabled(can_fwd, egui::Button::new("⯈")).clicked() {
                    let tab = &mut self.tabs[self.active_tab];
                    if let Some(next) = tab.history_forward.pop() {
                        tab.history_back.push(tab.url.clone());
                        tab.url = next.clone();
                        self.url_input = next.clone();
                        let _ = self.net_sender.send((self.active_tab, next));
                    }
                }

                if ui.button("⟳").clicked() {
                    let url = self.tabs[self.active_tab].url.clone();
                    self.navigate_current(&url);
                }

                let response = ui.add_sized(
                    Vec2::new(ui.available_width() - 260.0, 24.0),
                    egui::TextEdit::singleline(&mut self.url_input)
                        .hint_text("Enter Lumi URI (e.g. lumi://welcome.lumi)...")
                        .font(egui::TextStyle::Monospace),
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let mut url = self.url_input.clone();
                    if !url.starts_with("lumi://") {
                        url = format!("lumi://{}", url);
                    }
                    self.navigate_current(&url);
                }

                if ui
                    .selectable_label(self.show_bookmarks, "★ Bookmarks")
                    .clicked()
                {
                    self.show_bookmarks = !self.show_bookmarks;
                }

                if ui
                    .selectable_label(self.show_history, "📜 History")
                    .clicked()
                {
                    self.show_history = !self.show_history;
                }

                if ui
                    .selectable_label(self.show_extensions, "🧩 LPX")
                    .clicked()
                {
                    self.show_extensions = !self.show_extensions;
                }

                if ui
                    .selectable_label(self.show_dev_console, "🛠 Dev")
                    .clicked()
                {
                    self.show_dev_console = !self.show_dev_console;
                }

                if ui.selectable_label(self.show_settings, "⚙").clicked() {
                    self.show_settings = !self.show_settings;
                }
            });

            ui.add_space(4.0);
        });

        if self.show_dev_console {
            egui::SidePanel::right("dev_console_panel")
                .default_width(340.0)
                .show(ctx, |ui| {
                    ui.heading("Developer Console v0.3");
                    ui.separator();

                    ui.label(RichText::new("LMP Packet Inspector").strong());
                    let payload_str = &self.tabs[self.active_tab].raw_payload;
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.code(payload_str);
                        });

                    ui.separator();
                    ui.label(RichText::new("Network Logs").strong());
                    egui::ScrollArea::vertical()
                        .max_height(250.0)
                        .show(ui, |ui| {
                            for log in &self.logs {
                                ui.label(
                                    RichText::new(format!("[{}] {}", log.time, log.msg))
                                        .size(11.0)
                                        .color(Color32::GRAY),
                                );
                            }
                        });
                });
        }

        if self.show_extensions {
            egui::Window::new("Lumi Extensions (.lpx)")
                .open(&mut self.show_extensions)
                .show(ctx, |ui| {
                    ui.heading("Installed Extensions");
                    ui.separator();
                    for ext in &self.extensions.extensions {
                        ui.label(
                            RichText::new(&ext.name)
                                .strong()
                                .color(Color32::from_rgb(100, 180, 255)),
                        );
                        ui.label(format!("Version: {}", ext.version));
                        ui.label(&ext.description);
                        ui.separator();
                    }
                });
        }

        if self.show_settings {
            egui::Window::new("Lumi Settings")
                .open(&mut self.show_settings)
                .show(ctx, |ui| {
                    ui.heading("Lumi Ecosystem Standards");
                    ui.checkbox(&mut true, "Strict LNS Standard Active");
                    ui.checkbox(&mut true, "Zero Telemetry / Local Storage Only");
                    ui.checkbox(&mut true, ".lpkg Decompressor Active");
                    ui.checkbox(&mut true, ".lpx Extensions Engine");
                    ui.separator();
                    ui.label("LMP Core RFC-0001 Compliant");
                });
        }

        if self.show_bookmarks {
            let mut show = self.show_bookmarks;
            let bookmarks_to_show = self.bookmarks.clone();
            egui::Window::new("Bookmarks")
                .open(&mut show)
                .show(ctx, |ui| {
                    for bm in bookmarks_to_show {
                        if ui.button(&bm).clicked() {
                            next_nav = Some(bm);
                        }
                    }
                });
            self.show_bookmarks = show;
        }

        if self.show_history {
            let mut show = self.show_history;
            let history_to_show = self.history.clone();
            egui::Window::new("Browsing History")
                .open(&mut show)
                .show(ctx, |ui| {
                    if history_to_show.is_empty() {
                        ui.label("No history recorded yet.");
                    } else {
                        for item in history_to_show.iter().rev() {
                            if ui.button(item).clicked() {
                                next_nav = Some(item.clone());
                            }
                        }
                    }
                });
            self.show_history = show;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let tab = &self.tabs[self.active_tab];

            if let Some(ref err) = tab.error {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.heading(
                        RichText::new("Unable to load page")
                            .color(Color32::from_rgb(255, 90, 90))
                            .size(24.0),
                    );
                    ui.add_space(10.0);
                    ui.label(RichText::new(err).size(15.0).color(Color32::LIGHT_RED));
                    ui.add_space(20.0);
                    ui.label("Run 'cargo run -p lumid' to launch the Lumi network server daemon.");
                });
            } else if let Some(ref content) = tab.content {
                let mut options = RenderOptions::default();
                match content {
                    PageContent::LumiMl(ast) => {
                        lumi_renderer::render_page(ui, ast, &mut options);
                    }
                    PageContent::Markdown(md_text) => {
                        lumi_renderer::render_markdown(ui, md_text, &mut options);
                    }
                }
                if let Some(target_url) = options.pending_navigation {
                    next_nav = Some(target_url);
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.spinner();
                    ui.label("Loading page over LMP network channel...");
                });
            }
        });

        if let Some(target) = next_nav {
            self.navigate_current(&target);
        }

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lumi Browser v0.3 - First Public Network Edition")
            .with_inner_size([1100.0, 750.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Lumi Browser",
        native_options,
        Box::new(|cc| Box::new(LumiApp::new(cc))),
    )
}
