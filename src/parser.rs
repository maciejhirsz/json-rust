use std::{ str, slice, char, f64 };
use object::Object;
use { JsonValue, Error, Result };

const MAX_PRECISION: u64 = 576460752303423500;

struct Position {
    pub line: usize,
    pub column: usize,
}

struct Parser<'a> {
    buffer: Vec<u8>,
    source: &'a str,
    byte_ptr: *const u8,
    index: usize,
    length: usize,
}

macro_rules! expect_byte {
    ($parser:ident) => ({
        if $parser.is_eof() {
            return Err(Error::UnexpectedEndOfJson);
        }

        let ch = $parser.read_byte();
        $parser.bump();
        ch
    })
}

macro_rules! sequence {
    ($parser:ident, $( $ch:pat ),*) => {
        $(
            match expect_byte!($parser) {
                $ch => {},
                ch  => return $parser.unexpected_character(ch),
            }
        )*
    }
}

macro_rules! read_num {
    ($parser:ident, $num:ident, $then:expr) => {
        loop {
            if $parser.is_eof() { break; }
            let ch = $parser.read_byte();
            match ch {
                b'0' ... b'9' => {
                    $parser.bump();
                    let $num = ch - b'0';
                    $then;
                },
                _  => break
            }
        }
    }
}

macro_rules! consume_whitespace {
    ($parser:ident, $ch:ident) => {
        match $ch {
            // whitespace
            9 ... 13 | 32 => {
                loop {
                    match expect_byte!($parser) {
                        9 ... 13 | 32 => {},
                        ch            => { $ch = ch; break }
                    }
                }
            },
            _ => {}
        }
    }
}

macro_rules! expect {
    ($parser:ident, $byte:expr) => ({
        let mut ch = expect_byte!($parser);

        consume_whitespace!($parser, ch);

        if ch != $byte {
            return $parser.unexpected_character(ch)
        }
    });

    {$parser:ident $(, $byte:pat => $then:expr )*} => ({
        let mut ch = expect_byte!($parser);

        consume_whitespace!($parser, ch);

        match ch {
            $(
                $byte => $then,
            )*
            _ => return $parser.unexpected_character(ch)
        }

    })
}

const QU: bool = false;  // double quote       0x22
const BS: bool = false;  // backslash          0x5C
const CT: bool = false;  // control character  0x00 ... 0x1F
const __: bool = true;

// Look up table that marks which characters are allowed in their raw
// form in a string.
static ALLOWED: [bool; 256] = [
// 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
  CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
  CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
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

macro_rules! expect_string {
    ($parser:ident) => ({
        let result: &str;
        let start = $parser.index;

        loop {
            let ch = expect_byte!($parser);
            if ALLOWED[ch as usize] {
                continue;
            }
            if ch == b'"' {
                unsafe {
                    let ptr = $parser.byte_ptr.offset(start as isize);
                    let len = $parser.index - 1 - start;
                    result = str::from_utf8_unchecked(slice::from_raw_parts(ptr, len));
                }
                break;
            }
            if ch == b'\\' {
                result = try!($parser.read_complex_string(start));
                break;
            }

            return $parser.unexpected_character(ch);
        }

        result
    })
}

fn exponent_to_power(e: i32) -> f64 {
    static POWERS: [f64; 22] = [
          1e1,    1e2,    1e3,    1e4,    1e5,    1e6,    1e7,    1e8,
          1e9,   1e10,   1e11,   1e12,   1e13,   1e14,   1e15,   1e16,
         1e17,   1e18,   1e19,   1e20,   1e21,   1e22
    ];

    static NEG_POWERS: [f64; 22] = [
         1e-1,   1e-2,   1e-3,   1e-4,   1e-5,   1e-6,   1e-7,   1e-8,
         1e-9,  1e-10,  1e-11,  1e-12,  1e-13,  1e-14,  1e-15,  1e-16,
        1e-17,  1e-18,  1e-19,  1e-20,  1e-21,  1e-22
    ];

    let index = (e.abs() - 1) as usize;

    // index=0 is e=1
    if index < 22 {
        if e < 0 {
            NEG_POWERS[index]
        } else {
            POWERS[index]
        }
    } else {
        // powf is more accurate
        10f64.powf(e as f64)
    }
}

fn make_float(num: u64, e: i32) -> f64 {
    (num as f64) * exponent_to_power(e)
}

macro_rules! expect_number {
    ($parser:ident, $first:ident) => ({
        let mut num = ($first - b'0') as u64;

        let result: f64;

        // Cap on how many iterations we do while reading to u64
        // in order to avoid an overflow.
        loop {
            if num >= 576460752303423500 {
                result = try!($parser.read_big_number(num));
                break;
            }

            if $parser.is_eof() {
                result = num as f64;
                break;
            }

            let ch = $parser.read_byte();

            match ch {
                b'0' ... b'9' => {
                    $parser.bump();
                    // Avoid multiplication with bitshifts and addition
                    num = (num << 1) + (num << 3) + (ch - b'0') as u64;
                },
                b'.' | b'e' | b'E' => {
                    result = try!($parser.read_number_with_fraction(num, 0));
                    break;
                },
                _  => {
                    result = num as f64;
                    break;
                }
            }
        }

        result
    })
}

macro_rules! expect_value {
    {$parser:ident $(, $byte:pat => $then:expr )*} => ({
        let mut ch = expect_byte!($parser);

        consume_whitespace!($parser, ch);

        match ch {
            $(
                $byte => $then,
            )*
            b'[' => JsonValue::Array(try!($parser.read_array())),
            b'{' => JsonValue::Object(try!($parser.read_object())),
            b'"' => expect_string!($parser).into(),
            b'0' => {
                let num = try!($parser.read_number_with_fraction(0, 0));
                JsonValue::Number(num)
            },
            b'1' ... b'9' => {
                let num = expect_number!($parser, ch);
                JsonValue::Number(num)
            },
            b'-' => {
                let ch = expect_byte!($parser);
                let num = match ch {
                    b'0' => try!($parser.read_number_with_fraction(0, 0)),
                    b'1' ... b'9' => expect_number!($parser, ch),
                    _    => return $parser.unexpected_character(ch)
                };
                JsonValue::Number(-num)
            }
            b't' => {
                sequence!($parser, b'r', b'u', b'e');
                JsonValue::Boolean(true)
            },
            b'f' => {
                sequence!($parser, b'a', b'l', b's', b'e');
                JsonValue::Boolean(false)
            },
            b'n' => {
                sequence!($parser, b'u', b'l', b'l');
                JsonValue::Null
            },
            _ => return $parser.unexpected_character(ch)
        }
    })
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            buffer: Vec::with_capacity(30),
            source: source,
            byte_ptr: source.as_ptr(),
            index: 0,
            length: source.len(),
        }
    }

    #[inline(always)]
    fn is_eof(&mut self) -> bool {
        self.index == self.length
    }

    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        unsafe { *self.byte_ptr.offset(self.index as isize) }
    }

    #[inline(always)]
    fn bump(&mut self) {
        self.index += 1;
    }

    fn source_position_from_index(&self, index: usize) -> Position {
        let (bytes, _) = self.source.split_at(index-1);

        Position {
            line: bytes.lines().count(),
            column: bytes.lines().last().map_or(1, |line| {
                line.chars().count() + 1
            })
        }
    }

    fn unexpected_character<T: Sized>(&mut self, byte: u8) -> Result<T> {
        let pos = self.source_position_from_index(self.index);

        let ch = if byte & 0x80 != 0 {
            let mut buf = [byte,0,0,0];
            let mut len = 0usize;

            if byte & 0xE0 == 0xCE {
                // 2 bytes, 11 bits
                len = 2;
                buf[1] = expect_byte!(self);
            } else if byte & 0xF0 == 0xE0 {
                // 3 bytes, 16 bits
                len = 3;
                buf[1] = expect_byte!(self);
                buf[2] = expect_byte!(self);
            } else if byte & 0xF8 == 0xF0 {
                // 4 bytes, 21 bits
                len = 4;
                buf[1] = expect_byte!(self);
                buf[2] = expect_byte!(self);
                buf[3] = expect_byte!(self);
            }

            let slice = try!(
                str::from_utf8(&buf[0..len])
                .map_err(|_| Error::FailedUtf8Parsing)
            );

            slice.chars().next().unwrap()
        } else {

            // codepoints < 128 are safe ASCII compatibles
            unsafe { char::from_u32_unchecked(byte as u32) }
        };

        Err(Error::UnexpectedCharacter {
            ch: ch,
            line: pos.line,
            column: pos.column,
        })
    }

    fn read_hexdec_digit(&mut self) -> Result<u32> {
        let ch = expect_byte!(self);
        Ok(match ch {
            b'0' ... b'9' => (ch - b'0'),
            b'a' ... b'f' => (ch + 10 - b'a'),
            b'A' ... b'F' => (ch + 10 - b'A'),
            ch            => return self.unexpected_character(ch),
        } as u32)
    }

    fn read_hexdec_codepoint(&mut self) -> Result<u32> {
        Ok(
            try!(self.read_hexdec_digit()) << 12 |
            try!(self.read_hexdec_digit()) << 8  |
            try!(self.read_hexdec_digit()) << 4  |
            try!(self.read_hexdec_digit())
        )
    }

    fn read_codepoint(&mut self) -> Result<()> {
        let mut codepoint = try!(self.read_hexdec_codepoint());

        match codepoint {
            0x0000 ... 0xD7FF => {},
            0xD800 ... 0xDBFF => {
                codepoint -= 0xD800;
                codepoint <<= 10;

                sequence!(self, b'\\', b'u');

                let lower = try!(self.read_hexdec_codepoint());

                if let 0xDC00 ... 0xDFFF = lower {
                    codepoint = (codepoint | lower - 0xDC00) + 0x010000;
                } else {
                    return Err(Error::FailedUtf8Parsing)
                }
            },
            0xE000 ... 0xFFFF => {},
            _ => return Err(Error::FailedUtf8Parsing)
        }

        match codepoint {
            0x0000 ... 0x007F => self.buffer.push(codepoint as u8),
            0x0080 ... 0x07FF => self.buffer.extend_from_slice(&[
                (((codepoint >> 6) as u8) & 0x1F) | 0xC0,
                ((codepoint        as u8) & 0x3F) | 0x80
            ]),
            0x0800 ... 0xFFFF => self.buffer.extend_from_slice(&[
                (((codepoint >> 12) as u8) & 0x0F) | 0xE0,
                (((codepoint >> 6)  as u8) & 0x3F) | 0x80,
                ((codepoint         as u8) & 0x3F) | 0x80
            ]),
            0x10000 ... 0x10FFFF => self.buffer.extend_from_slice(&[
                (((codepoint >> 18) as u8) & 0x07) | 0xF0,
                (((codepoint >> 12) as u8) & 0x3F) | 0x80,
                (((codepoint >> 6)  as u8) & 0x3F) | 0x80,
                ((codepoint         as u8) & 0x3F) | 0x80
            ]),
            _ => return Err(Error::FailedUtf8Parsing)
        }

        Ok(())
    }

    fn read_complex_string<'b>(&mut self, start: usize) -> Result<&'b str> {
        // let mut buffer = Vec::new();
        self.buffer.clear();
        let mut ch = b'\\';

        self.buffer.extend_from_slice(self.source[start .. self.index - 1].as_bytes());

        loop {
            if ALLOWED[ch as usize] {
                self.buffer.push(ch);
                ch = expect_byte!(self);
                continue;
            }
            match ch {
                b'"'  => break,
                b'\\' => {
                    let escaped = expect_byte!(self);
                    let escaped = match escaped {
                        b'u'  => {
                            try!(self.read_codepoint());
                            ch = expect_byte!(self);
                            continue;
                        },
                        b'"'  |
                        b'\\' |
                        b'/'  => escaped,
                        b'b'  => 0x8,
                        b'f'  => 0xC,
                        b't'  => b'\t',
                        b'r'  => b'\r',
                        b'n'  => b'\n',
                        _     => return self.unexpected_character(escaped)
                    };
                    self.buffer.push(escaped);
                },
                _ => return self.unexpected_character(ch)
            }
            ch = expect_byte!(self);
        }

        // Since the original source is already valid UTF-8, and `\`
        // cannot occur in front of a codepoint > 127, this is safe.
        Ok(unsafe {
            str::from_utf8_unchecked(
                // Construct the slice from parts to satisfy the borrow checker
                slice::from_raw_parts(self.buffer.as_ptr(), self.buffer.len())
            )
        })
    }

    fn read_big_number(&mut self, num: u64) -> Result<f64> {
        // Attempt to continue reading digits that would overflow
        // u64 into freshly converted f64

        let mut e = 0i32;
        loop {
            if self.is_eof() {
                return Ok(make_float(num, e));
            }
            match self.read_byte() {
                b'0' ... b'9' => {
                    self.bump();
                    e += 1;
                },
                _  => break
            }
        }

        self.read_number_with_fraction(num, e)
    }

    fn read_number_with_fraction(&mut self, mut num: u64, mut e: i32) -> Result<f64> {
        if self.is_eof() {
            return Ok(make_float(num, e));
        }

        let mut ch = self.read_byte();

        if ch == b'.' {
            self.bump();

            loop {
                if self.is_eof() {
                    return Ok(make_float(num, e));
                }
                ch = self.read_byte();

                match ch {
                    b'0' ... b'9' => {
                        self.bump();
                        if num < MAX_PRECISION {
                            num = (num << 3) + (num << 1) + (ch - b'0') as u64;
                            e -= 1;
                        }
                    },
                    _ => break
                }
            }
        }

        if ch == b'e' || ch == b'E' {
            self.bump();
            ch = expect_byte!(self);
            let sign = match ch {
                b'-' => {
                    ch = expect_byte!(self);
                    -1
                },
                b'+' => {
                    ch = expect_byte!(self);
                    1
                },
                _    => 1
            };

            let num = make_float(num, e);

            let mut e = match ch {
                b'0' ... b'9' => (ch - b'0') as i32,
                _ => return self.unexpected_character(ch),
            };

            read_num!(self, digit, e = (e << 3) + (e << 1) + digit as i32);

            return Ok(num * exponent_to_power(e * sign));
        }

        Ok(make_float(num, e))
    }

    fn read_object(&mut self) -> Result<Object> {
        let mut object = Object::with_capacity(3);

        let key = expect!{ self,
            b'}'  => return Ok(object),
            b'\"' => expect_string!(self)
        };

        expect!(self, b':');

        object.insert(key, expect_value!(self));

        loop {
            let key = expect!{ self,
                b'}' => break,
                b',' => {
                    expect!(self, b'"');
                    expect_string!(self)
                }
            };

            expect!(self, b':');

            object.insert(key, expect_value!(self));
        }

        Ok(object)
    }

    fn read_array(&mut self) -> Result<Vec<JsonValue>> {
        let first = expect_value!{ self, b']' => return Ok(Vec::new()) };

        let mut array = Vec::with_capacity(2);
        array.push(first);

        loop {
            expect!{ self,
                b']' => break,
                b',' => {
                    array.push(expect_value!(self));
                }
            };
        }

        Ok(array)
    }

    fn ensure_end(&mut self) -> Result<()> {
        while !self.is_eof() {
            match self.read_byte() {
                9 ... 13 | 32 => self.bump(),
                ch            => {
                    self.bump();
                    return self.unexpected_character(ch);
                }
            }
        }

        Ok(())
    }

    fn value(&mut self) -> Result<JsonValue> {
        Ok(expect_value!(self))
    }
}

pub fn parse(source: &str) -> Result<JsonValue> {
    let mut parser = Parser::new(source);

    let value = try!(parser.value());

    try!(parser.ensure_end());

    Ok(value)
}
