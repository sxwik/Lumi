use lumi_protocol::{LmpMessage, LumiPackage, LumiUri};
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    let bind_addr = "127.0.0.1:9001";

    if args.len() >= 4 && args[1] == "serve" {
        let pkg_path = &args[2];
        let domain = &args[4];
        println!("[lumid] Serving custom package '{}' on domain 'lumi://{}'...", pkg_path, domain);
        
        let pkg_bytes = fs::read(pkg_path).expect("Failed to read package file");
        run_server(bind_addr, Some((domain.clone(), pkg_bytes)));
    } else {
        println!("[lumid] Starting Lumi Default Public Network Server daemon on {}", bind_addr);
        run_server(bind_addr, None);
    }
}

fn run_server(bind_addr: &str, custom_site: Option<(String, Vec<u8>)>) {
    let listener = match TcpListener::bind(bind_addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[lumid] Failed to bind to {}: {}", bind_addr, e);
            return;
        }
    };

    println!("[lumid] Network server active! Ready for persistent LMP connections...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let site_copy = custom_site.clone();
                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, site_copy) {
                        eprintln!("[lumid] Connection session ended: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("[lumid] Incoming connection error: {}", e),
        }
    }
}

fn handle_connection(
    mut stream: TcpStream,
    custom_site: Option<(String, Vec<u8>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let msg = match LmpMessage::read_from(&mut stream) {
            Ok(m) => m,
            Err(lumi_protocol::LmpError::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Box::new(e)),
        };

        let uri = LumiUri::parse(&msg.header.uri).unwrap_or(LumiUri {
            host: "welcome.lumi".to_string(),
            port: 7878,
            path: "/".to_string(),
        });

        println!("[lumid] [Stream #{}] Request URI: 'lumi://{}{}'", msg.stream_id, uri.host, uri.path);

        let (content_type, payload_bytes) = if let Some((ref custom_domain, ref custom_bytes)) = custom_site {
            if &uri.host == custom_domain {
                ("application/lpkg", custom_bytes.clone())
            } else {
                route_default_pages(&uri.host, &uri.path)
            }
        } else {
            route_default_pages(&uri.host, &uri.path)
        };

        let response = LmpMessage::new_response(msg.stream_id, content_type, payload_bytes);
        response.write_to(&mut stream)?;
    }

    Ok(())
}

fn route_default_pages(host: &str, path: &str) -> (&'static str, Vec<u8>) {
    match (host, path) {
        ("welcome.lumi", _) | ("welcome.home", _) => {
            let pkg = LumiPackage::new("Welcome Portal", WELCOME_PAGE);
            ("application/lpkg", pkg.to_bytes().unwrap())
        }
        ("search.lumi", _) | ("search.home", _) => {
            let pkg = LumiPackage::new("Lumi Search", SEARCH_PAGE);
            ("application/lpkg", pkg.to_bytes().unwrap())
        }
        ("docs.lumi", _) | ("docs.home", _) => {
            let pkg = LumiPackage::new("Docs", DOCS_PAGE);
            ("application/lpkg", pkg.to_bytes().unwrap())
        }
        ("chat.lumi", _) | ("chat.home", _) => {
            let pkg = LumiPackage::new("Encrypted Chat", CHAT_PAGE);
            ("application/lpkg", pkg.to_bytes().unwrap())
        }
        ("gallery.lumi", _) | ("gallery.home", _) => {
            let pkg = LumiPackage::new("Gallery", GALLERY_PAGE);
            ("application/lpkg", pkg.to_bytes().unwrap())
        }
        ("games.lumi", _) => {
            let pkg = LumiPackage::new("Lumi Arcade", GAMES_PAGE);
            ("application/lpkg", pkg.to_bytes().unwrap())
        }
        ("wiki.lumi", _) => {
            let pkg = LumiPackage::new("Lumi Wiki", WIKI_PAGE);
            ("application/lpkg", pkg.to_bytes().unwrap())
        }
        _ => {
            let pkg = LumiPackage::new("Welcome Portal", WELCOME_PAGE);
            ("application/lpkg", pkg.to_bytes().unwrap())
        }
    }
}

const WELCOME_PAGE: &str = r#"
page {
    title "Welcome to Lumi Network"

    badge {
        text "Lumi v0.3 Public Network Prototype"
    }

    heading {
        text "The Open Privacy Web"
    }

    paragraph {
        text "No Chromium. No Electron. No HTML/JS/CSS. Just pure Rust binary LMP protocol and LumiML."
    }

    divider {}

    container {
        heading {
            text "Registered Network Sites"
        }
        list {
            item { text "lumi://welcome.lumi - Ecosystem Entrance" }
            item { text "lumi://search.lumi - Decentralized Search Index" }
            item { text "lumi://docs.lumi - Protocol & RFC Documentation" }
            item { text "lumi://games.lumi - Native Arcade Showcase" }
            item { text "lumi://wiki.lumi - Community Knowledgebase" }
        }
    }

    divider {}

    row {
        button {
            text "Open Lumi Search"
            goto "lumi://search.lumi"
        }
        button {
            text "Read Protocol Specs"
            goto "lumi://docs.lumi"
        }
    }
}
"#;

const SEARCH_PAGE: &str = r#"
page {
    title "Lumi Search"

    badge {
        text "Decentralized Site Index"
    }

    heading {
        text "Search the Lumi Network"
    }

    paragraph {
        text "Privacy-first search indexing registered .lumi domains."
    }

    divider {}

    container {
        heading {
            text "Featured Domains"
        }
        list {
            item { text "[docs.lumi] - Official LMP Core Protocol RFC Specifications" }
            item { text "[welcome.lumi] - Main Portal & Gateway" }
            item { text "[games.lumi] - Experimental Native Web Games" }
            item { text "[wiki.lumi] - Open LumiML Architecture Knowledge Base" }
            item { text "[chat.lumi] - Encrypted Peer Node Communication" }
        }
    }

    divider {}

    row {
        button {
            text "Explore Games"
            goto "lumi://games.lumi"
        }
        button {
            text "Visit Wiki"
            goto "lumi://wiki.lumi"
        }
    }
}
"#;

const DOCS_PAGE: &str = r#"
page {
    title "Lumi RFC Documentation"

    badge {
        text "RFC Specifications 0001 - 0005"
    }

    heading {
        text "Official Standards"
    }

    paragraph {
        text "Lumi features formal Request for Comments (RFCs) governing protocol development."
    }

    container {
        heading {
            text "Published RFCs"
        }
        list {
            item { text "RFC-0001: LMP Core Binary Protocol Specification" }
            item { text "RFC-0002: LumiML Markup Specification & Grammar" }
            item { text "RFC-0003: Lumi Name Service (LNS) Standard" }
            item { text "RFC-0004: LPKG Package Envelope Format" }
            item { text "RFC-0005: Lumi Browser Extension (.lpx) API" }
        }
    }

    divider {}

    button {
        text "← Back to Search"
        goto "lumi://search.lumi"
    }
}
"#;

const CHAT_PAGE: &str = r#"
page {
    title "Encrypted Lumi Chat"
    paragraph { text "Secure node communication over persistent LMP channels." }
    button { text "Return Home" goto "lumi://welcome.lumi" }
}
"#;

const GALLERY_PAGE: &str = r#"
page {
    title "Lumi Gallery Showcase"
    paragraph { text "Custom wgpu/egui pipeline rendering native LumiML." }
    button { text "Return Home" goto "lumi://welcome.lumi" }
}
"#;

const GAMES_PAGE: &str = r#"
page {
    title "Lumi Arcade & Games"

    badge {
        text "Native GPU Speed"
    }

    heading {
        text "Low-Latency Gaming on Lumi"
    }

    paragraph {
        text "Experience native render speed with zero web assembly or JS engine bottlenecks."
    }

    button {
        text "Return to Search"
        goto "lumi://search.lumi"
    }
}
"#;

const WIKI_PAGE: &str = r#"
page {
    title "Lumi Open Wiki"

    badge {
        text "Community Knowledgebase"
    }

    heading {
        text "Lumi Ecosystem Architecture"
    }

    paragraph {
        text "Learn how to build custom servers, parsers, and custom Lumi browsers from the open RFC standards."
    }

    button {
        text "Read RFC Specs"
        goto "lumi://docs.lumi"
    }
}
"#;
