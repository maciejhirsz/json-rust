#[macro_use]
extern crate json;

use json::codegen::Generator;
use json::object::Object;
use json::JsonValue;
use std::io;

/// Custom generator that sort keys by name; based on DumpGenerator.
pub struct CustomGenerator {
    code: Vec<u8>,
}

impl CustomGenerator {
    pub fn new() -> Self {
        CustomGenerator {
            code: Vec::with_capacity(1024),
        }
    }

    pub fn consume(self) -> String {
        // Original strings were unicode, numbers are all ASCII,
        // therefore this is safe.
        unsafe { String::from_utf8_unchecked(self.code) }
    }
}

impl Generator for CustomGenerator {
    type T = Vec<u8>;
    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) -> io::Result<()> {
        self.code.push(min);
        Ok(())
    }
    #[inline(always)]
    fn write_object(&mut self, object: &Object) -> io::Result<()> {
        self.write_char(b'{')?;
        let mut entries: Vec<(&str, &JsonValue)> = Vec::new();
        for (k, v) in object.iter() {
            entries.push((k, v));
        }

        entries.sort_by(|(k1, _), (k2, _)| k1.partial_cmp(k2).unwrap());

        let mut iter = entries.iter();
        if let Some((key, value)) = iter.next() {
            self.indent();
            self.new_line()?;
            self.write_string(key)?;
            self.write_min(b": ", b':')?;
            self.write_json(value)?;
        } else {
            self.write_char(b'}')?;
            return Ok(());
        }

        for (key, value) in iter {
            self.write_char(b',')?;
            self.new_line()?;
            self.write_string(key)?;
            self.write_min(b": ", b':')?;
            self.write_json(value)?;
        }

        self.dedent();
        self.new_line()?;
        self.write_char(b'}')
    }
}

#[test]
fn object_keys_sorted() {
    let o = object! {
        c: null,
        b: null,
        a: null,
    };
    let mut gen = CustomGenerator::new();
    gen.write_json(&o).expect("Can't fail");
    let json = gen.consume();
    let dump = o.dump();
    assert_eq!(json, r#"{"a":null,"b":null,"c":null}"#);
    assert_eq!(dump, r#"{"c":null,"b":null,"a":null}"#);
    assert_ne!(json, dump);
}
