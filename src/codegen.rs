use std::io::Write;
use std::num::FpCategory;
use JsonValue;

extern crate itoa;

const QU: u8 = b'"';
const BS: u8 = b'\\';
const BB: u8 = b'b';
const TT: u8 = b't';
const NN: u8 = b'n';
const FF: u8 = b'f';
const RR: u8 = b'r';
const UU: u8 = b'u';
const __: u8 = 0;

// Look up table for characters that need escaping in a product string
static ESCAPED: [u8; 256] = [
// 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
  UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
  UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
  __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
  __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
  __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
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
        self.write(string[ .. start].as_bytes());

        for (index, ch) in string.bytes().enumerate().skip(start) {
            let escape = ESCAPED[ch as usize];
            if escape > 0 {
                self.write(string[start .. index].as_bytes());
                self.write(&[b'\\', escape]);
                start = index + 1;
            }
            if escape == b'u' {
                write!(self.get_writer(), "{:04x}", ch).unwrap();
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

    #[inline(always)]
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
            JsonValue::Null               => self.write(b"null"),
            JsonValue::Short(ref short)   => self.write_string(short.as_str()),
            JsonValue::String(ref string) => self.write_string(string),
            JsonValue::Number(ref number) => self.write_number(*number),
            JsonValue::Boolean(true)      => self.write(b"true"),
            JsonValue::Boolean(false)     => self.write(b"false"),
            JsonValue::Array(ref array)   => {
                self.write_char(b'[');
                let mut iter = array.iter();

                if let Some(item) = iter.next() {
                    self.indent();
                    self.new_line();
                    self.write_json(item);
                } else {
                    self.write_char(b']');
                    return;
                }

                for item in iter {
                    self.write_char(b',');
                    self.new_line();
                    self.write_json(item);
                }

                self.dedent();
                self.new_line();
                self.write_char(b']');
            },
            JsonValue::Object(ref object) => {
                self.write_char(b'{');
                let mut iter = object.iter();

                if let Some((key, value)) = iter.next() {
                    self.indent();
                    self.new_line();
                    self.write_string(key);
                    self.write_min(b": ", b':');
                    self.write_json(value);
                } else {
                    self.write_char(b'}');
                    return;
                }

                for (key, value) in iter {
                    self.write_char(b',');
                    self.new_line();
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
        // Original strings were unicode, numbers are all ASCII,
        // therefore this is safe.
        unsafe { String::from_utf8_unchecked(self.code) }
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
        unsafe { String::from_utf8_unchecked(self.code) }
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
