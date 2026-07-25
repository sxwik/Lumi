use egui::{Color32, FontId, RichText, Ui, Vec2};
use lumi_parser::{ElementType, LumiNode};

#[derive(Default)]
pub struct RenderOptions {
    pub pending_navigation: Option<String>,
}

pub fn render_page(ui: &mut Ui, root: &LumiNode, options: &mut RenderOptions) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(0.0, 12.0);
        let max_width = 880.0;
        let margin = ((ui.available_width() - max_width) / 2.0).max(16.0);

        ui.horizontal(|ui| {
            ui.add_space(margin);
            ui.vertical(|ui| {
                ui.set_max_width(max_width);
                render_node(ui, root, options);
            });
            ui.add_space(margin);
        });
    });
}

pub fn render_markdown(ui: &mut Ui, markdown: &str, _options: &mut RenderOptions) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(0.0, 12.0);
        let max_width = 880.0;
        let margin = ((ui.available_width() - max_width) / 2.0).max(16.0);

        ui.horizontal(|ui| {
            ui.add_space(margin);
            ui.vertical(|ui| {
                ui.set_max_width(max_width);
                let mut cache = egui_commonmark::CommonMarkCache::default();
                egui_commonmark::CommonMarkViewer::new("lumi_md_viewer")
                    .show(ui, &mut cache, markdown);
            });
            ui.add_space(margin);
        });
    });
}

fn render_node(ui: &mut Ui, node: &LumiNode, options: &mut RenderOptions) {
    match &node.element_type {
        ElementType::Page => {
            for child in &node.children {
                render_node(ui, child, options);
            }
        }
        ElementType::Title => {
            let text = extract_text(node);
            ui.heading(
                RichText::new(text)
                    .size(28.0)
                    .strong()
                    .color(Color32::from_rgb(90, 160, 255)),
            );
            ui.add_space(8.0);
        }
        ElementType::Heading => {
            let text = extract_text(node);
            let size = node
                .get_attr("size")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(22.0);
            ui.label(
                RichText::new(text)
                    .size(size)
                    .strong()
                    .color(Color32::from_rgb(220, 225, 235)),
            );
        }
        ElementType::Paragraph => {
            let text = extract_text(node);
            ui.label(
                RichText::new(text)
                    .size(15.0)
                    .color(Color32::from_rgb(160, 175, 195)),
            );
            ui.add_space(4.0);
        }
        ElementType::Text => {
            let text = node.value.as_deref().unwrap_or("");
            ui.label(RichText::new(text).size(15.0));
        }
        ElementType::Button => {
            let text = extract_text(node);
            let goto_url = extract_goto(node);

            let btn = egui::Button::new(
                RichText::new(if text.is_empty() { "Click" } else { &text })
                    .size(14.0)
                    .color(Color32::from_rgb(100, 180, 255)),
            )
            .fill(Color32::from_rgb(32, 42, 58))
            .rounding(6.0);

            if ui.add(btn).clicked() {
                if let Some(target) = goto_url {
                    options.pending_navigation = Some(target);
                }
            }
            ui.add_space(4.0);
        }
        ElementType::List => {
            ui.add_space(4.0);
            for child in &node.children {
                render_node(ui, child, options);
            }
            ui.add_space(4.0);
        }
        ElementType::Item => {
            let text = extract_text(node);
            ui.horizontal(|ui| {
                ui.label(RichText::new("✓").color(Color32::from_rgb(80, 200, 140)));
                ui.add_space(6.0);
                ui.label(
                    RichText::new(text)
                        .size(14.0)
                        .color(Color32::from_rgb(200, 210, 225)),
                );
            });
        }
        ElementType::Container => {
            ui.add_space(6.0);
            egui::Frame::none()
                .fill(Color32::from_rgb(24, 30, 42))
                .rounding(8.0)
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(38, 48, 68)))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    for child in &node.children {
                        render_node(ui, child, options);
                    }
                });
            ui.add_space(6.0);
        }
        ElementType::Row => {
            ui.horizontal(|ui| {
                for child in &node.children {
                    render_node(ui, child, options);
                }
            });
        }
        ElementType::Column => {
            ui.vertical(|ui| {
                for child in &node.children {
                    render_node(ui, child, options);
                }
            });
        }
        ElementType::Divider => {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
        }
        ElementType::Badge => {
            let text = extract_text(node);
            egui::Frame::none()
                .fill(Color32::from_rgb(40, 60, 90))
                .rounding(4.0)
                .inner_margin(Vec2::new(6.0, 3.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(text)
                            .size(12.0)
                            .color(Color32::from_rgb(120, 200, 255)),
                    );
                });
        }
        ElementType::CodeBlock => {
            let text = extract_text(node);
            egui::Frame::none()
                .fill(Color32::from_rgb(18, 20, 28))
                .rounding(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(text)
                            .font(FontId::monospace(13.0))
                            .color(Color32::from_rgb(150, 220, 160)),
                    );
                });
        }
        _ => {
            for child in &node.children {
                render_node(ui, child, options);
            }
        }
    }
}

fn extract_text(node: &LumiNode) -> String {
    if let Some(ref val) = node.value {
        if !val.is_empty() {
            return val.clone();
        }
    }
    for child in &node.children {
        let text = extract_text(child);
        if !text.is_empty() {
            return text;
        }
    }
    String::new()
}

fn extract_goto(node: &LumiNode) -> Option<String> {
    if let Some(url) = node.get_attr("goto") {
        return Some(url.to_string());
    }
    for child in &node.children {
        if let Some(url) = extract_goto(child) {
            return Some(url);
        }
    }
    None
}
