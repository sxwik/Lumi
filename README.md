# Lumi Web Ecosystem

Lumi is an experimental, open-source, privacy-first web ecosystem built completely from scratch in Rust (Zero Chromium, Zero Electron, Zero HTML/CSS/JS).

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
├── renderer/                 (Custom egui/wgpu layout engine)
├── server/                   (lumid - Serves .lpkg site packages & persistent LMP)
├── browser/                  (Lumi browser app supporting *.lumi & persistent sessions)
└── cli/                      (Lumi SDK CLI - `lumi new` & `lumi pack`)
```

---

## 🚀 Quickstart

### 1. Build a Lumi Website (`lumi-cli`)
```bash
cargo run -p lumi-cli -- new my-site
cargo run -p lumi-cli -- pack my-site my-site.lpkg
```

### 2. Launch the Lumi Server Daemon (`lumid`)
```bash
cargo run -p lumid
```

### 3. Launch the Lumi Browser
```bash
cargo run -p lumi-browser
```
Navigate to `lumi://welcome.lumi` or `lumi://search.lumi`!
