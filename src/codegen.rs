use std::io::Write;
use std::num::FpCategory;
use JsonValue;

extern crate itoa;

static ESCAPED: [u8; 256] = [
// 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
   0,   0,   0,   0,   0,   0,   0,   0,b'b',b't',b'n',  0,b'f',b'r',    0,   0, // 0
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // 1
   0,   0,b'"',   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // 2
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // 3
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // 4
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,b'\\',  0,   0,   0, // 5
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // 6
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // 7
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // 8
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // 9
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // A
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // B
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // C
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // D
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // E
   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // F
];

pub trait Generator {
    fn get_buffer(&mut self) -> &mut Vec<u8>;

    fn current_index(&mut self) -> usize {
        self.get_buffer().len()
    }

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) {
        self.get_buffer().extend_from_slice(slice)
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) {
        self.get_buffer().push(ch)
    }

    fn write_min(&mut self, slice: &[u8], minslice: &[u8]);

    fn new_line(&mut self) {}

    fn indent(&mut self) {}

    fn dedent(&mut self) {}

    fn write_string_complex(&mut self, string: &str, mut start: usize) {
        for (index, ch) in string.bytes().enumerate().skip(start) {
            let escape = ESCAPED[ch as usize];
            if escape > 0 {
                self.write(string[start .. index].as_bytes());
                self.write(&[b'\\', escape]);
                start = index + 1;
            }
        }

        self.write_char(b'"');
    }

    fn write_string(&mut self, string: &str) {
        self.write_char(b'"');

        for (index, ch) in string.bytes().enumerate() {
            if ESCAPED[ch as usize] > 0 {
                return self.write_string_complex(string, index)
            }
        }

        self.write(string.as_bytes());
        self.write_char(b'"');
    }

    fn write_number(&mut self, mut num: f64) {
        match num.classify() {
            FpCategory::Nan      |
            FpCategory::Infinite => {
                self.write(b"null");
                return;
            },
            FpCategory::Zero => {
                self.write(if num.is_sign_negative() { b"-0" } else { b"0" });
                return;
            },
            _ => {},
        }

        if num.is_sign_negative() {
            num = num.abs();
            self.write_char(b'-');
        }

        let fract = num.fract();

        if fract > 0.0 {
            if num < 1e-15 {
                write!(self.get_buffer(), "{:e}", num).unwrap();
            } else {
                write!(self.get_buffer(), "{}", num).unwrap();
            }
            return;
        }

        if num > 1e19 {
            write!(self.get_buffer(), "{:e}", num).unwrap();
            return;
        }

        itoa::write(self.get_buffer(), num as u64).unwrap();
    }

    fn write_json(&mut self, json: &JsonValue) {
        match *json {
            JsonValue::String(ref string) => self.write_string(string),
            JsonValue::Number(ref number) => self.write_number(*number),
            JsonValue::Boolean(true)      => self.write(b"true"),
            JsonValue::Boolean(false)     => self.write(b"false"),
            JsonValue::Null               => self.write(b"null"),
            JsonValue::Array(ref array)   => {
                self.write_char(b'[');
                self.indent();
                let mut first = true;
                for item in array {
                    if first {
                        first = false;
                        self.new_line();
                    } else {
                        self.write(b",");
                        self.new_line();
                    }
                    self.write_json(item);
                }
                self.dedent();
                self.new_line();
                self.write_char(b']');
            },
            JsonValue::Object(ref object) => {
                self.write_char(b'{');
                self.indent();
                let mut first = true;
                for (key, value) in object.iter() {
                    if first {
                        first = false;
                        self.new_line();
                    } else {
                        self.write(b",");
                        self.new_line();
                    }
                    self.write_string(key);
                    self.write_min(b": ", b":");
                    self.write_json(value);
                }
                self.dedent();
                self.new_line();
                self.write_char(b'}');
            }
        }
    }

    fn consume(self) -> String;
}

pub struct DumpGenerator {
    code: Vec<u8>,
}

impl DumpGenerator {
    pub fn new() -> Self {
        DumpGenerator {
            code: Vec::with_capacity(1024),
        }
    }
}

impl Generator for DumpGenerator {
    #[inline(always)]
    fn get_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], minslice: &[u8]) {
        self.code.extend_from_slice(minslice);
    }

    fn consume(self) -> String {
        String::from_utf8(self.code).unwrap()
    }
}

pub struct PrettyGenerator {
    code: Vec<u8>,
    dent: u16,
    spaces_per_indent: u16,
}

impl PrettyGenerator {
    pub fn new(spaces: u16) -> Self {
        PrettyGenerator {
            code: Vec::with_capacity(1024),
            dent: 0,
            spaces_per_indent: spaces
        }
    }
}

impl Generator for PrettyGenerator {
    #[inline(always)]
    fn get_buffer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: &[u8]) {
        self.code.extend_from_slice(slice);
    }

    fn new_line(&mut self) {
        self.code.push(b'\n');
        for _ in 0..(self.dent * self.spaces_per_indent) {
            self.code.push(b' ');
        }
    }

    fn indent(&mut self) {
        self.dent += 1;
    }

    fn dedent(&mut self) {
        self.dent -= 1;
    }

    fn consume(self) -> String {
        String::from_utf8(self.code).unwrap()
    }
}
