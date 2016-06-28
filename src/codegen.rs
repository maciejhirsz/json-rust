use std::io::Write;
use std::num::FpCategory;
use JsonValue;

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

    fn write_string(&mut self, string: &str) {
        self.write_char(b'"');

        for ch in string.bytes() {
            match ch {
                b'\\' | b'"' => {
                    self.write_char(b'\\');
                    self.write_char(ch);
                },
                b'\n' => self.write(b"\\n"),
                b'\r' => self.write(b"\\r"),
                b'\t' => self.write(b"\\t"),
                0xC   => self.write(b"\\f"),
                0x8   => self.write(b"\\b"),
                _     => self.write_char(ch)
            }
        }

        self.write_char(b'"');
    }

    fn write_digits_from_u64(&mut self, mut num: u64) {
        let digit = (num % 10) as u8;
        if num > 9 {
            num /= 10;
            self.write_digits_from_u64(num);
        }
        self.write_char(digit + b'0');
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

        if num > 1e19 || num < 1e-15 {
            write!(self.get_buffer(), "{:e}", num).unwrap();
            return;
        }

        self.write_digits_from_u64(num as u64);
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
