#![no_main]

extern crate libfuzzer_sys;

extern crate lavendeux_parser;
use lavendeux_parser::Lavendeux;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let mut parser = Lavendeux::new(Default::default());
        let _ = parser.parse(s);
    }
});
