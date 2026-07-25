use lumi_protocol::tls;
use lumi_protocol::{LmpError, LmpMessage};
use std::net::TcpListener;
use std::thread;

#[test]
fn test_successful_tls_handshake_and_lmp_exchange() {
    let (certs, key, _cert_pem, _key_pem) = tls::generate_dev_certs().unwrap();
    let server_config = tls::make_server_config(certs, key).unwrap();
    let client_config = tls::make_dev_client_config().unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let server_addr = listener.local_addr().unwrap().to_string();

    let server_handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut tls_stream = tls::accept_tls(stream, server_config).unwrap();

        let msg = LmpMessage::read_from(&mut tls_stream).unwrap();
        assert_eq!(msg.header.uri, "lumi://welcome.lumi");

        let response = LmpMessage::new_response(msg.stream_id, "text/plain", b"Hello TLS".to_vec());
        response.write_to(&mut tls_stream).unwrap();
    });

    let client_handle = thread::spawn(move || {
        let mut tls_stream = tls::connect_tls(&server_addr, "localhost", client_config).unwrap();

        let req = LmpMessage::new_request("lumi://welcome.lumi", 1);
        req.write_to(&mut tls_stream).unwrap();

        let res = LmpMessage::read_from(&mut tls_stream).unwrap();
        assert_eq!(res.header.status_code, 200);
        assert_eq!(res.payload, b"Hello TLS");
    });

    server_handle.join().unwrap();
    client_handle.join().unwrap();
}

#[test]
fn test_failed_tls_handshake_with_untrusted_cert() {
    let (certs, key, _cert_pem, _key_pem) = tls::generate_dev_certs().unwrap();
    let server_config = tls::make_server_config(certs, key).unwrap();

    // Strict client config with empty root store will reject the self-signed dev cert
    let strict_client_config = tls::make_client_config(None).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let server_addr = listener.local_addr().unwrap().to_string();

    let server_handle = thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            let _ = tls::accept_tls(stream, server_config);
        }
    });

    let client_res = tls::connect_tls(&server_addr, "localhost", strict_client_config);
    assert!(client_res.is_err());

    server_handle.join().unwrap();
}

#[test]
fn test_graceful_handling_of_tls_connection_failures() {
    let client_config = tls::make_dev_client_config().unwrap();
    // Connecting to an invalid / non-listening port should return Err gracefully without panic
    let res = tls::connect_tls("127.0.0.1:59999", "localhost", client_config);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), LmpError::Io(_)));
}
