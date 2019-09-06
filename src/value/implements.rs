// This is a private module that contains `PartialEq` and `From` trait
// implementations for `JsonValue`.

use std::collections::{BTreeMap, HashMap};

use crate::short::{self, Short};
use crate::number::Number;
use crate::object::Object;
use crate::value::JsonValue;

macro_rules! implement_eq {
    ($to:ident, $from:ty) => {
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
    }
}

macro_rules! implement {
    ($to:ident, $from:ty as num) => {
        impl From<$from> for JsonValue {
            fn from(val: $from) -> JsonValue {
                JsonValue::$to(val.into())
            }
        }

        implement_eq!($to, $from);
    };
    ($to:ident, $from:ty) => {
        impl From<$from> for JsonValue {
            fn from(val: $from) -> JsonValue {
                JsonValue::$to(val)
            }
        }

        implement_eq!($to, $from);
    }
}

impl<'a> From<&'a str> for JsonValue {
    fn from(val: &'a str) -> JsonValue {
        if val.len() <= short::MAX_LEN {
            JsonValue::Short(unsafe { Short::from_slice(val) })
        } else {
            JsonValue::String(val.into())
        }
    }
}

impl<T: Into<JsonValue>> From<Option<T>> for JsonValue {
    fn from(val: Option<T>) -> JsonValue {
        match val {
            Some(val) => val.into(),
            None      => JsonValue::Null,
        }
    }
}

impl<T: Into<JsonValue>> From<Vec<T>> for JsonValue {
    fn from(val: Vec<T>) -> JsonValue {
        JsonValue::Array(val.into_iter().map(Into::into).collect())
    }
}

impl<'a, T: Into<JsonValue> + Clone> From<&'a [T]> for JsonValue {
    fn from(val: &'a [T]) -> JsonValue {
        JsonValue::Array(val.iter().cloned().map(Into::into).collect())
    }
}

impl<K: AsRef<str>, V: Into<JsonValue>> From<HashMap<K, V>> for JsonValue {
    fn from(val: HashMap<K, V>) -> JsonValue {
        JsonValue::Object(val.into_iter().collect())
    }
}

impl<K: AsRef<str>, V: Into<JsonValue>> From<BTreeMap<K, V>> for JsonValue {
    fn from(val: BTreeMap<K, V>) -> JsonValue {
        JsonValue::Object(val.into_iter().collect())
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
implement!(Number, isize as num);
implement!(Number, usize as num);
implement!(Number, i8 as num);
implement!(Number, i16 as num);
implement!(Number, i32 as num);
implement!(Number, i64 as num);
implement!(Number, u8 as num);
implement!(Number, u16 as num);
implement!(Number, u32 as num);
implement!(Number, u64 as num);
implement!(Number, f32 as num);
implement!(Number, f64 as num);
implement!(Number, Number);
implement!(Object, Object);
implement!(Boolean, bool);
