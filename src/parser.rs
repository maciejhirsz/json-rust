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

use std::str;
use arena::Arena;
use list::{List, ListBuilder};
use object::Object;
use number::Number;
use { Json, JsonValue, Error, Result };

// This is not actual max precision, but a threshold at which number parsing
// kicks into checked math.
const MAX_PRECISION: u64 = 576460752303423500;


// How many nested Objects/Arrays are allowed to be parsed
const DEPTH_LIMIT: usize = 512;

trait ToError {
    fn to_error(&mut Parser) -> Self;
}

impl<T> ToError for Result<T> {
    fn to_error(parser: &mut Parser) -> Result<T> {
        Err(parser.error.take().unwrap_or_else(|| Error::UnexpectedEndOfJson))
    }
}

impl<'a> ToError for &'a JsonValue<'a> {
    fn to_error(_: &mut Parser) -> &'a JsonValue<'a> {
        &JsonValue::Null
    }
}

impl ToError for () {
    #[inline(always)]
    fn to_error(_: &mut Parser) -> () {
        ()
    }
}

impl ToError for u8 {
    #[inline(always)]
    fn to_error(_: &mut Parser) -> u8 {
        0
    }
}

impl ToError for u32 {
    #[inline(always)]
    fn to_error(_: &mut Parser) -> u32 {
        0
    }
}

impl ToError for Number {
    fn to_error(_: &mut Parser) -> Number {
        0u64.into()
    }
}

impl<'a> ToError for &'a str {
    fn to_error(_: &mut Parser) -> &'a str {
        ""
    }
}

macro_rules! unwind_loop {
    ($iteration:expr) => ({
        $iteration
        $iteration
        $iteration
        $iteration

        loop {
            $iteration
            $iteration
            $iteration
            $iteration
        }
    })
}

// The `Parser` struct keeps track of indexing over our buffer. All niceness
// has been abandoned in favor of raw pointer magic. Does that make you feel
// dirty? _Good._
struct Parser<'arena> {
    // Parsing error to be returned
    error: Option<Error>,

    // Allocation arena
    arena: &'arena Arena,

    // String slice to parse
    source: &'arena str,

    // Helper buffer for parsing strings that can't be just memcopied from
    // the original source (escaped characters)
    buffer: Vec<u8>,

    // Byte pointer to the slice above
    byte_ptr: *const u8,

    // Current index
    index: usize,

    // Length of the source
    length: usize,
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
            match $parser.read_byte() {
                $ch => $parser.bump(),
                _   => unexpected_character!($parser),
            }
        )*
    }
}

// Expect to find EOF or just whitespaces leading to EOF after a JSON value
macro_rules! expect_eof {
    ($parser:ident) => ({
        while !$parser.is_eof() {
            match $parser.read_byte() {
                9 ... 13 | 32 => $parser.bump(),
                _             => unexpected_character!($parser)
            }
        }
    })
}

// Expect a particular byte to be next. Also available with a variant
// creates a `match` expression just to ease some pain.
macro_rules! expect {
    ($parser:ident, $byte:pat) => {
        match $parser.ignore_whitespace() {
            $byte => $parser.bump(),
            _     => unexpected_character!($parser)
        }
    }
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


macro_rules! unexpected_character {
    ($parser:ident) => ({
        $parser.err_unexpected_character();
        return ToError::to_error($parser);
    })
}

// Invoked after parsing an integer, this will account for fractions and/or
// `e` notation.
macro_rules! allow_number_extensions {
    ($parser:ident, $num:ident, $e:ident, $ch:ident) => ({
        match $ch {
            b'.'        => {
                $parser.bump();
                expect_fraction!($parser, $num, $e)
            },
            b'e' | b'E' => {
                $parser.bump();
                $parser.read_exponent($num, $e)
            },
            _  => $num.into()
        }
    });

    // Alternative variant that defaults everything to 0. This is actually
    // quite handy as the only number that can begin with zero, has to have
    // a zero mantissa. Leading zeroes are illegal in JSON!
    ($parser:ident) => ({
        if $parser.is_eof() {
            0.into()
        } else {
            let mut num = 0;
            let mut e = 0;
            let ch = $parser.read_byte();
            allow_number_extensions!($parser, num, e, ch)
        }
    })
}


// If a dot `b"."` byte has been read, start reading the decimal fraction
// of the number.
macro_rules! expect_fraction {
    ($parser:ident, $num:ident, $e:ident) => ({
        let result: Number;

        let ch = $parser.expect_byte();

        match ch {
            b'0' ... b'9' => {
                if $num < MAX_PRECISION {
                    $num = $num * 10 + (ch - b'0') as u64;
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
            _ => unexpected_character!($parser)
        }

        loop {
            if $parser.is_eof() {
                result = unsafe { Number::from_parts_unchecked(true, $num, $e) };
                break;
            }
            let ch = $parser.read_byte();

            match ch {
                b'0' ... b'9' => {
                    $parser.bump();
                    if $num < MAX_PRECISION {
                        $num = $num * 10 + (ch - b'0') as u64;
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
                    result = $parser.read_exponent($num, $e);
                    break;
                }
                _ => {
                    result = unsafe { Number::from_parts_unchecked(true, $num, $e) };
                    break;
                }
            }
        }

        result
    })
}

impl<'arena> Parser<'arena> {
    pub fn new(arena: &'arena Arena, source: &'arena str) -> Self {
        Parser {
            error: None,
            source,
            arena,
            buffer: Vec::with_capacity(30),
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

    #[inline(always)]
    fn alloc<T: Copy>(&mut self, val: T) -> &'arena T {
        self.arena.alloc(val)
    }

    // Read a byte from the source. Note that this does not increment
    // the index. In few cases (all of them related to number parsing)
    // we want to peek at the byte before doing anything. This will,
    // very very rarely, lead to a situation where the same byte is read
    // twice, but since this operation is using a raw pointer, the cost
    // is virtually irrelevant.
    #[inline(always)]
    fn read_byte(&mut self) -> u8 {
        debug_assert!(self.index < self.length, "Reading out of bounds");

        unsafe { *self.byte_ptr.offset(self.index as isize) }
    }

    #[inline(always)]
    fn expect_byte(&mut self) -> u8 {
        if self.is_eof() {
            self.err(|| Error::UnexpectedEndOfJson);
            return 0;
        }

        let ch = self.read_byte();
        self.bump();
        ch
    }

    #[inline(always)]
    fn ignore_whitespace(&mut self) -> u8 {
        unwind_loop!({
            match self.read_byte() {
                9 ... 13 | 32 => self.bump(),
                ch            => return ch
            }
        })
    }

    // Manually increment the index. Calling `read_byte` and then `bump`
    // is equivalent to consuming a byte on an iterator.
    #[inline(always)]
    fn bump(&mut self) {
        self.index = self.index.wrapping_add(1);
    }

    fn err<F>(&mut self, f: F)
        where F: FnOnce() -> Error
    {
        if self.error.is_some() {
            return;
        }

        self.error = Some(f())
    }

    // So we got an unexpected character, now what? Well, figure out where
    // it is, and throw an error!
    fn err_unexpected_character(&mut self) {
        let at = self.index - 1;

        let ch = self.source[at..]
                     .chars()
                     .next()
                     .expect("Must have a character");

        let (lineno, col) = self.source[..at]
                                .lines()
                                .enumerate()
                                .last()
                                .unwrap_or((0, ""));

        let colno = col.chars().count();

        self.err(move || {
            Error::UnexpectedCharacter {
                ch,
                line: lineno + 1,
                column: colno + 1,
            }
        });
    }

    // Boring
    fn read_hexdec_digit(&mut self) -> u32 {
        match self.expect_byte() {
            ch @ b'0' ... b'9' => (ch - b'0') as u32,
            ch @ b'a' ... b'f' => (ch + 10 - b'a') as u32,
            ch @ b'A' ... b'F' => (ch + 10 - b'A') as u32,
            _                  => unexpected_character!(self),
        }
    }

    #[inline(always)]
    fn read_hexdec_codepoint(&mut self) -> u32 {
        self.read_hexdec_digit() << 12 |
        self.read_hexdec_digit() << 8  |
        self.read_hexdec_digit() << 4  |
        self.read_hexdec_digit()
    }

    // Oh look, some action. This method reads an escaped unicode
    // sequence such as `\uDEAD` from the string. Except `DEAD` is
    // not a valid codepoint, so it also needs to handle errors...
    fn read_codepoint(&mut self) {
        let mut codepoint = self.read_hexdec_codepoint();

        match codepoint {
            0x0000 ... 0xD7FF => {},
            0xD800 ... 0xDBFF => {
                codepoint -= 0xD800;
                codepoint <<= 10;

                expect_sequence!(self, b'\\', b'u');

                let lower = self.read_hexdec_codepoint();

                if let 0xDC00 ... 0xDFFF = lower {
                    codepoint = (codepoint | lower - 0xDC00) + 0x010000;
                } else {
                    return self.err(|| Error::FailedUtf8Parsing);
                }
            },
            0xE000 ... 0xFFFF => {},
            _ => return self.err(|| Error::FailedUtf8Parsing)
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
            _ => return self.err(|| Error::FailedUtf8Parsing)
        }
    }

    #[inline(always)]
    fn read_string(&mut self) -> &'arena str {
        let start = self.index;

        loop {
            match self.read_byte() {
                ch if ALLOWED[ch as usize] => self.bump(),

                b'"'  => {
                    let end = self.index;
                    self.bump();
                    return &self.source[start..end];
                },
                b'\\' => return self.read_complex_string(start),
                _     => unexpected_character!(self),
            }
        }
    }

    // What's so complex about strings you may ask? Not that much really.
    // This method is called if the `read_string` function encounters an
    // escape. The added complexity is that it will have to use an internal
    // buffer to read all the escaped characters into, before finally
    // producing a usable slice. What it means it that parsing "foo\bar"
    // is whole lot slower than parsing "foobar", as the former suffers from
    // having to be read from source to a buffer and then from a buffer to
    // our target string. Nothing to be done about this, really.
    fn read_complex_string(&mut self, start: usize) -> &'arena str {
        self.buffer.clear();

        // TODO: Use fastwrite here as well
        self.buffer.extend_from_slice(self.source[start .. self.index].as_bytes());

        // let mut ch = b'\\';
        self.bump();

        loop {
            match self.read_byte() {
                ch if ALLOWED[ch as usize] => {
                    // TODO: keep pushing slices
                    self.buffer.push(ch);
                    self.bump();
                }

                b'"'  => {
                    self.bump();
                    break;
                },
                b'\\' => {
                    self.bump();

                    let escaped = match self.expect_byte() {
                        b'u'  => {
                            self.read_codepoint();
                            continue;
                        },
                        escaped @ b'"'  |
                        escaped @ b'\\' |
                        escaped @ b'/'  => escaped,
                        b'b'  => 0x8,
                        b'f'  => 0xC,
                        b't'  => b'\t',
                        b'r'  => b'\r',
                        b'n'  => b'\n',
                        _     => unexpected_character!(self)
                    };
                    self.buffer.push(escaped);
                },
                _ => unexpected_character!(self)
            }
        }

        // Since the original source is already valid UTF-8, and `\`
        // cannot occur in front of a codepoint > 127, this is safe.
        let string = unsafe {
            str::from_utf8_unchecked(
                self.buffer.as_slice()
            )
        };

        self.arena.alloc_str(string)
    }

    #[inline(always)]
    fn read_number(&mut self, mut num: u64) -> Number {
        // Cap on how many iterations we do while reading to u64
        // in order to avoid an overflow.
        loop {
            if self.is_eof() {
                return num.into();
            }

            match self.read_byte() {
                ch @ b'0' ... b'9' => {
                    self.bump();
                    num = num * 10 + (ch - b'0') as u64;

                    if num >= MAX_PRECISION {
                        return self.read_big_number(num);
                    }
                },
                ch => {
                    let mut e = 0;
                    return allow_number_extensions!(self, num, e, ch);
                }
            }
        }
    }

    // Big numbers! If the `read_number` reaches a point where the decimal
    // mantissa could have overflown the size of u64, it will switch to this
    // control path instead. This method will pick up where the macro started,
    // but instead of continuing to read into the mantissa, it will increment
    // the exponent. Note that no digits are actually read here, as we already
    // exceeded the precision range of f64 anyway.
    fn read_big_number(&mut self, mut num: u64) -> Number {
        let mut e = 0i16;
        loop {
            if self.is_eof() {
                return unsafe { Number::from_parts_unchecked(true, num, e) };
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
                    return expect_fraction!(self, num, e);
                },
                b'e' | b'E' => {
                    self.bump();
                    return self.read_exponent(num, e);
                }
                _  => break
            }
        }

        unsafe { Number::from_parts_unchecked(true, num, e) }
    }

    // Called in the rare case that a number with `e` notation has been
    // encountered. This is pretty straight forward, I guess.
    fn read_exponent(&mut self, num: u64, big_e: i16) -> Number {
        let mut ch = self.expect_byte();
        let sign = match ch {
            b'-' => {
                ch = self.expect_byte();
                -1
            },
            b'+' => {
                ch = self.expect_byte();
                1
            },
            _    => 1
        };

        let mut e = match ch {
            b'0' ... b'9' => (ch - b'0') as i16,
            _ => unexpected_character!(self),
        };

        loop {
            if self.is_eof() {
                break;
            }
            let ch = self.read_byte();
            match ch {
                b'0' ... b'9' => {
                    self.bump();
                    e = e.saturating_mul(10).saturating_add((ch - b'0') as i16);
                },
                _  => break
            }
        }

        unsafe { Number::from_parts_unchecked(true, num, (big_e.saturating_add(e * sign))) }
    }

    #[inline(always)]
    fn byte_to_val(&mut self, byte: u8) -> &'arena JsonValue<'arena> {
        fn ar<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let mut builder = match parser.ignore_whitespace() {
                b']' => {
                    parser.bump();
                    return parser.alloc(JsonValue::Array(List::empty()));
                },
                ch   => {
                    parser.bump();
                    ListBuilder::new(&parser.arena, parser.byte_to_val(ch))
                }
            };

            loop {
                match parser.ignore_whitespace() {
                    b']' => {
                        parser.bump();
                        return parser.alloc(JsonValue::Array(builder.into_list()));
                    },
                    b',' => {
                        parser.bump();
                        builder.push(parser.read_value())
                    },
                    _ => unexpected_character!(parser)
                }
            }
        }

        fn oj<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let object = Object::new();
            let result = parser.alloc(JsonValue::Object(object));

            match parser.ignore_whitespace() {
                b'}' => {
                    parser.bump();
                    return result;
                },
                b'"' => {
                    parser.bump();

                    let key = parser.read_string();
                    expect!(parser, b':');

                    object.insert_allocated(&parser.arena, key, parser.read_value())
                },
                _ => unexpected_character!(parser)
            }

            loop {
                match parser.ignore_whitespace() {
                    b'}' => {
                        parser.bump();
                        return result;
                    },
                    b',' => {
                        parser.bump();
                        expect!(parser, b'"');
                        let key = parser.read_string();
                        expect!(parser, b':');

                        object.insert_allocated(&parser.arena, key, parser.read_value())
                    },
                    _ => unexpected_character!(parser)
                }
            }
        }

        fn st<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::String(parser.read_string());
            parser.alloc(val)
        }

        fn n0<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(allow_number_extensions!(parser));
            parser.alloc(val)
        }

        fn n1<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(parser.read_number(1));
            parser.alloc(val)
        }

        fn n2<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(parser.read_number(2));
            parser.alloc(val)
        }

        fn n3<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(parser.read_number(3));
            parser.alloc(val)
        }

        fn n4<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(parser.read_number(4));
            parser.alloc(val)
        }

        fn n5<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(parser.read_number(5));
            parser.alloc(val)
        }

        fn n6<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(parser.read_number(6));
            parser.alloc(val)
        }

        fn n7<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(parser.read_number(7));
            parser.alloc(val)
        }

        fn n8<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(parser.read_number(8));
            parser.alloc(val)
        }

        fn n9<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let val = JsonValue::Number(parser.read_number(9));
            parser.alloc(val)
        }

        fn nn<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            let ch = parser.expect_byte();
            let val = JsonValue::Number(- match ch {
                b'0' => allow_number_extensions!(parser),
                b'1' ... b'9' => parser.read_number((ch - b'0') as u64),
                _    => unexpected_character!(parser)
            });
            parser.alloc(val)
        }

        fn tr<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            expect_sequence!(parser, b'r', b'u', b'e');
            &JsonValue::Boolean(true)
        }

        fn fl<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            expect_sequence!(parser, b'a', b'l', b's', b'e');
            &JsonValue::Boolean(false)
        }

        fn nl<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            expect_sequence!(parser, b'u', b'l', b'l');
            &JsonValue::Null
        }

        fn __<'arena>(parser: &mut Parser<'arena>) -> &'arena JsonValue<'arena> {
            unexpected_character!(parser)
        }

        static BYTE_TO_VAL: [for<'arena> fn(&mut Parser<'arena>) -> &'arena JsonValue<'arena>; 256] = [
        // 0   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 0
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
          __, __, st, __, __, __, __, __, __, __, __, __, __, nn, __, __, // 2
          n0, n1, n2, n3, n4, n5, n6, n7, n8, n9, __, __, __, __, __, __, // 3
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
          __, __, __, __, __, __, __, __, __, __, __, ar, __, __, __, __, // 5
          __, __, __, __, __, __, fl, __, __, __, __, __, __, __, nl, __, // 6
          __, __, __, __, tr, __, __, __, __, __, __, oj, __, __, __, __, // 7
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
          __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
        ];

        BYTE_TO_VAL[byte as usize](self)
    }

    #[inline(always)]
    fn read_value(&mut self) -> &'arena JsonValue<'arena> {
        let ch = self.ignore_whitespace();
        self.bump();
        self.byte_to_val(ch)
    }

    // Parse away!
    #[inline(always)]
    fn parse(&mut self) -> Result<&'arena JsonValue<'arena>> {
        let value = self.read_value();

        expect_eof!(self);

        Ok(value)
    }
}

// All that hard work, and in the end it's just a single function in the API.
pub fn parse(source: &str) -> Result<Json> {
    let arena = Arena::new();
    let root = {
        let source = arena.alloc_str(source);
        Parser::new(&arena, source).parse().map(|value| value as *const JsonValue as usize)
    };

    root.map(move |root| Json {
        arena,
        root
    })
}
