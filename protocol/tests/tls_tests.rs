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

#[test]
fn test_realtime_chat_broadcast_over_tls() {
    use lumi_protocol::{ChatMessagePayload, PacketType};
    use std::sync::mpsc::channel;

    let (certs, key, _cert_pem, _key_pem) = tls::generate_dev_certs().unwrap();
    let server_config = tls::make_server_config(certs, key).unwrap();
    let client_config1 = tls::make_dev_client_config().unwrap();
    let client_config2 = tls::make_dev_client_config().unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let server_addr = listener.local_addr().unwrap().to_string();

    let (tx_done, rx_done) = channel();

    let server_handle = thread::spawn(move || {
        let (s1, _) = listener.accept().unwrap();
        let mut stream1 = tls::accept_tls(s1, server_config.clone()).unwrap();
        let (s2, _) = listener.accept().unwrap();
        let mut stream2 = tls::accept_tls(s2, server_config).unwrap();

        let msg = LmpMessage::read_from(&mut stream1).unwrap();
        assert_eq!(msg.packet_type, PacketType::ChatMessage);

        msg.write_to(&mut stream2).unwrap();
        rx_done.recv().unwrap();
    });

    let addr1 = server_addr.clone();
    let client1_handle = thread::spawn(move || {
        let mut stream1 = tls::connect_tls(&addr1, "localhost", client_config1).unwrap();
        let chat_msg = LmpMessage::new_chat_message(1, "Alice", "Hello Bob!", "14:00");
        chat_msg.write_to(&mut stream1).unwrap();
        thread::sleep(std::time::Duration::from_millis(200));
    });

    let addr2 = server_addr;
    let client2_handle = thread::spawn(move || {
        let mut stream2 = tls::connect_tls(&addr2, "localhost", client_config2).unwrap();
        let msg = LmpMessage::read_from(&mut stream2).unwrap();
        assert_eq!(msg.packet_type, PacketType::ChatMessage);
        let payload = ChatMessagePayload::from_slice(&msg.payload).unwrap();
        assert_eq!(payload.username, "Alice");
        assert_eq!(payload.content, "Hello Bob!");
        tx_done.send(()).unwrap();
    });

    client1_handle.join().unwrap();
    client2_handle.join().unwrap();
    server_handle.join().unwrap();
}
