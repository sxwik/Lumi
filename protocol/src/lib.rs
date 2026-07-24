use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use thiserror::Error;

pub const MAGIC_BYTES: &[u8; 4] = b"LUMI";
pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Error, Debug)]
pub enum LmpError {
    #[error("Invalid scheme in URI. Expected 'lumi://'")]
    InvalidScheme,
    #[error("Invalid magic bytes in LMP packet frame")]
    InvalidMagic,
    #[error("Unsupported LMP protocol version: {0}")]
    UnsupportedVersion(u8),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Packet frame payload too large: {0} bytes")]
    PayloadTooLarge(usize),
    #[error("LNS resolution failed for host: {0}")]
    LnsResolutionFailed(String),
}

/// Lumi Name Service (LNS) Resolver
pub struct LnsResolver {
    records: HashMap<String, String>,
}

impl LnsResolver {
    pub fn new() -> Self {
        let mut records = HashMap::new();
        // Built-in default .lumi domain routes
        records.insert("docs.lumi".to_string(), "127.0.0.1:9001".to_string());
        records.insert("chat.lumi".to_string(), "127.0.0.1:9001".to_string());
        records.insert("gallery.lumi".to_string(), "127.0.0.1:9001".to_string());
        records.insert("store.lumi".to_string(), "127.0.0.1:9001".to_string());

        // Backwards compatibility for .home
        records.insert("docs.home".to_string(), "127.0.0.1:9001".to_string());
        records.insert("chat.home".to_string(), "127.0.0.1:9001".to_string());
        records.insert("gallery.home".to_string(), "127.0.0.1:9001".to_string());

        Self { records }
    }

    pub fn resolve(&self, host: &str) -> Result<String, LmpError> {
        if host.contains(':') {
            return Ok(host.to_string());
        }
        self.records
            .get(host)
            .cloned()
            .ok_or_else(|| LmpError::LnsResolutionFailed(host.to_string()))
    }
}

impl Default for LnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LumiUri {
    pub host: String,
    pub port: u16,
    pub path: String,
}

impl LumiUri {
    pub fn parse(input: &str) -> Result<Self, LmpError> {
        let trimmed = input.trim();
        if !trimmed.starts_with("lumi://") {
            return Err(LmpError::InvalidScheme);
        }

        let rest = &trimmed["lumi://".len()..];
        let mut parts = rest.splitn(2, '/');
        let host_port = parts.next().unwrap_or("");
        let path = format!("/{}", parts.next().unwrap_or(""));

        let mut hp_split = host_port.splitn(2, ':');
        let host = hp_split.next().unwrap_or("docs.lumi").to_string();
        let port = hp_split
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(9001);

        Ok(LumiUri { host, port, path })
    }

    pub fn to_string_uri(&self) -> String {
        format!("lumi://{}{}", self.host, self.path)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketType {
    Request = 1,
    Response = 2,
    Ping = 3,
    Pong = 4,
    Error = 5,
}

impl PacketType {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(PacketType::Request),
            2 => Some(PacketType::Response),
            3 => Some(PacketType::Ping),
            4 => Some(PacketType::Pong),
            5 => Some(PacketType::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmpHeader {
    pub uri: String,
    pub method: String,
    pub status_code: u16,
    pub status_message: String,
    pub content_type: String,
    pub metadata: HashMap<String, String>,
}

impl Default for LmpHeader {
    fn default() -> Self {
        Self {
            uri: String::new(),
            method: "GET".to_string(),
            status_code: 200,
            status_message: "OK".to_string(),
            content_type: "text/lumiml".to_string(),
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LmpMessage {
    pub packet_type: PacketType,
    pub stream_id: u32,
    pub header: LmpHeader,
    pub payload: Vec<u8>,
}

impl LmpMessage {
    pub fn new_request(uri: &str, stream_id: u32) -> Self {
        Self {
            packet_type: PacketType::Request,
            stream_id,
            header: LmpHeader {
                uri: uri.to_string(),
                method: "GET".to_string(),
                ..Default::default()
            },
            payload: Vec::new(),
        }
    }

    pub fn new_response(stream_id: u32, content_type: &str, payload: Vec<u8>) -> Self {
        Self {
            packet_type: PacketType::Response,
            stream_id,
            header: LmpHeader {
                status_code: 200,
                status_message: "OK".to_string(),
                content_type: content_type.to_string(),
                ..Default::default()
            },
            payload,
        }
    }

    pub fn new_error(stream_id: u32, status_code: u16, message: &str) -> Self {
        Self {
            packet_type: PacketType::Error,
            stream_id,
            header: LmpHeader {
                status_code,
                status_message: message.to_string(),
                content_type: "text/plain".to_string(),
                ..Default::default()
            },
            payload: message.as_bytes().to_vec(),
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), LmpError> {
        let header_json = serde_json::to_vec(&self.header)?;
        let header_len = header_json.len() as u32;
        let payload_len = self.payload.len() as u32;

        writer.write_all(MAGIC_BYTES)?;
        writer.write_all(&[PROTOCOL_VERSION, self.packet_type as u8])?;
        writer.write_all(&self.stream_id.to_be_bytes())?;
        writer.write_all(&header_len.to_be_bytes())?;
        writer.write_all(&payload_len.to_be_bytes())?;

        writer.write_all(&header_json)?;
        writer.write_all(&self.payload)?;
        writer.flush()?;

        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, LmpError> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC_BYTES {
            return Err(LmpError::InvalidMagic);
        }

        let mut ver_type = [0u8; 2];
        reader.read_exact(&mut ver_type)?;
        if ver_type[0] != PROTOCOL_VERSION {
            return Err(LmpError::UnsupportedVersion(ver_type[0]));
        }

        let packet_type = PacketType::from_u8(ver_type[1]).ok_or_else(|| {
            LmpError::Serialization(serde::de::Error::custom("Invalid packet type"))
        })?;

        let mut stream_id_bytes = [0u8; 4];
        reader.read_exact(&mut stream_id_bytes)?;
        let stream_id = u32::from_be_bytes(stream_id_bytes);

        let mut header_len_bytes = [0u8; 4];
        reader.read_exact(&mut header_len_bytes)?;
        let header_len = u32::from_be_bytes(header_len_bytes) as usize;

        let mut payload_len_bytes = [0u8; 4];
        reader.read_exact(&mut payload_len_bytes)?;
        let payload_len = u32::from_be_bytes(payload_len_bytes) as usize;

        if header_len > 10 * 1024 * 1024 || payload_len > 100 * 1024 * 1024 {
            return Err(LmpError::PayloadTooLarge(payload_len));
        }

        let mut header_buf = vec![0u8; header_len];
        reader.read_exact(&mut header_buf)?;
        let header: LmpHeader = serde_json::from_slice(&header_buf)?;

        let mut payload_buf = vec![0u8; payload_len];
        reader.read_exact(&mut payload_buf)?;

        Ok(LmpMessage {
            packet_type,
            stream_id,
            header,
            payload: payload_buf,
        })
    }
}

/// Simple Lumi Package Archive (.lpkg) Format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumiPackage {
    pub name: String,
    pub version: String,
    pub index_lml: String,
    pub assets: HashMap<String, Vec<u8>>,
}

impl LumiPackage {
    pub fn new(name: &str, index_lml: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            index_lml: index_lml.to_string(),
            assets: HashMap::new(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, LmpError> {
        let json = serde_json::to_vec(self)?;
        Ok(json)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, LmpError> {
        let pkg: Self = serde_json::from_slice(bytes)?;
        Ok(pkg)
    }
}

#[cfg(test)]
mod tests;
