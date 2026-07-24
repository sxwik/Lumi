# Lumi Privacy Web Ecosystem

Lumi is an experimental, open-source, privacy-first web ecosystem built entirely from scratch in Rust (Zero Chromium, Zero Electron, Zero HTML/CSS/JS).

---

## 🌟 What is Lumi?

Lumi is an independent browsing platform and network specification designed around maximum performance, zero telemetry, and complete user privacy.

### Core Architectural Features:
- **Dual Presentation Pipeline**:
  - 📄 **Native CommonMark Markdown (`index.md`)**: Renders documentation, RFCs, blogs, wikis, and static text content cleanly using pure Rust native UI widgets.
  - ⚡ **LumiML Engine (`index.lml`)**: Renders interactive applications, dashboards, chat systems, games, and rich UI elements via a custom AST layout parser.
- **LMP Transport Protocol (RFC-0001)**: A binary-multiplexed TCP network protocol running over default port `9001` with zero tracking, zero header bloat, and instant stream multiplexing.
- **LNS Resolver (RFC-0003)**: Lumi Name Service for resolving `.lumi` domain names (e.g. `lumi://welcome.lumi`, `lumi://docs.lumi`).
- **LPKG Package Format (RFC-0004)**: Binary site archive format bundling `index.md` or `index.lml` along with asset resources.

---

## 📚 Ecosystem Documentation & Specifications

- 📄 **[Lumi Technical Whitepaper](docs/LUMI_WHITEPAPER.md)**: Architecture, Privacy Model, & Security Analysis.
- 🚀 **[Developer Quickstart Guide](docs/DEVELOPER_GUIDE.md)**: 5-minute tutorial to build and host Lumi sites.
- 📑 **Formal RFC Specifications**:
  - [RFC-0001: LMP Core Binary Protocol](docs/rfcs/RFC-0001-LMP.md)
  - [RFC-0002: LumiML Markup Standard](docs/rfcs/RFC-0002-LumiML.md)
  - [RFC-0003: Lumi Name Service (LNS)](docs/rfcs/RFC-0003-LNS.md)
  - [RFC-0004: LPKG Package Standard](docs/rfcs/RFC-0004-LPKG.md)
  - [RFC-0005: Lumi Extension API (.lpx)](docs/rfcs/RFC-0005-Extension-API.md)
  - [RFC-0006: Open Governance Model](docs/rfcs/RFC-0006-Community-Governance.md)
  - [RFC-0007: Security & Threat Model](docs/rfcs/RFC-0007-Security-Model.md)

---

## 🛠 Project Components

```text
lumi/
├── docs/                     (Whitepaper, Quickstart Guide, and RFCs)
├── protocol/                 (LMP framing, LNS resolver, .lpkg format)
├── parser/                   (LumiML tokenizer, parser & AST)
├── renderer/                 (Dual CommonMark Markdown & LumiML layout engine)
├── server/                   (lumid - Serves .lpkg site packages on port 9001)
├── browser/                  (Lumi browser app supporting *.lumi & persistent sessions)
└── cli/                      (Lumi SDK CLI - `lumi new` & `lumi pack`)
```

---

## 🚀 Quickstart

### 1. Launch the Lumi Server Daemon (`lumid`)
```bash
cargo run -p lumid
```

### 2. Launch the Lumi Browser
```bash
cargo run -p lumi-browser
```
Navigate to `lumi://welcome.lumi`, `lumi://docs.lumi`, or `lumi://search.lumi`!

### 3. Build a Lumi Package (`lumi-cli`)
```bash
cargo run -p lumi-cli -- new my-site
cargo run -p lumi-cli -- pack my-site my-site.lpkg
```

---

## 📜 License

Released under the **Apache License, Version 2.0**.
