use std::char;
use std::f64;
use std::str;
use std::collections::BTreeMap;
use { JsonValue, JsonError, JsonResult };

const MAX_FLOAT_PRECISION: u64 = 576460752303423500;

struct Position {
    pub line: usize,
    pub column: usize,
}

struct Parser<'a> {
    source: &'a str,
    byte_ptr: *const u8,
    index: usize,
    length: usize,
}

macro_rules! next_byte {
    ($parser:ident || $alt:expr) => {
        if $parser.index < $parser.length {
            let ch = unsafe { *$parser.byte_ptr.offset($parser.index as isize) };
            $parser.index += 1;
            ch
        } else {
            $alt
        }
    };

    ($parser:ident) => {
        next_byte!($parser || return Err(JsonError::UnexpectedEndOfJson))
    }
}

macro_rules! sequence {
    ($parser:ident, $( $ch:pat ),*) => {
        $(
            match next_byte!($parser) {
                $ch => {},
                ch  => return $parser.unexpected_character(ch),
            }
        )*
    }
}

macro_rules! read_num {
    ($parser:ident, $num:ident, $then:expr) => {
        loop {
            let ch = next_byte!($parser || break);
            match ch {
                b'0' ... b'9' => {
                    let $num = ch - b'0';
                    $then;
                },
                _  => {
                    $parser.index -= 1;
                    break;
                }
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
                    match next_byte!($parser) {
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
        let mut ch = next_byte!($parser);

        consume_whitespace!($parser, ch);

        if ch != $byte {
            return $parser.unexpected_character(ch)
        }
    });

    {$parser:ident $(, $byte:pat => $then:expr )*} => ({
        let mut ch = next_byte!($parser);

        consume_whitespace!($parser, ch);

        match ch {
            $(
                $byte => $then,
            )*
            _ => return $parser.unexpected_character(ch)
        }

    })
}

const QU: u8 = 1; // quote
const BS: u8 = 2; // backslash
const CT: u8 = 4; // control character

static CHARCODES: [u8; 256] = [
// 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
  CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 0
  CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, CT, // 1
   0,  0, QU,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 2
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 3
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 4
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, BS,  0,  0,  0, // 5
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 6
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, CT, // 7
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 8
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 9
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // A
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // B
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // C
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // D
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // E
   0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // F
];

macro_rules! expect_string {
    ($parser:ident) => ({
        let result: String;// = unsafe { mem::uninitialized() };
        let start = $parser.index;

        loop {
            let ch = next_byte!($parser);
            if CHARCODES[ch as usize] == 0 {
                continue;
            }
            if ch == b'"' {
                result = (&$parser.source[start .. $parser.index - 1]).into();
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
        let mut digits = 0u8;

        let result: f64;

        // Cap on how many iterations we do while reading to u64
        // in order to avoid an overflow.
        loop {
            if digits == 18 {
                result = try!($parser.read_big_number(num));
                break;
            }

            digits += 1;

            let ch = next_byte!($parser || {
                result = num as f64;
                break;
            });

            match ch {
                b'0' ... b'9' => {
                    // Avoid multiplication with bitshifts and addition
                    num = (num << 1) + (num << 3) + (ch - b'0') as u64;
                },
                b'.' | b'e' | b'E' => {
                    $parser.index -= 1;
                    result = try!($parser.read_number_with_fraction(num, 0));
                    break;
                },
                _  => {
                    $parser.index -= 1;
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
        let mut ch = next_byte!($parser);

        consume_whitespace!($parser, ch);

        match ch {
            $(
                $byte => $then,
            )*
            b'[' => JsonValue::Array(try!($parser.read_array())),
            b'{' => JsonValue::Object(try!($parser.read_object())),
            b'"' => JsonValue::String(expect_string!($parser)),
            b'0' => {
                let num = try!($parser.read_number_with_fraction(0, 0));
                JsonValue::Number(num)
            },
            b'1' ... b'9' => {
                let num = expect_number!($parser, ch);
                JsonValue::Number(num)
            },
            b'-' => {
                let ch = next_byte!($parser);
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
            source: source,
            byte_ptr: source.as_ptr(),
            index: 0,
            length: source.len(),
        }
    }

    pub fn source_position_from_index(&self, index: usize) -> Position {
        let (bytes, _) = self.source.split_at(index-1);

        Position {
            line: bytes.lines().count(),
            column: bytes.lines().last().map_or(1, |line| {
                line.chars().count() + 1
            })
        }
    }

    fn unexpected_character<T: Sized>(&mut self, byte: u8) -> JsonResult<T> {
        let pos = self.source_position_from_index(self.index);

        let ch = if byte & 0x80 != 0 {
            let mut buf = [byte,0,0,0];
            let mut len = 0usize;

            if byte & 0xE0 == 0xCE {
                // 2 bytes, 11 bits
                len = 2;
                buf[1] = next_byte!(self);
            } else if byte & 0xF0 == 0xE0 {
                // 3 bytes, 16 bits
                len = 3;
                buf[1] = next_byte!(self);
                buf[2] = next_byte!(self);
            } else if byte & 0xF8 == 0xF0 {
                // 4 bytes, 21 bits
                len = 4;
                buf[1] = next_byte!(self);
                buf[2] = next_byte!(self);
                buf[3] = next_byte!(self);
            }

            let slice = try!(
                str::from_utf8(&buf[0..len])
                .map_err(|_| JsonError::FailedUtf8Parsing)
            );

            slice.chars().next().unwrap()
        } else {

            // codepoints < 128 are safe ASCII compatibles
            unsafe { char::from_u32_unchecked(byte as u32) }
        };

        Err(JsonError::UnexpectedCharacter {
            ch: ch,
            line: pos.line,
            column: pos.column,
        })
    }

    fn read_hexdec_digit(&mut self) -> JsonResult<u32> {
        let ch = next_byte!(self);
        Ok(match ch {
            b'0' ... b'9' => (ch - b'0'),
            b'a' ... b'f' => (ch + 10 - b'a'),
            b'A' ... b'F' => (ch + 10 - b'A'),
            ch            => return self.unexpected_character(ch),
        } as u32)
    }

    fn read_hexdec_codepoint(&mut self) -> JsonResult<u32> {
        Ok(
            try!(self.read_hexdec_digit()) << 12 |
            try!(self.read_hexdec_digit()) << 8  |
            try!(self.read_hexdec_digit()) << 4  |
            try!(self.read_hexdec_digit())
        )
    }

    fn read_codepoint(&mut self, buffer: &mut Vec<u8>) -> JsonResult<()> {
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
                    return Err(JsonError::FailedUtf8Parsing)
                }
            },
            0xE000 ... 0xFFFF => {},
            _ => return Err(JsonError::FailedUtf8Parsing)
        }

        match codepoint {
            0x0000 ... 0x007F => buffer.push(codepoint as u8),
            0x0080 ... 0x07FF => buffer.extend_from_slice(&[
                (((codepoint >> 6) as u8) & 0x1F) | 0xC0,
                ((codepoint        as u8) & 0x3F) | 0x80
            ]),
            0x0800 ... 0xFFFF => buffer.extend_from_slice(&[
                (((codepoint >> 12) as u8) & 0x0F) | 0xE0,
                (((codepoint >> 6)  as u8) & 0x3F) | 0x80,
                ((codepoint         as u8) & 0x3F) | 0x80
            ]),
            0x10000 ... 0x10FFFF => buffer.extend_from_slice(&[
                (((codepoint >> 18) as u8) & 0x07) | 0xF0,
                (((codepoint >> 12) as u8) & 0x3F) | 0x80,
                (((codepoint >> 6)  as u8) & 0x3F) | 0x80,
                ((codepoint         as u8) & 0x3F) | 0x80
            ]),
            _ => return Err(JsonError::FailedUtf8Parsing)
        }

        Ok(())
    }

    fn read_complex_string(&mut self, start: usize) -> JsonResult<String> {
        let mut buffer = Vec::new();
        let mut ch = b'\\';

        buffer.extend_from_slice(self.source[start .. self.index - 1].as_bytes());

        loop {
            if CHARCODES[ch as usize] == 0 {
                buffer.push(ch);
                ch = next_byte!(self);
                continue;
            }
            match ch {
                b'"'  => break,
                b'\\' => {
                    let escaped = next_byte!(self);
                    let escaped = match escaped {
                        b'u'  => {
                            try!(self.read_codepoint(&mut buffer));
                            ch = next_byte!(self);
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
                    buffer.push(escaped);
                },
                _ => return self.unexpected_character(ch)
            }
            ch = next_byte!(self);
        }

        // Since the original source is already valid UTF-8, and `\`
        // cannot occur in front of a codepoint > 127, this is safe.
        Ok(unsafe { String::from_utf8_unchecked(buffer) })
    }

    fn read_big_number(&mut self, num: u64) -> JsonResult<f64> {
        // Attempt to continue reading digits that would overflow
        // u64 into freshly converted f64

        let mut e = 0i32;
        loop {
            match next_byte!(self || break) {
                b'0' ... b'9' => e += 1,
                _  => {
                    self.index -= 1;
                    break;
                }
            }
        }

        self.read_number_with_fraction(num, e)
    }

    fn read_number_with_fraction(&mut self, mut num: u64, mut e: i32) -> JsonResult<f64> {
        if next_byte!(self || return Ok(make_float(num, e))) == b'.' {
            loop {
                let ch = next_byte!(self || break);

                match ch {
                    b'0' ... b'9' => {
                        if num < MAX_FLOAT_PRECISION {
                            num = num * 10 + (ch - b'0') as u64;
                            e -= 1;
                        }
                    },
                    _ => {
                        self.index -= 1;
                        break;
                    }
                }
            }
        } else {
            self.index -= 1;
        }

        match next_byte!(self || return Ok(make_float(num, e))) {
            b'e' | b'E' => {
                let sign = match next_byte!(self) {
                    b'-' => -1,
                    b'+' => 1,
                    _    => {
                        self.index -= 1;
                        1
                    },
                };

                let num  = make_float(num, e);

                let ch = next_byte!(self);
                let mut e = match ch {
                    b'0' ... b'9' => (ch - b'0') as i32,
                    _ => return self.unexpected_character(ch),
                };

                read_num!(self, digit, e = (e << 3) + (e << 1) + digit as i32);

                return Ok(num * exponent_to_power(e * sign));
            },
            _ => self.index -= 1
        }

        Ok(make_float(num, e))
    }

    fn read_object(&mut self) -> JsonResult<BTreeMap<String, JsonValue>> {
        let mut object = BTreeMap::new();

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

    fn read_array(&mut self) -> JsonResult<Vec<JsonValue>> {
        let first = expect_value!{ self, b']' => return Ok(Vec::new()) };

        let mut array = Vec::with_capacity(20);
        array.push(first);

        loop {
            expect!{ self,
                b']' => break,
                b',' => {
                    let value = expect_value!(self);
                    array.push(value);
                }
            };
        }

        Ok(array)
    }

    fn ensure_end(&mut self) -> JsonResult<()> {
        let mut ch = next_byte!(self || return Ok(()));
        loop {
            match ch {
                // whitespace
                9 ... 13 | 32 => {},
                _             => return self.unexpected_character(ch)
            }
            ch = next_byte!(self || return Ok(()));
        }
    }

    fn value(&mut self) -> JsonResult<JsonValue> {
        Ok(expect_value!(self))
    }
}

pub fn parse(source: &str) -> JsonResult<JsonValue> {
    let mut parser = Parser::new(source);

    let value = try!(parser.value());

    try!(parser.ensure_end());

    Ok(value)
}
