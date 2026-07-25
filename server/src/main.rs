use lumi_protocol::tls::{self, LmpStream};
use lumi_protocol::{LmpMessage, LumiPackage, LumiUri};
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    let bind_addr = "127.0.0.1:9001";

    let mut cert_path = PathBuf::from("certs/dev_cert.pem");
    let mut key_path = PathBuf::from("certs/dev_key.pem");

    let mut i = 1;
    let mut custom_site = None;

    while i < args.len() {
        match args[i].as_str() {
            "--cert" if i + 1 < args.len() => {
                cert_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--key" if i + 1 < args.len() => {
                key_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "serve" if i + 3 < args.len() => {
                let pkg_path = &args[i + 1];
                let domain = &args[i + 3];
                println!(
                    "[lumid] Serving custom package '{}' on domain 'lumi://{}'...",
                    pkg_path, domain
                );
                let pkg_bytes = fs::read(pkg_path).expect("Failed to read package file");
                custom_site = Some((domain.clone(), pkg_bytes));
                i += 4;
            }
            _ => {
                i += 1;
            }
        }
    }

    println!(
        "[lumid] Starting Lumi Default Public Network TLS Server daemon on {}",
        bind_addr
    );

    let (certs, key) = match tls::load_or_generate_server_certs(&cert_path, &key_path) {
        Ok(ck) => ck,
        Err(e) => {
            eprintln!("[lumid] TLS Certificate setup failed: {}", e);
            return;
        }
    };

    let tls_config = match tls::make_server_config(certs, key) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("[lumid] TLS Server configuration failed: {}", e);
            return;
        }
    };

    run_server(bind_addr, custom_site, tls_config);
}

fn run_server(
    bind_addr: &str,
    custom_site: Option<(String, Vec<u8>)>,
    tls_config: Arc<tls::ServerConfig>,
) {
    let listener = match TcpListener::bind(bind_addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[lumid] Failed to bind to {}: {}", bind_addr, e);
            return;
        }
    };

    println!(
        "[lumid] Network server active! Ready for persistent TLS-encrypted LMP connections..."
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let site_copy = custom_site.clone();
                let config_copy = tls_config.clone();
                thread::spawn(move || {
                    let tls_stream = match tls::accept_tls(stream, config_copy) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("[lumid] TLS Handshake failed: {}", e);
                            return;
                        }
                    };

                    if let Err(e) = handle_connection(tls_stream, site_copy) {
                        eprintln!("[lumid] Connection session ended: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("[lumid] Incoming connection error: {}", e),
        }
    }
}

fn handle_connection(
    mut stream: LmpStream,
    custom_site: Option<(String, Vec<u8>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let msg = match LmpMessage::read_from(&mut stream) {
            Ok(m) => m,
            Err(lumi_protocol::LmpError::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Box::new(e)),
        };

        let uri = msg.header.uri.parse::<LumiUri>().unwrap_or(LumiUri {
            host: "welcome.lumi".to_string(),
            port: 7878,
            path: "/".to_string(),
        });

        println!(
            "[lumid] [Stream #{}] Request URI: 'lumi://{}{}'",
            msg.stream_id, uri.host, uri.path
        );

        let (content_type, payload_bytes) =
            if let Some((ref custom_domain, ref custom_bytes)) = custom_site {
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
    let pkg = match host {
        "welcome.lumi" | "welcome.home" => LumiPackage::new_md("Welcome Portal", WELCOME_MD),
        "search.lumi" | "search.home" => {
            if let Some(q_idx) = path.find("q=") {
                let term = &path[q_idx + 2..];
                let decoded = term.replace('+', " ").replace("%20", " ");
                let term_lower = decoded.to_lowercase();

                let all_links = vec![
                    (
                        "docs.lumi",
                        "Official LMP Core Protocol RFC Specifications & Architecture",
                    ),
                    ("welcome.lumi", "Main Gateway & Ecosystem Overview"),
                    ("games.lumi", "Interactive Native Web Games"),
                    ("wiki.lumi", "Open Lumi Ecosystem Knowledgebase"),
                    ("chat.lumi", "Encrypted Peer Node Communication"),
                    ("gallery.lumi", "Native LumiML UI Component Showcase"),
                ];

                let mut results = Vec::new();
                for (domain, desc) in all_links {
                    if domain.contains(&term_lower) || desc.to_lowercase().contains(&term_lower) {
                        results.push(format!(
                            "- [{}]({}://{}) - {}",
                            domain, "lumi", domain, desc
                        ));
                    }
                }

                let results_str = if results.is_empty() {
                    format!("No domains found matching '{}'.", decoded)
                } else {
                    results.join("\n")
                };

                let dynamic_search_md = format!(
                    r#"# Search Results for "{}"

Privacy-first search indexing results.

---

## Matches Found:

{}

---

[🔍 New Search / Back to Search Index](lumi://search.lumi)
[← Return to Welcome Portal](lumi://welcome.lumi)
"#,
                    decoded, results_str
                );
                LumiPackage::new_md("Lumi Search Results", &dynamic_search_md)
            } else {
                LumiPackage::new_md("Lumi Search", SEARCH_MD)
            }
        }
        "docs.lumi" | "docs.home" => LumiPackage::new_md("Docs", DOCS_MD),
        "chat.lumi" | "chat.home" => LumiPackage::new_lml("Encrypted Chat", CHAT_PAGE),
        "gallery.lumi" | "gallery.home" => LumiPackage::new_lml("Gallery", GALLERY_PAGE),
        "games.lumi" => LumiPackage::new_lml("Lumi Arcade", GAMES_PAGE),
        "wiki.lumi" => LumiPackage::new_md("Lumi Wiki", WIKI_MD),
        _ => LumiPackage::new_md("Welcome Portal", WELCOME_MD),
    };

    let bytes = pkg.to_bytes().unwrap_or_default();
    ("application/lpkg", bytes)
}

const WELCOME_MD: &str = r#"# The Lumi Project

> **Lumi Ecosystem Specification • v0.4.0-alpha**

An experimental, open-source, privacy-first web ecosystem built from scratch in Rust by **Satwik Bajpai**.

---

## Overview

Lumi is an independent browsing platform consisting of a custom transport protocol (**LMP**), a lightweight document layout language (**LumiML**), a native CommonMark **Markdown** document renderer, and a binary packaging system (**LPKG**). The project operates with zero telemetry, zero advertising identifiers, and minimal resource footprint.

---

## Documentation & Ecosystem Links

Explore technical specifications, search indices, and sample application packages:

- [📖 Documentation & RFCs](lumi://docs.lumi)
- [🔍 Search Index](lumi://search.lumi)
- [📦 Showcase Gallery](lumi://gallery.lumi)
- [🎮 Lumi Arcade](lumi://games.lumi)
- [📚 Open Wiki](lumi://wiki.lumi)

---

## System Implementation Status

| Component | Architecture | Status |
|---|---|---|
| **Browser Daemon** | Native `egui` / `wgpu` | Active |
| **LMP Protocol** | Binary Multiplexed TCP (1.0) | Active |
| **Renderer Engine** | Dual LumiML + Native CommonMark Markdown | Active |
| **LumiML Lexer** | AST Parser & Tokenizer | Active |
| **Server Daemon** | `lumid` Daemon (`127.0.0.1:9001`) | Active |

---

## Source Code & License

- **Official Repository**: [github.com/sxwik/lumi](https://github.com/sxwik/lumi)
- **License**: Released under the **Apache License, Version 2.0**

*Lumi v0.4.0-alpha • Built with Rust • Apache License 2.0*
"#;

const SEARCH_MD: &str = r#"# Search the Lumi Network

> **Decentralized Domain Index**

Privacy-first search indexing registered `.lumi` domains across the network.

---

## Featured Ecosystem Domains

- [docs.lumi](lumi://docs.lumi) - Official LMP Core Protocol RFC Specifications & Architecture
- [welcome.lumi](lumi://welcome.lumi) - Main Gateway & Ecosystem Overview
- [games.lumi](lumi://games.lumi) - Interactive Native Web Games
- [wiki.lumi](lumi://wiki.lumi) - Open Lumi Ecosystem Knowledgebase
- [chat.lumi](lumi://chat.lumi) - Encrypted Peer Node Communication
- [gallery.lumi](lumi://gallery.lumi) - Native LumiML UI Component Showcase

---

[← Return to Welcome Portal](lumi://welcome.lumi)
"#;

const DOCS_MD: &str = r#"# Lumi Core Specifications & RFC Documentation

> **Official Standards (RFC 0001 - 0005)**

Lumi features formal Request for Comments (RFCs) governing protocol and ecosystem development.

---

## Published RFC Standards

1. **RFC-0001**: LMP Core Binary Multiplexed Protocol Standard
2. **RFC-0002**: LumiML Markup Grammar & AST Specification
3. **RFC-0003**: Lumi Name Service (LNS) Domain Resolution Protocol
4. **RFC-0004**: LPKG Package Archive Format (`index.md` / `index.lml`)
5. **RFC-0005**: Lumi Browser Extension (`.lpx`) API

---

## Content Type Guidelines

- **Markdown (`index.md`)**: Used for documentation, RFCs, wikis, articles, blogs, and text content.
- **LumiML (`index.lml`)**: Used for interactive applications, games, dashboards, chat, and custom UI.

---

[← Back to Search](lumi://search.lumi)
"#;

const WIKI_MD: &str = r#"# Lumi Ecosystem Open Wiki

> **Community Architecture Knowledgebase**

Learn how to build custom servers, AST parsers, and custom Lumi browser engines from open RFC standards.

---

## Architecture Overview

Lumi decouples content presentation into two distinct engines:

```
Documentation (.md)  ──>  Native CommonMark Markdown Renderer
Applications (.lml) ──>  LumiML AST Layout Engine
```

All network traffic is transported framed inside persistent **LMP** TCP streams.

---

[Read RFC Specifications](lumi://docs.lumi) | [Return Home](lumi://welcome.lumi)
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
