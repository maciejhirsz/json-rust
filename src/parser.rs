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
                Some(ch)  => return Err($parser.unexpected_character(ch)),
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
                    let consume = try!($parser.expect_byte());
                    match consume {
                        9 ... 13 | 32 => continue,
                        _             => {
                            $ch = consume;
                            break
                        }
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
            _ => return Err($parser.unexpected_character(ch))
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
            _ => return Err($parser.unexpected_character(ch))
        }

    })
}

macro_rules! expect_string {
    ($parser:ident) => ({
        $parser.buffer.clear();

        loop {
            let ch = try!($parser.expect_byte());
            match ch {
                b'"'  => break,
                b'\\' => {
                    let ch = try!($parser.expect_byte());
                    let ch = match ch {
                        b'u'  => {
                            try!($parser.read_codepoint());
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
                        _     => return Err($parser.unexpected_character(ch))
                    };
                    $parser.buffer.push(ch);
                },
                _ => $parser.buffer.push(ch)
            }
        }

        // Since the original source is already valid UTF-8, and `\`
        // cannot occur in front of a codepoint > 127, this is safe.
        unsafe { str::from_utf8_unchecked(&$parser.buffer).into() }
    })
}

macro_rules! expect_value {
    {$parser:ident $( $byte:pat => $then:expr ),*} => ({
        let mut ch = try!($parser.checked_expect_byte());

        consume_whitespace!($parser, ch);

        match ch {
            $(
                $byte => $then,
            )*
            b'[' => JsonValue::Array(try!($parser.read_array())),
            b'{' => JsonValue::Object(try!($parser.read_object())),
            b'"' => JsonValue::String(expect_string!($parser)),
            b'0' => {
                let num = try!($parser.read_number_with_fraction(0.0, false));
                JsonValue::Number(num)
            },
            b'1' ... b'9' => {
                let num = try!($parser.read_number(ch, false));
                JsonValue::Number(num)
            },
            b'-' => {
                let ch = try!($parser.expect_byte());
                let num = match ch {
                    b'0' => try!($parser.read_number_with_fraction(0.0, true)),
                    b'1' ... b'9' => try!($parser.read_number(ch, true)),
                    _    => return Err($parser.unexpected_character(ch))
                };
                JsonValue::Number(num)
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
            _ => return Err($parser.unexpected_character(ch))
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
    buffer: Vec<u8>,
    left_over: Option<u8>,
    current_index: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            source: source,
            byte_iter: source.bytes(),
            buffer: Vec::with_capacity(512),
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

    fn unexpected_character(&self, byte: u8) -> JsonError {
        let pos = self.source_position_from_index(self.current_index);
        let ch = char::from_u32(byte as u32).unwrap_or('?');

        JsonError::UnexpectedCharacter {
            ch: ch,
            line: pos.line,
            column: pos.column,
        }
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

    fn read_char_as_hexnumber(&mut self) -> JsonResult<u32> {
        let ch = try!(self.expect_byte());
        Ok(match ch {
            b'0' ... b'9' => (ch - b'0') as u32,
            b'a' ... b'f' => (ch + 10 - b'a') as u32,
            b'A' ... b'F' => (ch + 10 - b'A') as u32,
            ch            => return Err(self.unexpected_character(ch)),
        })
    }

    fn read_codepoint(&mut self) -> JsonResult<()> {
        let mut codepoint = try!(self.read_char_as_hexnumber()) << 12
                      | try!(self.read_char_as_hexnumber()) << 8
                      | try!(self.read_char_as_hexnumber()) << 4
                      | try!(self.read_char_as_hexnumber());

        match codepoint {
            0xD800 ... 0xDFFF => {
                codepoint -= 0xD800;
                codepoint <<= 10;

                sequence!(self, b'\\', b'u');

                let lower = try!(self.read_char_as_hexnumber()) << 12
                          | try!(self.read_char_as_hexnumber()) << 8
                          | try!(self.read_char_as_hexnumber()) << 4
                          | try!(self.read_char_as_hexnumber());

                if let 0xDC00 ... 0xDFFF = lower {
                    codepoint |= lower - 0xDC00;
                    codepoint += 0x010000;
                } else {
                    return Err(JsonError::FailedUtf8Parsing)
                }
            },
            _ => {}
        }

        match codepoint {
            0x0000 ... 0x007F => self.buffer.push(codepoint as u8),
            0x0080 ... 0x07FF => {
                self.buffer.push((((codepoint >> 6) as u8) & 0x1F) | 0xC0);
                self.buffer.push(((codepoint as u8) & 0x3F) | 0x80);
            },
            0x0800 ... 0xFFFF => {
                self.buffer.push((((codepoint >> 12) as u8) & 0x0F) | 0xE0);
                self.buffer.push((((codepoint >> 6) as u8) & 0x3F) | 0x80);
                self.buffer.push(((codepoint as u8) & 0x3F) | 0x80);
            },
            0x10000 ... 0x10FFFF => {
                self.buffer.push((((codepoint >> 18) as u8) & 0x07) | 0xF0);
                self.buffer.push((((codepoint >> 12) as u8) & 0x3F) | 0x80);
                self.buffer.push((((codepoint >> 6) as u8) & 0x3F) | 0x80);
                self.buffer.push(((codepoint as u8) & 0x3F) | 0x80);
            },
            _ => return Err(JsonError::FailedUtf8Parsing)
        }

        Ok(())
    }

    fn read_number(&mut self, first: u8, is_negative: bool) -> JsonResult<f64> {
        let mut num = (first - b'0') as u64;
        let mut digits = 0u8;

        // Cap on how many iterations we do while reading to u64
        // in order to avoid an overflow.
        while digits < 18 {
            digits += 1;

            if let Some(ch) = self.next_byte() {
                match ch {
                    b'0' ... b'9' => {
                        num = num * 10 + (ch - b'0') as u64;
                    },
                    b'.' | b'e' | b'E' => {
                        self.left_over = Some(ch);
                        break;
                    }
                    ch => {
                        self.left_over = Some(ch);
                        return Ok(
                            if is_negative { -(num as f64) } else { num as f64 }
                        );
                    }
                }
            } else {
                return Ok(
                    if is_negative { -(num as f64) } else { num as f64 }
                );
            }
        }

        let mut num = num as f64;

        // Attempt to continue reading digits that would overflow
        // u64 into freshly converted f64
        read_num!(self, digit, num = num * 10.0 + digit as f64);

        self.read_number_with_fraction(num, is_negative)
    }

    fn read_number_with_fraction(&mut self, mut num: f64, is_negative: bool)
    -> JsonResult<f64> {
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
                    _ => return Err(self.unexpected_character(ch)),
                };

                read_num!(self, digit, e = e * 10 + digit as i32);

                num *= 10f64.powi(e * sign);
            },
            byte => self.left_over = byte
        }

        Ok(if is_negative { -num } else { num })
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
        let mut array = Vec::with_capacity(20);

        let first = expect_value!{ self
            b']' => return Ok(array)
        };

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
        let ch = self.checked_next_byte();

        if let Some(ch) = ch {
            match ch {
                // whitespace
                9 ... 13 | 32 => while let Some(ch) = self.next_byte() {
                    match ch {
                        9 ... 13 | 32 => {},
                        _ => return Err(self.unexpected_character(ch))
                    }
                },
                _ => return Err(self.unexpected_character(ch))
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
