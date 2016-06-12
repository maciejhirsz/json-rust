//! # JSON in Result
//!
//! Parse and serialize JSON with ease.
//!
//! ## Serialize with `json::stringify(value)`
//!
//! Primitives:
//!
//! ```
//! assert_eq!(json::stringify("foobar"), "\"foobar\"");
//! assert_eq!(json::stringify(42), "42");
//! assert_eq!(json::stringify(true), "true");
//! assert_eq!(json::stringify(false), "false");
//! assert_eq!(json::stringify(json::Null), "null");
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
//! `array!` macro:
//!
//! ```
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let data = array!["foo", "bar", 100, true, json::Null];
//! assert_eq!(json::stringify(data), "[\"foo\",\"bar\",100,true,null]");
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
//!     json::stringify(data),
//!     // Because object is internally using a BTreeMap,
//!     // the key order is alphabetical
//!     "{\"age\":30,\"canJSON\":true,\"name\":\"John Doe\"}"
//! );
//! # }
//! ```
//!
//!
//!

mod codegen;
mod parser;
mod value;
mod error;

pub use error::JsonError;
pub use value::JsonValue;
pub use value::JsonValue::Null;
pub type JsonResult<T> = Result<T, JsonError>;

pub use parser::parse;
use codegen::Generator;

use std::collections::HashMap;
use std::collections::BTreeMap;

pub type Array = Vec<JsonValue>;
pub type Object = BTreeMap<String, JsonValue>;

pub fn stringify<T>(root: T) -> String where T: Into<JsonValue> {
    let mut gen = Generator::new(true);
    gen.write_json(&root.into());
    gen.consume()
}

#[macro_export]
macro_rules! array {
    [] => (json::JsonValue::new_array());

    [ $( $item:expr ),* ] => ({
        let mut array = Vec::new();

        $(
            array.push($item.into());
        )*

        json::JsonValue::Array(array)
    })
}

#[macro_export]
macro_rules! object {
    {} => (json::JsonValue::new_object());

    { $( $key:expr => $value:expr ),* } => ({
        let mut object = std::collections::BTreeMap::new();

        $(
            object.insert($key.into(), $value.into());
        )*

        json::JsonValue::Object(object)
    })
}

macro_rules! implement_extras {
    ($from:ty) => {
        impl Into<JsonValue> for Option<$from> {
            fn into(self) -> JsonValue {
                match self {
                    Some(value) => value.into(),
                    None        => Null,
                }
            }
        }

        impl Into<JsonValue> for Vec<$from> {
            fn into(mut self) -> JsonValue {
                JsonValue::Array(self.drain(..)
                    .map(|value| value.into())
                    .collect::<Vec<JsonValue>>()
                )
            }
        }

        impl Into<JsonValue> for Vec<Option<$from>> {
            fn into(mut self) -> JsonValue {
                JsonValue::Array(self.drain(..)
                    .map(|item| item.into())
                    .collect::<Vec<JsonValue>>()
                )
            }
        }
    }
}

macro_rules! implement {
    ($to:ident, $from:ty as $wanted:ty) => {
        impl Into<JsonValue> for $from {
            fn into(self) -> JsonValue {
                JsonValue::$to(self as $wanted)
            }
        }

        implement_extras!($from);
    };
    ($to:ident, $from:ty) => {
        impl Into<JsonValue> for $from {
            fn into(self) -> JsonValue {
                JsonValue::$to(self)
            }
        }

        implement_extras!($from);
    }
}

impl<'a> Into<JsonValue> for &'a str {
    fn into(self) -> JsonValue {
        JsonValue::String(self.to_string())
    }
}

impl<'a> Into<JsonValue> for Option<&'a str> {
    fn into(self) -> JsonValue {
        match self {
            Some(value) => value.into(),
            None        => Null,
        }
    }
}

impl Into<JsonValue> for HashMap<String, JsonValue> {
    fn into(mut self) -> JsonValue {
        let mut object = BTreeMap::new();

        for (key, value) in self.drain() {
            object.insert(key, value);
        }

        JsonValue::Object(object)
    }
}

impl Into<JsonValue> for Option<HashMap<String, JsonValue>> {
    fn into(self) -> JsonValue {
        match self {
            Some(value) => value.into(),
            None        => Null,
        }
    }
}

impl Into<JsonValue> for Option<JsonValue> {
    fn into(self) -> JsonValue {
        match self {
            Some(value) => value,
            None        => Null,
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
