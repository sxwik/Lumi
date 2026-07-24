# LMP 1.0 Specification (Lumi Messaging Protocol)

**Status:** Official Standard / Draft  
**Scheme:** `lumi://`  
**Default Port:** `7878`  

---

## 1. Overview & Goals
The **Lumi Messaging Protocol (LMP)** is a binary, multiplexed, low-latency application protocol engineered specifically for privacy-first, zero-telemetry document and asset transfer.

---

## 2. Packet Framing Format
All LMP data transmitted across the transport layer (TCP/TLS) MUST strictly follow the binary header format:

```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       Magic Bytes ('L', 'U', 'M', 'I')                        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Version (0x01) | Packet Type  |        Stream ID              |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                               |  Header Length (32-bit BE)    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Payload Length (32-bit BE)    | Header JSON Payload (...)     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Binary Body Payload (...)                                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### Fields:
- **Magic Bytes**: 4 bytes `0x4C 0x55 0x4D 0x49` (`LUMI`).
- **Version**: 1 byte (current: `0x01`).
- **Packet Type**: 1 byte enum:
  - `0x01`: REQUEST
  - `0x02`: RESPONSE
  - `0x03`: PING
  - `0x04`: PONG
  - `0x05`: ERROR
- **Stream ID**: 4-bit / 32-bit Big-Endian integer identifying multiplexed stream channel.
- **Header Length**: 32-bit Big-Endian integer indicating size of JSON header metadata.
- **Payload Length**: 32-bit Big-Endian integer indicating size of binary document/asset payload.

---

## 3. Lumi Name Service (LNS)
LMP uses `.lumi` domain resolution instead of DNS.
- Example: `lumi://docs.lumi` -> Resolved via LNS routing table to IP:Port (`127.0.0.1:7878`).

---

## 4. Lumi Package Format (`.lpkg`)
Site content is packaged as a `.lpkg` single archive containing:
- `manifest.toml` (Site metadata, entry point)
- `index.lml` (Root LumiML document)
- `assets/` (Embedded images, assets)

---

## 5. Error Codes
- `200`: OK
- `400`: Bad Request / Syntax Error in LumiML
- `404`: Name / Resource Not Found in LNS
- `500`: Internal Server Error
