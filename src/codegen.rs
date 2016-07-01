use std::io::Write;
use std::num::FpCategory;
use JsonValue;

extern crate itoa;

const QU: u8 = b'"';
const BS: u8 = b'\\';
const  B: u8 = b'b';
const  T: u8 = b't';
const  N: u8 = b'n';
const  F: u8 = b'f';
const  R: u8 = b'r';
const  U: u8 = b'u';

static ESCAPED: [u8; 256] = [
// 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
   U,  U,  U,  U,  U,  U,  U,  U,  B,  T,  N,  U,  F,  R,  U,  U, // 0
   U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U, // 1
   0,  0, QU,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 2
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 3
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 4
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, BS,  0,  0,  0, // 5
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 6
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  U, // 7
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 8
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 9
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // A
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // B
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // C
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // D
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // E
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // F
];

pub trait Generator {
    type T: Write;

    fn get_writer(&mut self) -> &mut Self::T;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) {
        self.get_writer().write_all(slice).unwrap();
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) {
        self.get_writer().write_all(&[ch]).unwrap();
    }

    fn write_min(&mut self, slice: &[u8], min: u8);

    fn new_line(&mut self) {}

    fn indent(&mut self) {}

    fn dedent(&mut self) {}

    #[inline(never)]
    fn write_string_complex(&mut self, string: &str, mut start: usize) {
        for (index, ch) in string.bytes().enumerate().skip(start) {
            let escape = ESCAPED[ch as usize];
            if escape > 0 {
                self.write(string[start .. index].as_bytes());
                self.write(&[b'\\', escape]);
                start = index + 1;
            }
        }
        self.write(string[start ..].as_bytes());

        self.write_char(b'"');
    }

    #[inline(always)]
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

    fn write_number(&mut self, num: f64) {
        match num.classify() {
            FpCategory::Normal    |
            FpCategory::Subnormal => {
                if num.fract() == 0.0 && num.abs() < 1e19 {
                    itoa::write(self.get_writer(), num as i64).unwrap();
                } else {
                    let abs = num.abs();
                    if abs < 1e-15 || abs > 1e19 {
                        write!(self.get_writer(), "{:e}", num).unwrap();
                    } else {
                        write!(self.get_writer(), "{}", num).unwrap();
                    }
                }
            },
            FpCategory::Zero => {
                if num.is_sign_negative() {
                    self.write(b"-0");
                } else {
                    self.write_char(b'0');
                }
            },
            FpCategory::Nan      |
            FpCategory::Infinite => {
                self.write(b"null");
            }
        }
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
                        self.write_char(b',');
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
                        self.write_char(b',');
                        self.new_line();
                    }
                    self.write_string(key);
                    self.write_min(b": ", b':');
                    self.write_json(value);
                }
                self.dedent();
                self.new_line();
                self.write_char(b'}');
            }
        }
    }
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

    pub fn consume(self) -> String {
        String::from_utf8(self.code).unwrap()
    }
}

impl Generator for DumpGenerator {
    type T = Vec<u8>;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) {
        self.code.extend_from_slice(slice)
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) {
        self.code.push(ch)
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) {
        self.code.push(min);
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

    pub fn consume(self) -> String {
        String::from_utf8(self.code).unwrap()
    }
}

impl Generator for PrettyGenerator {
    type T = Vec<u8>;

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) {
        self.code.extend_from_slice(slice)
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) {
        self.code.push(ch)
    }

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.code
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) {
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
}

pub struct WriterGenerator<'a, W: 'a + Write> {
    writer: &'a mut W
}

impl<'a, W> WriterGenerator<'a, W> where W: 'a + Write {
    pub fn new(writer: &'a mut W) -> Self {
        WriterGenerator {
            writer: writer
        }
    }
}

impl<'a, W> Generator for WriterGenerator<'a, W> where W: Write {
    type T = W;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut W {
        &mut self.writer
    }

    #[inline(always)]
    fn write_min(&mut self, _: &[u8], min: u8) {
        self.writer.write_all(&[min]).unwrap();
    }
}
