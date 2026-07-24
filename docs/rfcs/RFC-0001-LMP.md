# RFC-0001: Lumi Messaging Protocol (LMP) Core Specification

**Category:** Core Standard  
**Status:** Draft  
**Author:** Lumi Open Source Community  

---

## 1. Abstract
This RFC defines the binary framing, transport lifecycle, multiplexing capabilities, and error handling for the Lumi Messaging Protocol (LMP).

## 2. Framing
All data frames transmitted over TCP start with a 16-byte fixed binary header:
- `Magic (4 bytes)`: ASCII "LUMI" (`0x4C 0x55 0x4D 0x49`)
- `Version (1 byte)`: Current `0x01`
- `Packet Type (1 byte)`: REQUEST (1), RESPONSE (2), PING (3), PONG (4), ERROR (5)
- `Stream ID (4 bytes)`: Big-endian integer identifying stream channel
- `Header Length (4 bytes)`: Big-endian length of JSON header frame
- `Payload Length (4 bytes)`: Big-endian length of body payload

## 3. Persistent Channels
A client MUST establish a persistent TCP session per LNS resolved endpoint. Multiple requests MUST use distinct `stream_id` values on the same underlying connection.
