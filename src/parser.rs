use std::char;
use std::str;
use std::str::Bytes;
use std::collections::BTreeMap;
use { JsonValue, JsonError, JsonResult };

#[derive(Debug)]
pub enum Token {
    Comma,
    Colon,
    BracketOn,
    BracketOff,
    BraceOn,
    BraceOff,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

impl Token {
    pub fn to_string(&self) -> String {
        match *self {
            Token::Comma          => ",",
            Token::Colon          => ":",
            Token::BracketOn      => "[",
            Token::BracketOff     => "]",
            Token::BraceOn        => "{",
            Token::BraceOff       => "}",
            Token::String(_)      => "[string]",
            Token::Number(_)      => "[number]",
            Token::Boolean(true)  => "true",
            Token::Boolean(false) => "false",
            Token::Null           => "null",
        }.into()
    }
}

macro_rules! sequence {
    ($tok:ident, $( $ch:pat ),*) => {
        $(
            match $tok.next_byte() {
                Some($ch) => {},
                Some(ch)  => return Err($tok.unexpected_character_error(ch)),
                None      => return Err(JsonError::UnexpectedEndOfJson)
            }
        )*
    }
}

macro_rules! read_num {
    ($tok:ident, $num:ident, $then:expr) => {
        while let Some(ch) = $tok.checked_next_byte() {
            match ch {
                b'0' ... b'9' => {
                    let $num = ch - b'0';
                    $then;
                },
                ch => {
                    $tok.left_over = Some(ch);
                    break;
                }
            }
        }
    }
}

struct Position {
    pub line: usize,
    pub column: usize,
}

struct Tokenizer<'a> {
    source: &'a str,
    byte_iter: Bytes<'a>,
    buffer: Vec<u8>,
    left_over: Option<u8>,
    current_index: usize,
    pub current_token_index: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source: source,
            byte_iter: source.bytes(),
            buffer: Vec::with_capacity(512),
            left_over: None,
            current_index: 0,
            current_token_index: 0,
        }
    }

    pub fn source_position_from_index(&self, index: usize) -> Position {
        let (bytes, _) = self.source.split_at(index-1);

        Position {
            line: bytes.lines().count(),
            column: bytes.lines().last().map(|line| {
                line.chars().count() + 1
            }).unwrap_or(1)
        }
    }

    fn unexpected_character_error(&self, byte: u8) -> JsonError {
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

        return self.left_over;
    }

    #[inline(always)]
    fn checked_next_byte(&mut self) -> Option<u8> {
        if self.left_over.is_some() {
            let byte = self.left_over;
            self.left_over = None;
            return byte;
        }

        self.next_byte()
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
            ch            => return Err(self.unexpected_character_error(ch)),
        })
    }

    fn read_codepoint(&mut self) -> JsonResult<()> {
        let codepoint = try!(self.read_char_as_hexnumber()) << 12
                      | try!(self.read_char_as_hexnumber()) << 8
                      | try!(self.read_char_as_hexnumber()) << 4
                      | try!(self.read_char_as_hexnumber());

        let ch = try!(
            char::from_u32(codepoint).ok_or(JsonError::FailedUtf8Parsing)
        );

        let mut buffer = String::new();
        buffer.push(ch);

        self.buffer.extend_from_slice(buffer.as_bytes());
        Ok(())
    }

    fn read_string(&mut self) -> JsonResult<String> {
        self.buffer.clear();

        loop {
            let ch = try!(self.expect_byte());
            match ch {
                b'"'  => break,
                b'\\' => {
                    let ch = try!(self.expect_byte());
                    let ch = match ch {
                        b'u'  => {
                            try!(self.read_codepoint());
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
                        _     => return Err(self.unexpected_character_error(ch))
                    };
                    self.buffer.push(ch);
                },
                _     => self.buffer.push(ch)
            }
        }

        // Since the original source is already valid UTF-8, and `\`
        // cannot occur in front of a codepoint > 127, this is safe.
        Ok( unsafe { str::from_utf8_unchecked(&self.buffer).into() } )
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
                let mut e = 0;
                let sign = match self.next_byte() {
                    Some(b'-') => -1,
                    Some(b'+') => 1,
                    byte => {
                        self.left_over = byte;
                        1
                    },
                };

                read_num!(self, digit, e = e * 10 + digit as i32);

                num *= 10f64.powi(e * sign);
            },
            byte => self.left_over = byte
        }

        Ok(if is_negative { -num } else { num })
    }

    fn next(&mut self) -> JsonResult<Token> {
        let ch = try!(self.checked_expect_byte());
        self.current_token_index = self.current_index;

        Ok(match ch {
            b',' => Token::Comma,
            b':' => Token::Colon,
            b'[' => Token::BracketOn,
            b']' => Token::BracketOff,
            b'{' => Token::BraceOn,
            b'}' => Token::BraceOff,
            b'"' => Token::String(try!(self.read_string())),
            b'0' ... b'9' => Token::Number(try!(self.read_number(ch, false))),
            b'-' => {
                let ch = try!(self.expect_byte());
                Token::Number(try!(self.read_number(ch, true)))
            }
            b't' => {
                sequence!(self, b'r', b'u', b'e');
                Token::Boolean(true)
            },
            b'f' => {
                sequence!(self, b'a', b'l', b's', b'e');
                Token::Boolean(false)
            },
            b'n' => {
                sequence!(self, b'u', b'l', b'l');
                Token::Null
            },
            // whitespace
            9 ... 13 | 32 | 133 | 160 => return self.next(),
            _ => return Err(self.unexpected_character_error(ch))
        })
    }
}

macro_rules! expect {
    ($parser:ident, $token:pat => $value:ident) => (
        match $parser.tokenizer.next() {
            Ok($token) => $value,
            Ok(token)  => return Err($parser.unexpected_token_error(token)),
            Err(error)         => return Err(error),
        }
    );
    ($parser:ident, $token:pat) => ({
        match $parser.tokenizer.next() {
            Ok($token) => {},
            Ok(token)  => return Err($parser.unexpected_token_error(token)),
            Err(error) => return Err(error),
        }
    })
}

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Parser {
            tokenizer: Tokenizer::new(source),
        }
    }

    fn unexpected_token_error(&self, token: Token) -> JsonError {
        let index = self.tokenizer.current_token_index;
        let pos = self.tokenizer.source_position_from_index(index);

        JsonError::UnexpectedToken {
            token: token.to_string(),
            line: pos.line,
            column: pos.column,
        }
    }

    #[must_use]
    fn ensure_end(&mut self) -> JsonResult<()> {
        match self.tokenizer.next() {
            Ok(token) => Err(self.unexpected_token_error(token)),
            Err(JsonError::UnexpectedEndOfJson) => Ok(()),
            Err(error)                          => Err(error)
        }
    }

    fn array(&mut self) -> JsonResult<JsonValue> {
        let mut array = Vec::with_capacity(20);

        match try!(self.tokenizer.next()) {
            Token::BracketOff => return Ok(JsonValue::Array(array)),
            token             => {
                array.push(try!(self.value_from(token)));
            }
        }

        loop {
            match try!(self.tokenizer.next()) {
                Token::Comma      => {
                    array.push(try!(self.value()));
                    continue
                },
                Token::BracketOff => break,
                token             => {
                    return Err(self.unexpected_token_error(token))
                }
            }
        }

        Ok(JsonValue::Array(array))
    }

    fn object(&mut self) -> JsonResult<JsonValue> {
        let mut object = BTreeMap::new();

        match try!(self.tokenizer.next()) {
            Token::BraceOff    => return Ok(JsonValue::Object(object)),
            Token::String(key) => {
                expect!(self, Token::Colon);
                object.insert(key, try!(self.value()));
            },
            token              => return Err(self.unexpected_token_error(token))
        }

        loop {
            match try!(self.tokenizer.next()) {
                Token::Comma => {
                    let key = expect!(self,
                        Token::String(key) => key
                    );
                    expect!(self, Token::Colon);
                    object.insert(key, try!(self.value()));
                    continue
                },
                Token::BraceOff => break,
                token           => return Err(self.unexpected_token_error(token))
            }
        }

        Ok(JsonValue::Object(object))
    }

    fn value_from(&mut self, token: Token) -> JsonResult<JsonValue> {
        Ok(match token {
            Token::String(value)    => JsonValue::String(value),
            Token::Number(value)    => JsonValue::Number(value),
            Token::Boolean(value)   => JsonValue::Boolean(value),
            Token::Null             => JsonValue::Null,
            Token::BracketOn        => return self.array(),
            Token::BraceOn          => return self.object(),
            _                       => {
                return Err(self.unexpected_token_error(token))
            }
        })
    }

    fn value(&mut self) -> JsonResult<JsonValue> {
        let token = try!(self.tokenizer.next());
        self.value_from(token)
    }
}

pub fn parse(source: &str) -> JsonResult<JsonValue> {
    let mut parser = Parser::new(source);

    let value = try!(parser.value());

    try!(parser.ensure_end());

    Ok(value)
}
