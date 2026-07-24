# Developer Quickstart Guide - Lumi Web Ecosystem

Get a privacy-first website live on the Lumi network in under 5 minutes!

---

## 1. Prerequisites
- Install Rust & Cargo: [https://rustup.rs/](https://rustup.rs/)

---

## 2. Step-by-Step Tutorial

### Step 1: Install `lumi-cli`
```bash
cargo install --path cli
```

### Step 2: Scaffold Your First Site
```bash
lumi new my-first-site
cd my-first-site
```

This creates:
```text
my-first-site/
├── manifest.toml
├── index.lml
└── assets/
```

### Step 3: Edit Your LumiML Page (`index.lml`)
```lumiml
page {
    title "My First Site"

    heading {
        text "Hello Lumi Network!"
    }

    paragraph {
        text "This is my decentralized, zero-telemetry website."
    }

    button {
        text "Search Lumi"
        goto "lumi://search.lumi"
    }
}
```

### Step 4: Build Package Archive (`.lpkg`)
```bash
lumi pack . my-first-site.lpkg
```

### Step 5: Host Your Site with `lumid`
```bash
cargo run -p lumid -- serve my-first-site.lpkg --domain hello.lumi
```

### Step 6: Browse!
Launch `lumi-browser` and navigate to:
```text
lumi://hello.lumi
```
