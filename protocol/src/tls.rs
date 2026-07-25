use crate::LmpError;
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
pub use rustls::{ClientConfig, ServerConfig};
use rustls::{ClientConnection, RootCertStore, ServerConnection, StreamOwned};
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;

pub fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, LmpError> {
    let certfile = File::open(path).map_err(|e| {
        LmpError::Certificate(format!("Failed to open cert file {:?}: {}", path, e))
    })?;
    let mut reader = BufReader::new(certfile);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            LmpError::Certificate(format!("Failed to parse cert file {:?}: {}", path, e))
        })?;
    if certs.is_empty() {
        return Err(LmpError::Certificate(format!(
            "No certificates found in {:?}",
            path
        )));
    }
    Ok(certs)
}

pub fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>, LmpError> {
    let keyfile = File::open(path)
        .map_err(|e| LmpError::Certificate(format!("Failed to open key file {:?}: {}", path, e)))?;
    let mut reader = BufReader::new(keyfile);
    let key = rustls_pemfile::private_key(&mut reader)
        .map_err(|e| LmpError::Certificate(format!("Failed to parse key file {:?}: {}", path, e)))?
        .ok_or_else(|| LmpError::Certificate(format!("No private key found in {:?}", path)))?;
    Ok(key)
}

pub fn generate_dev_certs() -> Result<
    (
        Vec<CertificateDer<'static>>,
        PrivateKeyDer<'static>,
        String,
        String,
    ),
    LmpError,
> {
    let subject_alt_names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "welcome.lumi".to_string(),
        "search.lumi".to_string(),
        "docs.lumi".to_string(),
        "chat.lumi".to_string(),
        "gallery.lumi".to_string(),
        "games.lumi".to_string(),
        "wiki.lumi".to_string(),
    ];

    let certified_key = generate_simple_self_signed(subject_alt_names).map_err(|e| {
        LmpError::Certificate(format!("Failed to generate self-signed cert: {}", e))
    })?;

    let cert_pem = certified_key.cert.pem();
    let key_pem = certified_key.key_pair.serialize_pem();

    let cert_der = certified_key.cert.der().to_vec();
    let key_der = certified_key.key_pair.serialize_der();

    Ok((
        vec![CertificateDer::from(cert_der)],
        PrivateKeyDer::Pkcs8(key_der.into()),
        cert_pem,
        key_pem,
    ))
}

pub fn load_or_generate_server_certs(
    cert_path: &Path,
    key_path: &Path,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), LmpError> {
    if cert_path.exists() && key_path.exists() {
        let certs = load_certs(cert_path)?;
        let key = load_private_key(key_path)?;
        Ok((certs, key))
    } else {
        println!(
            "[LMP TLS] Certificates missing. Generating development self-signed certs at {:?} and {:?}",
            cert_path, key_path
        );
        let (certs, key, cert_pem, key_pem) = generate_dev_certs()?;

        if let Some(parent) = cert_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Some(parent) = key_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        fs::write(cert_path, cert_pem)
            .map_err(|e| LmpError::Certificate(format!("Failed to write dev cert: {}", e)))?;
        fs::write(key_path, key_pem)
            .map_err(|e| LmpError::Certificate(format!("Failed to write dev key: {}", e)))?;

        Ok((certs, key))
    }
}

pub fn ensure_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

pub fn make_server_config(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> Result<Arc<ServerConfig>, LmpError> {
    ensure_crypto_provider();
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| LmpError::Tls(format!("Failed to configure TLS server: {}", e)))?;

    Ok(Arc::new(config))
}

pub fn make_client_config(
    root_certs: Option<&[CertificateDer<'_>]>,
) -> Result<Arc<ClientConfig>, LmpError> {
    ensure_crypto_provider();
    let mut root_store = RootCertStore::empty();
    if let Some(certs) = root_certs {
        for cert in certs {
            root_store
                .add(cert.clone())
                .map_err(|e| LmpError::Tls(format!("Failed to add root cert: {}", e)))?;
        }
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(Arc::new(config))
}

#[derive(Debug)]
struct DevServerCertVerifier;

impl rustls::client::danger::ServerCertVerifier for DevServerCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::RSA_PSS_SHA256,
        ]
    }
}

pub fn make_dev_client_config() -> Result<Arc<ClientConfig>, LmpError> {
    ensure_crypto_provider();
    let verifier = Arc::new(DevServerCertVerifier);
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();

    Ok(Arc::new(config))
}

#[derive(Debug)]
pub enum LmpStream {
    Client(StreamOwned<ClientConnection, TcpStream>),
    Server(StreamOwned<ServerConnection, TcpStream>),
}

impl Read for LmpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            LmpStream::Client(s) => s.read(buf),
            LmpStream::Server(s) => s.read(buf),
        }
    }
}

impl Write for LmpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            LmpStream::Client(s) => s.write(buf),
            LmpStream::Server(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            LmpStream::Client(s) => s.flush(),
            LmpStream::Server(s) => s.flush(),
        }
    }
}

pub fn connect_tls(
    addr: &str,
    server_name_str: &str,
    config: Arc<ClientConfig>,
) -> Result<LmpStream, LmpError> {
    let tcp_stream = TcpStream::connect(addr).map_err(LmpError::Io)?;

    let name = ServerName::try_from(server_name_str.to_string()).map_err(|e| {
        LmpError::Tls(format!(
            "Invalid TLS server name '{}': {}",
            server_name_str, e
        ))
    })?;

    let client_conn = ClientConnection::new(config, name)
        .map_err(|e| LmpError::Tls(format!("TLS client connection creation failed: {}", e)))?;

    let mut stream = StreamOwned::new(client_conn, tcp_stream);

    // Complete TLS handshake
    if let Err(e) = stream.flush() {
        return Err(LmpError::Tls(format!(
            "TLS handshake failed during connect: {}",
            e
        )));
    }

    Ok(LmpStream::Client(stream))
}

pub fn accept_tls(tcp_stream: TcpStream, config: Arc<ServerConfig>) -> Result<LmpStream, LmpError> {
    let server_conn = ServerConnection::new(config)
        .map_err(|e| LmpError::Tls(format!("TLS server connection creation failed: {}", e)))?;

    let mut stream = StreamOwned::new(server_conn, tcp_stream);

    // Complete TLS handshake
    if let Err(e) = stream.flush() {
        return Err(LmpError::Tls(format!(
            "TLS handshake failed during accept: {}",
            e
        )));
    }

    Ok(LmpStream::Server(stream))
}
