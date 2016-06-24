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

macro_rules! expect_char {
    ($tok:ident, $( $ch:pat ),*) => {
        $(
            match $tok.source.next() {
                Some($ch) => {},
                Some(ch)  => return Err(JsonError::unexpected_character(ch)),
                None      => return Err(JsonError::UnexpectedEndOfJson)
            }
        )*
    }
}

macro_rules! read_num {
    ($tok:ident, $num:ident, $then:expr) => {
        while let Some(ch) = $tok.next_byte() {
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

struct Tokenizer<'a> {
    source: Bytes<'a>,
    buffer: Vec<u8>,
    left_over: Option<u8>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Tokenizer {
            source: source.bytes(),
            buffer: Vec::with_capacity(512),
            left_over: None,
        }
    }

    fn peek_byte(&mut self) -> Option<u8> {
        if self.left_over.is_none() {
            self.left_over = self.source.next();
        }

        return self.left_over;
    }

    fn next_byte(&mut self) -> Option<u8> {
        if self.left_over.is_some() {
            let byte = self.left_over;
            self.left_over = None;
            return byte;
        }

        self.source.next()
    }

    #[inline(always)]
    fn expect_byte(&mut self) -> JsonResult<u8> {
        self.next_byte().ok_or(JsonError::UnexpectedEndOfJson)
    }

    fn read_char_as_hexnumber(&mut self) -> JsonResult<u32> {
        let ch = try!(self.expect_byte());
        Ok(match ch {
            b'0' ... b'9' => (ch - b'0') as u32,
            b'a' ... b'f' => (ch + 10 - b'a') as u32,
            b'A' ... b'F' => (ch + 10 - b'A') as u32,
            ch            => return Err(JsonError::unexpected_character(ch)),
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
                        b'b' => 0x8,
                        b'f' => 0xC,
                        b't' => b'\t',
                        b'r' => b'\r',
                        b'n' => b'\n',
                        b'u' => {
                            try!(self.read_codepoint());
                            continue;
                        },
                        _   => ch
                    };
                    self.buffer.push(ch);
                },
                _     => self.buffer.push(ch)
            }
        }

        String::from_utf8(self.buffer.clone())
        .or(Err(JsonError::FailedUtf8Parsing))
    }

    fn read_number(&mut self, first: u8) -> JsonResult<f64> {
        let mut num = if first == b'-' { 0 } else { (first - b'0') as u64 };

        read_num!(self, digit, num = num * 10 + digit as u64);

        match self.peek_byte() {
            Some(b'.') | Some(b'e') | Some(b'E') => {},
            _ => {
                return if first == b'-' {
                    Ok(-(num as f64))
                } else {
                    Ok(num as f64)
                };
            }
        }

        let mut num = num as f64;

        if let Some(b'.') = self.peek_byte() {
            self.next_byte();

            let mut precision = -1;

            read_num!(self, digit, {
                num += (digit as f64) * 10_f64.powi(precision);
                precision -= 1;
            });
        }

        match self.next_byte() {
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

        Ok(if first == b'-' { num * -1.0 } else { num })
    }

    fn next(&mut self) -> JsonResult<Token> {
        loop {
            let ch = try!(self.expect_byte());
            return Ok(match ch {
                b',' => Token::Comma,
                b':' => Token::Colon,
                b'[' => Token::BracketOn,
                b']' => Token::BracketOff,
                b'{' => Token::BraceOn,
                b'}' => Token::BraceOff,
                b'"' => Token::String(try!(self.read_string())),
                b'0' ... b'9' | b'-' => Token::Number(try!(self.read_number(ch))),
                b't' => {
                    expect_char!(self, b'r', b'u', b'e');
                    Token::Boolean(true)
                },
                b'f' => {
                    expect_char!(self, b'a', b'l', b's', b'e');
                    Token::Boolean(false)
                },
                b'n' => {
                    expect_char!(self, b'u', b'l', b'l');
                    Token::Null
                },
                // whitespace
                9 ... 13 | 32 | 133 | 160 => continue,
                _ => return Err(JsonError::unexpected_character(ch))
            });
        }
    }
}

macro_rules! expect {
    ($parser:ident, $token:pat => $value:ident) => (
        match $parser.tokenizer.next() {
            Ok($token) => $value,
            Ok(token)  => return Err(JsonError::unexpected_token(token)),
            Err(error) => return Err(error),
        }
    );
    ($parser:ident, $token:pat) => ({
        match $parser.tokenizer.next() {
            Ok($token) => {},
            Ok(token)  => return Err(JsonError::unexpected_token(token)),
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

    #[must_use]
    fn ensure_end(&mut self) -> JsonResult<()> {
        match self.tokenizer.next() {
            Ok(token) => Err(JsonError::unexpected_token(token)),
            Err(JsonError::UnexpectedEndOfJson) => Ok(()),
            Err(error)                          => Err(error)
        }
    }

    fn array(&mut self) -> JsonResult<JsonValue> {
        let mut array = Vec::with_capacity(20);

        match try!(self.tokenizer.next()) {
            Token::BracketOff => return Ok(JsonValue::Array(array)),
            token             => array.push(try!(self.value_from(token))),
        }

        loop {
            match try!(self.tokenizer.next()) {
                Token::Comma => {
                    array.push(try!(self.value()));
                    continue
                },
                Token::BracketOff => break,
                token => return Err(JsonError::unexpected_token(token))
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
            token => return Err(JsonError::unexpected_token(token))
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
                token => return Err(JsonError::unexpected_token(token))
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
            token                   => {
                return Err(JsonError::unexpected_token(token))
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
