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
    [] => (JsonValue::Array(Vec::new()));

    [ $( $item:expr ),* ] => ({
        let mut array = Vec::new();

        $(
            array.push($item.into());
        )*

        JsonValue::Array(array)
    })
}

#[macro_export]
macro_rules! object {
    {} => (JsonValue::Object(BTreeMap::new()));

    { $( $key:expr => $value:expr ),* } => ({
        let mut object = BTreeMap::new();

        $(
            object.insert($key.into(), $value.into());
        )*

        JsonValue::Object(object)
    })
}

macro_rules! implement {
    ($to:ident, $from:ty as $wanted:ty) => {
        impl Into<JsonValue> for $from {
            fn into(self) -> JsonValue {
                JsonValue::$to(self as $wanted)
            }
        }

        impl Into<JsonValue> for Option<$from> {
            fn into(self) -> JsonValue {
                match self {
                    Some(value) => JsonValue::$to(value as $wanted),
                    None        => JsonValue::Null,
                }
            }
        }
    };
    ($to:ident, $from:ty) => {
        impl Into<JsonValue> for $from {
            fn into(self) -> JsonValue {
                JsonValue::$to(self)
            }
        }

        impl Into<JsonValue> for Option<$from> {
            fn into(self) -> JsonValue {
                match self {
                    Some(value) => JsonValue::$to(value),
                    None        => JsonValue::Null,
                }
            }
        }
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
            None        => JsonValue::Null,
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
            None        => JsonValue::Null,
        }
    }
}

implement!(String, String);
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
