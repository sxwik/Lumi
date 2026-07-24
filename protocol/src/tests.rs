#[cfg(test)]
mod tests {
    use crate::{LmpMessage, LnsResolver, LumiPackage, LumiUri};
    use lumi_parser::parse;

    #[test]
    fn test_lns_resolution() {
        let resolver = LnsResolver::new();
        assert_eq!(resolver.resolve("docs.lumi").unwrap(), "127.0.0.1:9001");
        assert_eq!(resolver.resolve("search.lumi").unwrap(), "127.0.0.1:9001");
        assert!(resolver.resolve("nonexistent.invalid").is_err());
    }

    #[test]
    fn test_lmp_framing_and_malformed_packet_protection() {
        let msg = LmpMessage::new_request("lumi://docs.lumi", 42);
        let mut buf = Vec::new();
        msg.write_to(&mut buf).unwrap();

        let decoded = LmpMessage::read_from(&mut &buf[..]).unwrap();
        assert_eq!(decoded.stream_id, 42);
        assert_eq!(decoded.header.uri, "lumi://docs.lumi");

        // Verify invalid magic header detection
        let mut bad_buf = buf.clone();
        bad_buf[0] = b'X';
        assert!(LmpMessage::read_from(&mut &bad_buf[..]).is_err());
    }

    #[test]
    fn test_lpkg_bundle_cycle() {
        let lml = "page { title \"Test\" paragraph { text \"Hello\" } }";
        let pkg = LumiPackage::new("TestSite", lml);

        let bytes = pkg.to_bytes().unwrap();
        let unpacked = LumiPackage::from_bytes(&bytes).unwrap();

        assert_eq!(unpacked.name, "TestSite");
        assert_eq!(unpacked.index_lml, lml);

        let ast = parse(&unpacked.index_lml).unwrap();
        assert_eq!(ast.element_type, lumi_parser::ElementType::Page);
    }
}
