#![no_main]

use libfuzzer_sys::fuzz_target;
use lumi_protocol::LmpMessage;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);
    let _ = LmpMessage::read_from(&mut cursor);
});
