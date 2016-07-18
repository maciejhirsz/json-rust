// HERE BE DRAGONS!
// ================
//
// Making a fast parser is hard. This is a _not so naive_ implementation of
// recursive descent that does almost nothing. _There is no backtracking_, the
// whole parsing is 100% predictive, even though it's not BNF, and will have
// linear performance based on the length of the source!
//
// There is a lot of macros here! Like, woah! This is mostly due to the fact
// that Rust isn't very cool about optimizing inlined functions that return
// a `Result` type. Since different functions will have different `Result`
// signatures, the `try!` macro will always have to repackage our results.
// With macros those issues don't exist, the macro will return an unpackaged
// result - whatever it is - and if we ever stumble upon the error, we can
// return an `Err` without worrying about the exact signature of `Result`.
//
// This makes for some ugly code, but it is faster. Hopefully in the future
// with MIR support the compiler will get smarter about this.

use std::{ ptr, mem, str, slice, char };
use object::Object;
use number::Number;
use { JsonValue, Error, Result };

// This is not actual max precision, but a treshold at which number parsing
// kicks into checked math.
const MAX_PRECISION: u64 = 576460752303423500;


// Position is only used when we stumble upon an unexpected character. We don't
// track lines during parsing, as that would mean doing unnecessary work.
// Instead, if an error occurs, we figure out the line and column from the
// current index position of the parser.
struct Position {
    pub line: usize,
    pub column: usize,
}


// The `Parser` struct keeps track of indexing over our buffer. All niceness
// has been abandonned in favor of raw pointer magic. Does that make you feel
// dirty? _Good._
struct Parser<'a> {
    // Helper buffer for parsing strings that can't be just memcopied from
    // the original source (escaped characters)
    buffer: Vec<u8>,

    // String slice to parse
    source: &'a str,

    // Byte pointer to the slice above
    byte_ptr: *const u8,

    // Current index
    index: usize,

    // Lenght of the source
    length: usize,
}


// Read a byte from the source.
// Will return an error if there are no more bytes.
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


// Expect a sequence of specific bytes in specific order, error otherwise.
// This is useful for reading the 3 JSON identifiers:
//
// - "t" has to be followed by "rue"
// - "f" has to be followed by "alse"
// - "n" has to be followed by "ull"
//
// Anything else is an error.
macro_rules! expect_sequence {
    ($parser:ident, $( $ch:pat ),*) => {
        $(
            match expect_byte!($parser) {
                $ch => {},
                ch  => return $parser.unexpected_character(ch),
            }
        )*
    }
}


// A drop in macro for when we expect to read a byte, but we don't care
// about any whitespace characters that might occure before it.
macro_rules! expect_byte_ignore_whitespace {
    ($parser:ident) => ({
        let mut ch = expect_byte!($parser);

        // Don't go straight for the loop, assume we are in the clear first.
        match ch {
            // whitespace
            9 ... 13 | 32 => {
                loop {
                    match expect_byte!($parser) {
                        9 ... 13 | 32 => {},
                        next          => {
                            ch = next;
                            break;
                        }
                    }
                }
            },
            _ => {}
        }

        ch
    })
}


// Expect a particular byte to be next. Also available with a variant
// creates a `match` expression just to ease some pain.
macro_rules! expect {
    ($parser:ident, $byte:expr) => ({
        let ch = expect_byte_ignore_whitespace!($parser);

        if ch != $byte {
            return $parser.unexpected_character(ch)
        }
    });

    {$parser:ident $(, $byte:pat => $then:expr )*} => ({
        let ch = expect_byte_ignore_whitespace!($parser);

        match ch {
            $(
                $byte => $then,
            )*
            _ => return $parser.unexpected_character(ch)
        }

    })
}


// Look up table that marks which characters are allowed in their raw
// form in a string.
const QU: bool = false;  // double quote       0x22
const BS: bool = false;  // backslash          0x5C
const CT: bool = false;  // control character  0x00 ... 0x1F
const __: bool = true;

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


// Expect a string. This is called after encontering, and consuming, a
// double quote character. This macro has a happy path variant where it
// does almost nothing as long as all characters are allowed (as described
// in the look up table above). If it encounters a closing quote without
// any escapes, it will use a slice straight from the source, avoiding
// unnecessary buffering.
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


// Expect a number. Of some kind.
macro_rules! expect_number {
    ($parser:ident, $first:ident) => ({
        let mut num = ($first - b'0') as u64;

        let result: Number;

        // Cap on how many iterations we do while reading to u64
        // in order to avoid an overflow.
        loop {
            if num >= MAX_PRECISION {
                result = try!($parser.read_big_number(num));
                break;
            }

            if $parser.is_eof() {
                result = num.into();
                break;
            }

            let ch = $parser.read_byte();

            match ch {
                b'0' ... b'9' => {
                    $parser.bump();
                    // Avoid multiplication with bitshifts and addition
                    num = (num << 1) + (num << 3) + (ch - b'0') as u64;
                },
                _             => {
                    let mut e = 0;
                    result = allow_number_extensions!($parser, num, e, ch);
                    break;
                }
            }
        }

        result
    })
}


// Invoked after parsing an integer, this will account for fractions and/or
// `e` notation.
macro_rules! allow_number_extensions {
    ($parser:ident, $num:ident, $e:ident, $ch:ident) => ({
        match $ch {
            b'.'        => {
                $parser.bump();
                expect_fracton!($parser, $num, $e)
            },
            b'e' | b'E' => {
                $parser.bump();
                try!($parser.expect_exponent($num, $e))
            },
            _  => $num.into()
        }
    });

    // Alternative variant that defaults everything to 0. This is actually
    // quite handy as the only number that can begin with zero, has to have
    // a zero mantissa. Leading zeroes are illegal in JSON!
    ($parser:ident) => ({
        let mut num = 0;
        let mut e = 0;
        let ch = $parser.read_byte();
        allow_number_extensions!($parser, num, e, ch)
    })
}


// If a dot `b"."` byte has been read, start reading the decimal fraction
// of the number.
macro_rules! expect_fracton {
    ($parser:ident, $num:ident, $e:ident) => ({
        let result: Number;

        loop {
            if $parser.is_eof() {
                result = Number::from_parts(true, $num, $e as i16);
                break;
            }
            let ch = $parser.read_byte();

            match ch {
                b'0' ... b'9' => {
                    $parser.bump();
                    if $num < MAX_PRECISION {
                        $num = ($num << 3) + ($num << 1) + (ch - b'0') as u64;
                        $e -= 1;
                    } else {
                        match $num.checked_mul(10).and_then(|num| {
                            num.checked_add((ch - b'0') as u64)
                        }) {
                            Some(result) => {
                                $num = result;
                                $e -= 1;
                            },
                            None => {}
                        }
                    }
                },
                b'e' | b'E' => {
                    $parser.bump();
                    result = try!($parser.expect_exponent($num, $e));
                    break;
                }
                _ => {
                    result = Number::from_parts(true, $num, $e as i16);
                    break;
                }
            }
        }

        result
    })
}


// This is where the magic happens. This macro will read from the source
// and try to create an instance of `JsonValue`. Note that it only reads
// bytes that _begin_ a JSON value, however it can also accept an optional
// pattern with custom logic. This is used in arrays, which expect either
// a value or a closing bracket `b"]"`.
macro_rules! expect_value {
    {$parser:ident $(, $byte:pat => $then:expr )*} => ({
        let ch = expect_byte_ignore_whitespace!($parser);

        match ch {
            $(
                $byte => $then,
            )*
            b'[' => JsonValue::Array(try!($parser.read_array())),
            b'{' => JsonValue::Object(try!($parser.read_object())),
            b'"' => expect_string!($parser).into(),
            b'0' => JsonValue::Number(allow_number_extensions!($parser)),
            b'1' ... b'9' => {
                JsonValue::Number(expect_number!($parser, ch))
            },
            b'-' => {
                let ch = expect_byte!($parser);
                JsonValue::Number(- match ch {
                    b'0' => allow_number_extensions!($parser),
                    b'1' ... b'9' => expect_number!($parser, ch),
                    _    => return $parser.unexpected_character(ch)
                })
            }
            b't' => {
                expect_sequence!($parser, b'r', b'u', b'e');
                JsonValue::Boolean(true)
            },
            b'f' => {
                expect_sequence!($parser, b'a', b'l', b's', b'e');
                JsonValue::Boolean(false)
            },
            b'n' => {
                expect_sequence!($parser, b'u', b'l', b'l');
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

    // Check if we are at the end of the source.
    #[inline(always)]
    fn is_eof(&mut self) -> bool {
        self.index == self.length
    }

    // Read a byte from the source. Note that this does not increment
    // the index. In few cases (all of them related to number parsing)
    // we want to peek at the byte before doing anything. This will,
    // very very rarely, lead to a situation where the same byte is read
    // twice, but since this operation is using a raw pointer, the cost
    // is virtually irrelevant.
    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        unsafe { *self.byte_ptr.offset(self.index as isize) }
    }

    // Manually increment the index. Calling `read_byte` and then `bump`
    // is equivalent to consuming a byte on an iterator.
    #[inline(always)]
    fn bump(&mut self) {
        self.index = self.index.wrapping_add(1);
    }

    // Figure out the `Position` in the source. This doesn't look like it's
    // very fast - it probably isn't, and it doesn't really have to be.
    // This method is only called when an unexpected character error occurs.
    fn source_position_from_index(&self, index: usize) -> Position {
        let (bytes, _) = self.source.split_at(index-1);

        Position {
            line: bytes.lines().count(),
            column: bytes.lines().last().map_or(1, |line| {
                line.chars().count() + 1
            })
        }
    }

    // So we got an unexpected character, now what? Well, figure out where
    // it is, and throw an error!
    fn unexpected_character<T: Sized>(&mut self, byte: u8) -> Result<T> {
        let pos = self.source_position_from_index(self.index);

        // If the first byte is non ASCII (> 127), attempt to read the
        // codepoint from the following UTF-8 sequence. This can lead
        // to a fun scenario where an unexpected character error can
        // produce an end of json or UTF-8 failure error first :).
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

    // Boring
    fn read_hexdec_digit(&mut self) -> Result<u32> {
        let ch = expect_byte!(self);
        Ok(match ch {
            b'0' ... b'9' => (ch - b'0'),
            b'a' ... b'f' => (ch + 10 - b'a'),
            b'A' ... b'F' => (ch + 10 - b'A'),
            ch            => return self.unexpected_character(ch),
        } as u32)
    }

    // Boring
    fn read_hexdec_codepoint(&mut self) -> Result<u32> {
        Ok(
            try!(self.read_hexdec_digit()) << 12 |
            try!(self.read_hexdec_digit()) << 8  |
            try!(self.read_hexdec_digit()) << 4  |
            try!(self.read_hexdec_digit())
        )
    }

    // Oh look, some action. This method reads an escaped unicode
    // sequence such as `\uDEAD` from the string. Except `DEAD` is
    // not a valid codepoint, so it also needs to handle errors...
    fn read_codepoint(&mut self) -> Result<()> {
        let mut codepoint = try!(self.read_hexdec_codepoint());

        match codepoint {
            0x0000 ... 0xD7FF => {},
            0xD800 ... 0xDBFF => {
                codepoint -= 0xD800;
                codepoint <<= 10;

                expect_sequence!(self, b'\\', b'u');

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

    // What's so complex about strings you may ask? Not that much really.
    // This method is called if the `expect_string!` macro encounters an
    // escape. The added complexity is that it will have to use an internal
    // buffer to read all the escaped characters into, before finally
    // producing a usable slice. What it means it that parsing "foo\bar"
    // is whole lot slower than parsing "foobar", as the former suffers from
    // having to be read from source to a buffer and then from a buffer to
    // our target string. Nothing to be done about this, really.
    fn read_complex_string<'b>(&mut self, start: usize) -> Result<&'b str> {
        self.buffer.clear();
        let mut ch = b'\\';

        // TODO: Use fastwrite here as well
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
                // Because the buffer is stored on the parser, returning it
                // as a slice here freaks out the borrow checker. The compiler
                // can't know that the buffer isn't used till the result
                // of this function is long used and irrelevant. To avoid
                // issues here, we construct a new slice from raw parts, which
                // then has lifetime bound to the outer function scope instead
                // of the parser itself.
                slice::from_raw_parts(self.buffer.as_ptr(), self.buffer.len())
            )
        })
    }

    // Big numbers! If the `expect_number!` reaches a point where the decimal
    // mantissa could have overflown the size of u64, it will switch to this
    // control path instead. This method will pick up where the macro started,
    // but instead of continuing to read into the mantissa, it will increment
    // the exponent. Note that no digits are actually read here, as we already
    // exceeded the precision range of f64 anyway.
    fn read_big_number(&mut self, mut num: u64) -> Result<Number> {
        let mut e = 0i32;
        loop {
            if self.is_eof() {
                return Ok(Number::from_parts(true, num, e as i16));
            }
            let ch = self.read_byte();
            match ch {
                b'0' ... b'9' => {
                    self.bump();
                    match num.checked_mul(10).and_then(|num| {
                        num.checked_add((ch - b'0') as u64)
                    }) {
                        Some(result) => num = result,
                        None         => e += 1 ,
                    }
                },
                b'.' => {
                    self.bump();
                    return Ok(expect_fracton!(self, num, e));
                },
                b'e' | b'E' => {
                    self.bump();
                    return self.expect_exponent(num, e);
                }
                _  => break
            }
        }

        Ok(Number::from_parts(true, num, e as i16))
    }

    // Called in the rare case that a number with `e` notation has been
    // encountered. This is pretty straight forward, I guess.
    fn expect_exponent(&mut self, num: u64, big_e: i32) -> Result<Number> {
        let mut ch = expect_byte!(self);
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

        let mut e = match ch {
            b'0' ... b'9' => (ch - b'0') as i32,
            _ => return self.unexpected_character(ch),
        };

        loop {
            if self.is_eof() {
                break;
            }
            let ch = self.read_byte();
            match ch {
                b'0' ... b'9' => {
                    self.bump();
                    e = (e << 3) + (e << 1) + (ch - b'0') as i32;
                },
                _  => break
            }
        }

        Ok(Number::from_parts(true, num, (big_e + (e * sign)) as i16))
    }

    // Given how compilcated reading numbers and strings is, reading objects
    // is actually pretty simple.
    fn read_object(&mut self) -> Result<Object> {
        let key = expect!{ self,
            b'}'  => return Ok(Object::new()),
            b'\"' => expect_string!(self)
        };

        let mut object = Object::with_capacity(3);

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

    // And reading arrays is simpler still!
    fn read_array(&mut self) -> Result<Vec<JsonValue>> {
        let first = expect_value!{ self, b']' => return Ok(Vec::new()) };

        let mut array = Vec::with_capacity(2);

        unsafe {
            // First member can be written to the array without any checks!
            ptr::copy_nonoverlapping(
                &first as *const JsonValue,
                array.as_mut_ptr(),
                1
            );
            mem::forget(first);
            array.set_len(1);
        }

        expect!{
            self,
            b']' => return Ok(array),
            b',' => {
                // Same for the second one!
                let value = expect_value!(self);
                unsafe {
                    ptr::copy_nonoverlapping(
                        &value as *const JsonValue,
                        array.as_mut_ptr().offset(1),
                        1
                    );
                    mem::forget(value);
                    array.set_len(2);
                }
            }
        }

        loop {
            expect!{ self,
                b']' => break,
                b',' => array.push(expect_value!(self))
            };
        }

        Ok(array)
    }

    // Parse away!
    fn parse(&mut self) -> Result<JsonValue> {
        let value = expect_value!(self);

        // We have read whatever value was there, but we need to make sure
        // there is nothing left to read - if there is, that's an error.
        while !self.is_eof() {
            match self.read_byte() {
                9 ... 13 | 32 => self.bump(),
                ch            => {
                    self.bump();
                    return self.unexpected_character(ch);
                }
            }
        }

        Ok(value)
    }
}

// All that hard work, and in the end it's just a single function in the API.
#[inline]
pub fn parse(source: &str) -> Result<JsonValue> {
    Parser::new(source).parse()
}
