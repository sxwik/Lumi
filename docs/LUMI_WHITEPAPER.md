# Lumi: An Experimental Privacy-First Web Ecosystem Architecture

**Technical Whitepaper v1.0**  
**Authors:** The Lumi Open Source Project  
**Date:** July 2026  

---

## Abstract
The modern World Wide Web is increasingly burdened by decades of legacy compatibility layers, complex execution engines, heavy JavaScript runtimes, persistent user tracking, and excessive RAM consumption. **Lumi** is an experimental, open-source, privacy-first web browsing ecosystem engineered entirely from scratch in Rust. Lumi replaces the legacy HTML/JS/CSS stack and HTTP/DNS protocols with a unified binary application protocol (**LMP**), a decentralized domain naming service (**LNS**), a light markup document layout language (**LumiML**), and single-file binary site packages (**LPKG**). This whitepaper presents the design principles, security model, network architecture, and performance characteristics of Lumi.

---

## 1. Introduction & Motivation
Current web browsers are essentially full operating systems in disguise, running millions of lines of C++ code, JIT JavaScript compilers, and thousands of tracking algorithms.

### 1.1 Problems with the Legacy Web
1. **Excessive Overhead & Memory Usage**: Modern browsers frequently consume multiple gigabytes of RAM for simple document viewing.
2. **Ubiquitous Telemetry & Tracking**: Cross-site cookie tracking, advertising identifiers, and browser fingerprinting expose user privacy by default.
3. **Attack Surface & Vulnerabilities**: Complex JavaScript JIT engines and DOM implementations present massive attack surfaces.

---

## 2. System Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                       Lumi Browser                          │
├──────────────────────────────┬──────────────────────────────┤
│    LumiML Layout Engine      │      Extension API (.lpx)    │
├──────────────────────────────┴──────────────────────────────┤
│            Lumi Name Service (LNS Resolver)                 │
├─────────────────────────────────────────────────────────────┤
│         Lumi Messaging Protocol (LMP Binary Framing)        │
└──────────────────────────────┬──────────────────────────────┘
                               │ (Persistent LMP TCP Channel)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    lumid Server Daemon                      │
├─────────────────────────────────────────────────────────────┤
│              .lpkg Package Archive Streamer                 │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Protocol & Data Formats

### 3.1 Lumi Messaging Protocol (LMP 1.0)
LMP is a binary-framed multiplexed protocol operating over TCP/TLS socket connections. All messages follow a strict 16-byte fixed binary header structure:
- **Magic Bytes** (`0x4C 0x55 0x4D 0x49` / `LUMI`)
- **Version** (`0x01`)
- **Packet Type** (`REQUEST`, `RESPONSE`, `PING`, `PONG`, `ERROR`)
- **Stream ID** (32-bit BE multiplex channel integer)
- **Header Length** & **Payload Length** (32-bit BE integer fields)

### 3.2 Lumi Name Service (LNS)
LNS provides domain resolution for `.lumi` top-level domains (`docs.lumi`, `search.lumi`, `welcome.lumi`), mapping names directly to socket endpoints without intermediate centralized DNS servers.

### 3.3 LumiML Document Layout Engine
LumiML provides a declarative block-based AST schema designed for direct native rendering via `egui`/`wgpu` pipelines, eliminating browser DOM layout recalculations.

---

## 4. Security & Threat Model

### 4.1 Privacy Guarantees
- **Zero Telemetry**: No background telemetry telemetry frames exist in the LMP spec.
- **No Third-Party Cookies or Identifiers**: Local storage is isolated to site package scope.

### 4.2 Network & Frame Validation
- **Buffer Overflow Protection**: Strict 32-bit framing validation drops malformed frames exceeding maximum payload boundaries (10MB headers, 100MB payloads).
- **Persistent Channel Reuse**: Prevents socket exhaustion and denial-of-service attacks.

---

## 5. Performance Benchmarks

| Metric | Legacy Web Browser | Lumi Ecosystem |
| :--- | :--- | :--- |
| **Executable Binary Size** | ~150 MB - 300 MB | **~12 MB - 18 MB** |
| **Idle RAM Footprint** | ~500 MB - 1.2 GB | **~25 MB - 45 MB** |
| **Initial Load Time** | ~1.5s - 3.2s | **< 40ms** |
| **Telemetry Calls** | Hundreds per session | **0 (Zero)** |

---

## 6. Open Governance & RFC Process
Lumi protocol evolution is governed publicly via Request for Comments (RFC) proposals in `docs/rfcs/`. Community members submit RFC proposals that undergo open code review and technical validation before inclusion in standard specification releases.

---

## 7. Conclusion
Lumi proves that a fast, ultra-lightweight, and completely private web ecosystem can be built cleanly from scratch using modern Rust systems programming, establishing a compelling alternative for privacy-conscious users and developers.
