use std::char;
use std::str;
use std::str::Bytes;
use std::collections::BTreeMap;
use { JsonValue, JsonError, JsonResult };

macro_rules! sequence {
    ($parser:ident, $( $ch:pat ),*) => {
        $(
            match $parser.next_byte() {
                Some($ch) => {},
                Some(ch)  => return $parser.unexpected_character(ch),
                None      => return Err(JsonError::UnexpectedEndOfJson)
            }
        )*
    }
}

macro_rules! read_num {
    ($parser:ident, $num:ident, $then:expr) => {
        while let Some(ch) = $parser.checked_next_byte() {
            match ch {
                b'0' ... b'9' => {
                    let $num = ch - b'0';
                    $then;
                },
                ch => {
                    $parser.left_over = Some(ch);
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
                    match try!($parser.expect_byte()) {
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
    ($parser:ident, $byte:pat) => ({
        let mut ch = try!($parser.expect_byte());

        consume_whitespace!($parser, ch);

        match ch {
            $byte         => {},
            _ => return $parser.unexpected_character(ch)
        }
    })
}

macro_rules! expect_one_of {
    {$parser:ident $(, $byte:pat => $then:expr )*} => ({
        let mut ch = try!($parser.checked_expect_byte());

        consume_whitespace!($parser, ch);

        match ch {
            $(
                $byte => $then,
            )*
            _ => return $parser.unexpected_character(ch)
        }

    })
}

macro_rules! expect_string {
    ($parser:ident) => ({
        let result: String;// = unsafe { mem::uninitialized() };
        let start = $parser.current_index;

        loop {
            let mut ch = try!($parser.expect_byte());
            if ch == b'"' {
                result = (&$parser.source[start .. $parser.current_index-1]).into();
                break;
            };
            if ch == b'\\' {
                result = expect_complex_string!($parser, start, ch);
                break;
            }
        }

        result
    })
}

macro_rules! expect_complex_string {
    ($parser:ident, $start:ident, $ch:ident) => ({
        // $parser.buffer.clear();
        let mut buffer = Vec::new();
        buffer.extend_from_slice($parser.source[$start .. $parser.current_index-1].as_bytes());

        loop {
            match $ch {
                b'"'  => break,
                b'\\' => {
                    let ch = try!($parser.expect_byte());
                    let ch = match ch {
                        b'u'  => {
                            try!($parser.read_codepoint(&mut buffer));
                            $ch = try!($parser.expect_byte());
                            continue;
                        },
                        b'"'  |
                        b'\\' |
                        b'/'  => ch,
                        b'b'  => 0x8,
                        b'f'  => 0xC,
                        b't'  => b'\t',
                        b'r'  => b'\r',
                        b'n'  => b'\n',
                        _     => return $parser.unexpected_character(ch)
                    };
                    buffer.push(ch);
                },
                _ => buffer.push($ch)
            }
            $ch = try!($parser.expect_byte());
        }

        // Since the original source is already valid UTF-8, and `\`
        // cannot occur in front of a codepoint > 127, this is safe.
        unsafe { String::from_utf8_unchecked(buffer) }
    })
}

macro_rules! expect_value {
    {$parser:ident $(, $byte:pat => $then:expr )*} => ({
        let mut ch = try!($parser.expect_byte());

        consume_whitespace!($parser, ch);

        match ch {
            $(
                $byte => $then,
            )*
            b'[' => JsonValue::Array(try!($parser.read_array())),
            b'{' => JsonValue::Object(try!($parser.read_object())),
            b'"' => JsonValue::String(expect_string!($parser)),
            b'0' => {
                let num = try!($parser.read_number_with_fraction(0.0));
                JsonValue::Number(num)
            },
            b'1' ... b'9' => {
                let num = try!($parser.read_number(ch));
                JsonValue::Number(num)
            },
            b'-' => {
                let ch = try!($parser.expect_byte());
                let num = match ch {
                    b'0' => try!($parser.read_number_with_fraction(0.0)),
                    b'1' ... b'9' => try!($parser.read_number(ch)),
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

struct Position {
    pub line: usize,
    pub column: usize,
}

struct Parser<'a> {
    source: &'a str,
    byte_iter: Bytes<'a>,
    left_over: Option<u8>,
    current_index: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            source: source,
            byte_iter: source.bytes(),
            left_over: None,
            current_index: 0,
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
        let pos = self.source_position_from_index(self.current_index);

        let ch = if byte & 0x80 != 0 {
            let mut buf = [byte,0,0,0];
            let mut len = 0usize;

            if byte & 0xE0 == 0xCE {
                // 2 bytes, 11 bits
                len = 2;
                buf[1] = try!(self.expect_byte());
            } else if byte & 0xF0 == 0xE0 {
                // 3 bytes, 16 bits
                len = 3;
                buf[1] = try!(self.expect_byte());
                buf[2] = try!(self.expect_byte());
            } else if byte & 0xF8 == 0xF0 {
                // 4 bytes, 21 bits
                len = 4;
                buf[1] = try!(self.expect_byte());
                buf[2] = try!(self.expect_byte());
                buf[3] = try!(self.expect_byte());
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

    #[inline(always)]
    fn next_byte(&mut self) -> Option<u8> {
        self.current_index += 1;
        self.byte_iter.next()
    }

    #[inline(always)]
    fn peek_byte(&mut self) -> Option<u8> {
        if self.left_over.is_none() {
            self.left_over = self.next_byte();
        }

        self.left_over
    }

    #[inline(always)]
    fn checked_next_byte(&mut self) -> Option<u8> {
        self.left_over.take().or_else(|| self.next_byte())
    }

    #[inline(always)]
    fn expect_byte(&mut self) -> JsonResult<u8> {
        self.next_byte().ok_or(JsonError::UnexpectedEndOfJson)
    }

    #[inline(always)]
    fn checked_expect_byte(&mut self) -> JsonResult<u8> {
        self.checked_next_byte().ok_or(JsonError::UnexpectedEndOfJson)
    }

    fn read_hexdec_digit(&mut self) -> JsonResult<u32> {
        let ch = try!(self.expect_byte());
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

    fn read_number(&mut self, first: u8) -> JsonResult<f64> {
        let mut num = (first - b'0') as u64;
        let mut digits = 0u8;

        // Cap on how many iterations we do while reading to u64
        // in order to avoid an overflow.
        while digits < 18 {
            digits += 1;

            if let Some(ch) = self.next_byte() {
                match ch {
                    b'0' ... b'9' => {
                        // Avoid multiplication with bitshifts and addition
                        num = (num << 1) + (num << 3) + (ch - b'0') as u64;
                    },
                    b'.' | b'e' | b'E' => {
                        self.left_over = Some(ch);
                        break;
                    },
                    ch => {
                        self.left_over = Some(ch);
                        return Ok(num as f64);
                    }
                }
            } else {
                return Ok(num as f64);
            }
        }

        let mut num = num as f64;

        // Attempt to continue reading digits that would overflow
        // u64 into freshly converted f64
        read_num!(self, digit, num = num * 10.0 + digit as f64);

        self.read_number_with_fraction(num)
    }

    fn read_number_with_fraction(&mut self, mut num: f64) -> JsonResult<f64> {
        if let Some(b'.') = self.peek_byte() {
            self.left_over = None;
            let mut precision = -1;

            read_num!(self, digit, {
                num += (digit as f64) * 10_f64.powi(precision);
                precision -= 1;
            });
        }

        match self.checked_next_byte() {
            Some(b'e') | Some(b'E') => {
                let sign = match self.next_byte() {
                    Some(b'-') => -1,
                    Some(b'+') => 1,
                    byte => {
                        self.left_over = byte;
                        1
                    },
                };

                let ch = try!(self.checked_expect_byte());
                let mut e = match ch {
                    b'0' ... b'9' => (ch - b'0') as i32,
                    _ => return self.unexpected_character(ch),
                };

                read_num!(self, digit, e = (e << 1) + (e << 3) + digit as i32);

                num *= 10f64.powi(e * sign);
            },
            byte => self.left_over = byte
        }

        Ok(num)
    }

    fn read_object(&mut self) -> JsonResult<BTreeMap<String, JsonValue>> {
        let mut object = BTreeMap::new();

        let key = expect_one_of!{ self,
            b'}'  => return Ok(object),
            b'\"' => expect_string!(self)
        };

        expect!(self, b':');

        object.insert(key, expect_value!(self));

        loop {
            let key = expect_one_of!{ self,
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
            expect_one_of!{ self,
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
        let mut next = self.checked_next_byte();

        while let Some(ch) = next {
            match ch {
                // whitespace
                9 ... 13 | 32 => next = self.next_byte(),
                _             => return self.unexpected_character(ch)
            }
        }

        Ok(())
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
