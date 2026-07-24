#[cfg(test)]
mod tests {
    use crate::{LmpError, LmpMessage, LnsResolver, LumiPackage, LumiUri, PacketType};
    use lumi_parser::parse;

    #[test]
    fn test_lns_resolution_success_and_failure() {
        let resolver = LnsResolver::new();
        assert_eq!(resolver.resolve("docs.lumi").unwrap(), "127.0.0.1:9001");
        assert_eq!(resolver.resolve("search.lumi").unwrap(), "127.0.0.1:9001");
        assert_eq!(resolver.resolve("welcome.lumi").unwrap(), "127.0.0.1:9001");
        assert_eq!(resolver.resolve("chat.lumi").unwrap(), "127.0.0.1:9001");

        // Pass-through for host with explicit port
        assert_eq!(
            resolver.resolve("custom.host:8080").unwrap(),
            "custom.host:8080"
        );

        // Failure cases
        assert!(matches!(
            resolver.resolve("nonexistent.invalid"),
            Err(LmpError::LnsResolutionFailed(host)) if host == "nonexistent.invalid"
        ));
    }

    #[test]
    fn test_lumi_uri_parsing_success_and_failure() {
        // Successful parses
        let uri = LumiUri::parse("lumi://docs.lumi/index.lml").unwrap();
        assert_eq!(uri.host, "docs.lumi");
        assert_eq!(uri.port, 9001);
        assert_eq!(uri.path, "/index.lml");
        assert_eq!(uri.to_string_uri(), "lumi://docs.lumi/index.lml");

        let uri_custom_port = LumiUri::parse("lumi://127.0.0.1:8000/api/v1").unwrap();
        assert_eq!(uri_custom_port.host, "127.0.0.1");
        assert_eq!(uri_custom_port.port, 8000);
        assert_eq!(uri_custom_port.path, "/api/v1");

        // Invalid scheme failure
        assert!(matches!(
            LumiUri::parse("http://docs.lumi"),
            Err(LmpError::InvalidScheme)
        ));
        assert!(matches!(
            LumiUri::parse("https://welcome.lumi"),
            Err(LmpError::InvalidScheme)
        ));
    }

    #[test]
    fn test_lmp_packet_types_conversion() {
        assert_eq!(PacketType::from_u8(1), Some(PacketType::Request));
        assert_eq!(PacketType::from_u8(2), Some(PacketType::Response));
        assert_eq!(PacketType::from_u8(3), Some(PacketType::Ping));
        assert_eq!(PacketType::from_u8(4), Some(PacketType::Pong));
        assert_eq!(PacketType::from_u8(5), Some(PacketType::Error));
        assert_eq!(PacketType::from_u8(0), None);
        assert_eq!(PacketType::from_u8(99), None);
    }

    #[test]
    fn test_lmp_framing_success_and_failure_cases() {
        // Standard request framing
        let req = LmpMessage::new_request("lumi://docs.lumi", 42);
        let mut buf = Vec::new();
        req.write_to(&mut buf).unwrap();

        let decoded = LmpMessage::read_from(&mut &buf[..]).unwrap();
        assert_eq!(decoded.packet_type, PacketType::Request);
        assert_eq!(decoded.stream_id, 42);
        assert_eq!(decoded.header.uri, "lumi://docs.lumi");

        // Standard response framing
        let resp = LmpMessage::new_response(101, "text/markdown", b"# Hello".to_vec());
        let mut resp_buf = Vec::new();
        resp.write_to(&mut resp_buf).unwrap();

        let decoded_resp = LmpMessage::read_from(&mut &resp_buf[..]).unwrap();
        assert_eq!(decoded_resp.packet_type, PacketType::Response);
        assert_eq!(decoded_resp.stream_id, 101);
        assert_eq!(decoded_resp.header.status_code, 200);
        assert_eq!(decoded_resp.payload, b"# Hello");

        // Error message framing
        let err_msg = LmpMessage::new_error(99, 404, "Not Found");
        let mut err_buf = Vec::new();
        err_msg.write_to(&mut err_buf).unwrap();

        let decoded_err = LmpMessage::read_from(&mut &err_buf[..]).unwrap();
        assert_eq!(decoded_err.packet_type, PacketType::Error);
        assert_eq!(decoded_err.header.status_code, 404);
        assert_eq!(decoded_err.header.status_message, "Not Found");

        // Failure Case 1: Invalid Magic Bytes
        let mut bad_magic = buf.clone();
        bad_magic[0] = b'X';
        assert!(matches!(
            LmpMessage::read_from(&mut &bad_magic[..]),
            Err(LmpError::InvalidMagic)
        ));

        // Failure Case 2: Unsupported Protocol Version
        let mut bad_version = buf.clone();
        bad_version[4] = 99; // Version offset
        assert!(matches!(
            LmpMessage::read_from(&mut &bad_version[..]),
            Err(LmpError::UnsupportedVersion(99))
        ));

        // Failure Case 3: Invalid Packet Type
        let mut bad_type = buf.clone();
        bad_type[5] = 255;
        assert!(LmpMessage::read_from(&mut &bad_type[..]).is_err());
    }

    #[test]
    fn test_lpkg_bundle_cycle_lml_and_md() {
        let lml = "page { title \"Test Site\" paragraph { text \"Hello Lumi\" } }";
        let pkg_lml = LumiPackage::new_lml("TestLml", lml);

        let bytes_lml = pkg_lml.to_bytes().unwrap();
        let unpacked_lml = LumiPackage::from_bytes(&bytes_lml).unwrap();

        assert_eq!(unpacked_lml.name, "TestLml");
        assert_eq!(unpacked_lml.index_lml, Some(lml.to_string()));
        assert!(unpacked_lml.index_md.is_none());

        let ast = parse(unpacked_lml.index_lml.as_ref().unwrap()).unwrap();
        assert_eq!(ast.element_type, lumi_parser::ElementType::Page);

        // MD Package cycle
        let md = "# Welcome to Lumi\nThis is static Markdown.";
        let pkg_md = LumiPackage::new_md("TestMd", md);
        let bytes_md = pkg_md.to_bytes().unwrap();
        let unpacked_md = LumiPackage::from_bytes(&bytes_md).unwrap();
        assert_eq!(unpacked_md.name, "TestMd");
        assert_eq!(unpacked_md.index_md, Some(md.to_string()));

        // Corrupt JSON package bytes
        let corrupt_bytes = b"NOT_VALID_JSON_PACKAGE";
        assert!(LumiPackage::from_bytes(corrupt_bytes).is_err());
    }
}
