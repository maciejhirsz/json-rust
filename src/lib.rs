//! ![](http://terhix.com/doc/json-rust-logo-small.png)
//!
//! # json-rust
//!
//! Parse and serialize [JSON](http://json.org/) with ease.
//!
//! **[Complete Documentation](http://terhix.com/doc/json/) -**
//! **[Cargo](https://crates.io/crates/json) -**
//! **[Repository](https://github.com/maciejhirsz/json-rust)**
//!
//! ## Why?
//!
//! JSON is a very loose format where anything goes - arrays can hold mixed
//! types, object keys can change types between API calls or not include
//! some keys under some conditions. Mapping that to idiomatic Rust structs
//! introduces friction.
//!
//! This crate intends to avoid that friction.
//!
//! ```rust
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let parsed = json::parse(r#"
//!
//! {
//!     "code": 200,
//!     "success": true,
//!     "payload": {
//!         "features": [
//!             "awesome",
//!             "easyAPI",
//!             "lowLearningCurve"
//!         ]
//!     }
//! }
//!
//! "#).unwrap();
//!
//! let instantiated = object!{
//!     "code" => 200,
//!     "success" => true,
//!     "payload" => object!{
//!         "features" => array![
//!             "awesome",
//!             "easyAPI",
//!             "lowLearningCurve"
//!         ]
//!     }
//! };
//!
//! assert_eq!(parsed, instantiated);
//! # }
//! ```
//!
//! ## First class citizen
//!
//! Using macros and indexing, it's easy to work with the data.
//!
//! ```rust
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let mut data = object!{
//!     "foo" => false,
//!     "bar" => json::Null,
//!     "answer" => 42,
//!     "list" => array![json::Null, "world", true]
//! };
//!
//! // Partial equality is implemented for most raw types:
//! assert!(data["foo"] == false);
//!
//! // And it's type aware, `null` and `false` are different values:
//! assert!(data["bar"] != false);
//!
//! // But you can use any Rust number types:
//! assert!(data["answer"] == 42);
//! assert!(data["answer"] == 42.0);
//! assert!(data["answer"] == 42isize);
//!
//! // Access nested structures, arrays and objects:
//! assert!(data["list"][0].is_null());
//! assert!(data["list"][1] == "world");
//! assert!(data["list"][2] == true);
//!
//! // Error resilient - accessing properties that don't exist yield null:
//! assert!(data["this"]["does"]["not"]["exist"].is_null());
//!
//! // Mutate by assigning:
//! data["list"][0] = "Hello".into();
//!
//! // Use the `dump` method to serialize the data:
//! assert_eq!(data.dump(), r#"{"answer":42,"bar":null,"foo":false,"list":["Hello","world",true]}"#);
//!
//! // Or pretty print it out:
//! println!("{:#}", data);
//! # }
//! ```
//!
//! ## Serialize with `json::stringify(value)`
//!
//! Primitives:
//!
//! ```
//! // str slices
//! assert_eq!(json::stringify("foobar"), "\"foobar\"");
//!
//! // Owned strings
//! assert_eq!(json::stringify("foobar".to_string()), "\"foobar\"");
//!
//! // Any number types
//! assert_eq!(json::stringify(42), "42");
//!
//! // Booleans
//! assert_eq!(json::stringify(true), "true");
//! assert_eq!(json::stringify(false), "false");
//! ```
//!
//! Explicit `null` type `json::Null`:
//!
//! ```
//! assert_eq!(json::stringify(json::Null), "null");
//! ```
//!
//! Optional types:
//!
//! ```
//! let value: Option<String> = Some("foo".to_string());
//! assert_eq!(json::stringify(value), "\"foo\"");
//!
//! let no_value: Option<String> = None;
//! assert_eq!(json::stringify(no_value), "null");
//! ```
//!
//! Vector:
//!
//! ```
//! let data = vec![1,2,3];
//! assert_eq!(json::stringify(data), "[1,2,3]");
//! ```
//!
//! Vector with optional values:
//!
//! ```
//! let data = vec![Some(1), None, Some(2), None, Some(3)];
//! assert_eq!(json::stringify(data), "[1,null,2,null,3]");
//! ```
//!
//! Pushing to arrays:
//!
//! ```
//! let mut data = json::JsonValue::new_array();
//!
//! data.push(10);
//! data.push("foo");
//! data.push(false);
//!
//! assert_eq!(data.dump(), r#"[10,"foo",false]"#);
//! ```
//!
//! Putting fields on objects:
//!
//! ```
//! let mut data = json::JsonValue::new_object();
//!
//! data["answer"] = 42.into();
//! data["foo"] = "bar".into();
//!
//! assert_eq!(data.dump(), r#"{"answer":42,"foo":"bar"}"#);
//! ```
//!
//! `array!` macro:
//!
//! ```
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let data = array!["foo", "bar", 100, true, json::Null];
//! assert_eq!(data.dump(), r#"["foo","bar",100,true,null]"#);
//! # }
//! ```
//!
//! `object!` macro:
//!
//! ```
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let data = object!{
//!     "name"    => "John Doe",
//!     "age"     => 30,
//!     "canJSON" => true
//! };
//! assert_eq!(
//!     data.dump(),
//!     // Because object is internally using a BTreeMap,
//!     // the key order is alphabetical
//!     r#"{"age":30,"canJSON":true,"name":"John Doe"}"#
//! );
//! # }
//! ```

use std::io::Write;
use std::collections::{ BTreeMap, HashMap, btree_map };
use std::{ fmt, result };
use std::slice;

mod codegen;
mod parser;
mod value;
mod error;
mod short;

pub use error::Error;
pub use value::JsonValue;
pub use value::JsonValue::Null;
pub type Result<T> = result::Result<T, Error>;

/// Iterator over members of `JsonValue::Array`.
pub type Members<'a> = slice::Iter<'a, JsonValue>;

/// Mutable iterator over members of `JsonValue::Array`.
pub type MembersMut<'a> = slice::IterMut<'a, JsonValue>;

/// Iterator over key value pairs of `JsonValue::Object`.
pub type Entries<'a> = btree_map::Iter<'a, String, JsonValue>;

/// Mutable iterator over key value pairs of `JsonValue::Object`.
pub type EntriesMut<'a> = btree_map::IterMut<'a, String, JsonValue>;

#[deprecated(since="0.9.0", note="use `json::Error` instead")]
pub use Error as JsonError;

#[deprecated(since="0.9.0", note="use `json::Result` instead")]
pub use Result as JsonResult;

pub use parser::parse;
use codegen::{ Generator, PrettyGenerator, DumpGenerator, WriterGenerator };

pub type Array = Vec<JsonValue>;
pub type Object = BTreeMap<String, JsonValue>;

impl JsonValue {
    /// Prints out the value as JSON string.
    pub fn dump(&self) -> String {
        let mut gen = DumpGenerator::new();
        gen.write_json(self);
        gen.consume()
    }

    /// Pretty prints out the value as JSON string. Takes an argument that's
    /// number of spaces to indent new blocks with.
    pub fn pretty(&self, spaces: u16) -> String {
        let mut gen = PrettyGenerator::new(spaces);
        gen.write_json(self);
        gen.consume()
    }

    /// Dumps the JSON as byte stream into an instance of `std::io::Write`.
    pub fn to_writer<W: Write>(&self, writer: &mut W) {
        let mut gen = WriterGenerator::new(writer);
        gen.write_json(self);
    }
}

/// Implements formatting
///
/// ```
/// # use json;
/// let data = json::parse(r#"{"url":"https://github.com/"}"#).unwrap();
/// println!("{}", data);
/// println!("{:#}", data);
/// ```
impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            f.write_str(&self.pretty(4))
        } else {
            match *self {
                JsonValue::Short(ref value)   => value.fmt(f),
                JsonValue::String(ref value)  => value.fmt(f),
                JsonValue::Number(ref value)  => value.fmt(f),
                JsonValue::Boolean(ref value) => value.fmt(f),
                JsonValue::Null               => f.write_str("null"),
                _                             => f.write_str(&self.dump())
            }
        }
    }
}

/// Convenience for `JsonValue::from(value)`
pub fn from<T>(value: T) -> JsonValue where T: Into<JsonValue> {
    value.into()
}

/// Pretty prints out the value as JSON string.
pub fn stringify<T>(root: T) -> String where T: Into<JsonValue> {
    let root: JsonValue = root.into();
    root.dump()
}

/// Pretty prints out the value as JSON string. Second argument is a
/// number of spaces to indent new blocks with.
pub fn stringify_pretty<T>(root: T, spaces: u16) -> String where T: Into<JsonValue> {
    let root: JsonValue = root.into();
    root.pretty(spaces)
}


#[macro_export]
macro_rules! array {
    [] => ($crate::JsonValue::new_array());

    [ $( $item:expr ),* ] => ({
        let mut array = Vec::new();

        $(
            array.push($item.into());
        )*

        $crate::JsonValue::Array(array)
    })
}

#[macro_export]
macro_rules! object {
    {} => ($crate::JsonValue::new_object());

    { $( $key:expr => $value:expr ),* } => ({
        use std::collections::BTreeMap;

        let mut object = BTreeMap::new();

        $(
            object.insert($key.into(), $value.into());
        )*

        $crate::JsonValue::Object(object)
    })
}

macro_rules! implement_extras {
    ($from:ty) => {
        impl From<Option<$from>> for JsonValue {
            fn from(val: Option<$from>) -> JsonValue {
                match val {
                    Some(value) => value.into(),
                    None        => Null,
                }
            }
        }

        impl From<Vec<$from>> for JsonValue {
            fn from(mut val: Vec<$from>) -> JsonValue {
                JsonValue::Array(
                    val.drain(..)
                       .map(|value| value.into())
                       .collect()
                )
            }
        }

        impl From<Vec<Option<$from>>> for JsonValue {
            fn from(mut val: Vec<Option<$from>>) -> JsonValue {
                JsonValue::Array(
                    val.drain(..)
                       .map(|item| item.into())
                       .collect()
                )
            }
        }
    }
}

macro_rules! implement {
    ($to:ident, $from:ty as $wanted:ty) => {
        impl From<$from> for JsonValue {
            fn from(val: $from) -> JsonValue {
                JsonValue::$to(val as $wanted)
            }
        }

        impl PartialEq<$from> for JsonValue {
            fn eq(&self, other: &$from) -> bool {
                match *self {
                    JsonValue::$to(ref value) => value == &(*other as $wanted),
                    _ => false
                }
            }
        }

        impl<'a> PartialEq<$from> for &'a JsonValue {
            fn eq(&self, other: &$from) -> bool {
                match **self {
                    JsonValue::$to(ref value) => value == &(*other as $wanted),
                    _ => false
                }
            }
        }

        impl PartialEq<JsonValue> for $from {
            fn eq(&self, other: &JsonValue) -> bool {
                match *other {
                    JsonValue::$to(ref value) => value == &(*self as $wanted),
                    _ => false
                }
            }
        }

        implement_extras!($from);
    };
    ($to:ident, $from:ty) => {
        impl From<$from> for JsonValue {
            fn from(val: $from) -> JsonValue {
                JsonValue::$to(val)
            }
        }

        impl PartialEq<$from> for JsonValue {
            fn eq(&self, other: &$from) -> bool {
                match *self {
                    JsonValue::$to(ref value) => value == other,
                    _                         => false
                }
            }
        }

        impl<'a> PartialEq<$from> for &'a JsonValue {
            fn eq(&self, other: &$from) -> bool {
                match **self {
                    JsonValue::$to(ref value) => value == other,
                    _                         => false
                }
            }
        }

        impl PartialEq<JsonValue> for $from {
            fn eq(&self, other: &JsonValue) -> bool {
                match *other {
                    JsonValue::$to(ref value) => value == self,
                    _ => false
                }
            }
        }

        implement_extras!($from);
    }
}

impl<'a> From<&'a str> for JsonValue {
    fn from(val: &'a str) -> JsonValue {
        if val.len() <= short::MAX_LEN {
            JsonValue::Short(unsafe { short::Short::from_slice(val) })
        } else {
            JsonValue::String(val.into())
        }
    }
}

impl<'a> From<Option<&'a str>> for JsonValue {
    fn from(val: Option<&'a str>) -> JsonValue {
        match val {
            Some(value) => value.into(),
            None        => Null,
        }
    }
}

impl From<HashMap<String, JsonValue>> for JsonValue {
    fn from(mut val: HashMap<String, JsonValue>) -> JsonValue {
        let mut object = BTreeMap::new();

        for (key, value) in val.drain() {
            object.insert(key, value);
        }

        JsonValue::Object(object)
    }
}

impl From<Option<HashMap<String, JsonValue>>> for JsonValue {
    fn from(val: Option<HashMap<String, JsonValue>>) -> JsonValue {
        match val {
            Some(value) => value.into(),
            None        => Null,
        }
    }
}

impl From<Option<JsonValue>> for JsonValue {
    fn from(val: Option<JsonValue>) -> JsonValue {
        match val {
            Some(value) => value,
            None        => Null,
        }
    }
}

impl<'a> PartialEq<&'a str> for JsonValue {
    fn eq(&self, other: &&str) -> bool {
        match *self {
            JsonValue::Short(ref value)  => value == *other,
            JsonValue::String(ref value) => value == *other,
            _ => false
        }
    }
}

impl<'a> PartialEq<JsonValue> for &'a str {
    fn eq(&self, other: &JsonValue) -> bool {
        match *other {
            JsonValue::Short(ref value)  => value == *self,
            JsonValue::String(ref value) => value == *self,
            _ => false
        }
    }
}

impl PartialEq<str> for JsonValue {
    fn eq(&self, other: &str) -> bool {
        match *self {
            JsonValue::Short(ref value)  => value == other,
            JsonValue::String(ref value) => value == other,
            _ => false
        }
    }
}

impl<'a> PartialEq<JsonValue> for str {
    fn eq(&self, other: &JsonValue) -> bool {
        match *other {
            JsonValue::Short(ref value)  => value == self,
            JsonValue::String(ref value) => value == self,
            _ => false
        }
    }
}

implement!(String, String);
implement!(Number, isize as f64);
implement!(Number, usize as f64);
implement!(Number, i8 as f64);
implement!(Number, i16 as f64);
implement!(Number, i32 as f64);
implement!(Number, i64 as f64);
implement!(Number, u8 as f64);
implement!(Number, u16 as f64);
implement!(Number, u32 as f64);
implement!(Number, u64 as f64);
implement!(Number, f32 as f64);
implement!(Number, f64);
implement!(Object, Object);
implement!(Array, Array);
implement!(Boolean, bool);
