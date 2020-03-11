// This is a private module that contains `PartialEq` and `From` trait
// implementations for `JsonValue`.

use std::collections::{BTreeMap, HashMap};
use cowvec::CowStr;

use crate::short::{self, Short};
use crate::number::Number;
use crate::object::Object;
use crate::value::JsonValue;

macro_rules! implement_eq {
    ($to:ident, $from:ty) => {
        impl<'json> PartialEq<$from> for JsonValue<'_> {
            fn eq(&self, other: &$from) -> bool {
                match *self {
                    JsonValue::$to(ref value) => value == other,
                    _                         => false
                }
            }
        }

        impl<'json> PartialEq<$from> for &JsonValue<'json> {
            fn eq(&self, other: &$from) -> bool {
                match **self {
                    JsonValue::$to(ref value) => value == other,
                    _                         => false
                }
            }
        }

        impl<'json> PartialEq<JsonValue<'json>> for $from {
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
        impl<'json> From<$from> for JsonValue<'json> {
            fn from(val: $from) -> JsonValue<'json> {
                JsonValue::$to(val.into())
            }
        }

        implement_eq!($to, $from);
    };
    ($to:ident, $from:ty) => {
        impl<'json> From<$from> for JsonValue<'json> {
            fn from(val: $from) -> JsonValue<'json> {
                JsonValue::$to(val.into())
            }
        }

        implement_eq!($to, $from);
    }
}

impl<'json> From<&'json str> for JsonValue<'json> {
    fn from(val: &'json str) -> JsonValue<'json> {
        JsonValue::String(val.into())
    }
}

impl<'json> From<CowStr<'json>> for JsonValue<'json> {
    fn from(val: CowStr<'json>) -> JsonValue<'json> {
        JsonValue::String(val)
    }
}

impl<'json, T> From<Option<T>> for JsonValue<'json>
where
    T: Into<JsonValue<'json>>,
{
    fn from(val: Option<T>) -> JsonValue<'json> {
        match val {
            Some(val) => val.into(),
            None      => JsonValue::Null,
        }
    }
}

impl<'json, T> From<Vec<T>> for JsonValue<'json>
where
    T: Into<JsonValue<'json>>,
{
    fn from(val: Vec<T>) -> JsonValue<'json> {
        JsonValue::Array(val.into_iter().map(Into::into).collect())
    }
}

impl<'json, T> From<&[T]> for JsonValue<'json>
where
    T: Into<JsonValue<'json>> + Clone,
{
    fn from(val: &[T]) -> JsonValue<'json> {
        JsonValue::Array(val.iter().cloned().map(Into::into).collect())
    }
}

impl<'json, K, V> From<HashMap<K, V>> for JsonValue<'json>
where
    K: Into<CowStr<'json>> + 'json,
    V: Into<JsonValue<'json>>,
{
    fn from(val: HashMap<K, V>) -> JsonValue<'json> {
        JsonValue::Object(val.into_iter().collect())
    }
}

impl<'json, K, V> From<BTreeMap<K, V>> for JsonValue<'json>
where
    K: Into<CowStr<'json>> + 'json,
    V: Into<JsonValue<'json>>,
{
    fn from(val: BTreeMap<K, V>) -> JsonValue<'json> {
        JsonValue::Object(val.into_iter().collect())
    }
}

impl PartialEq<&str> for JsonValue<'_> {
    fn eq(&self, other: &&str) -> bool {
        match self {
            JsonValue::Short(value)  => value == *other,
            JsonValue::String(value) => value.as_ref() == *other,
            _ => false
        }
    }
}

impl PartialEq<JsonValue<'_>> for &str {
    fn eq(&self, other: &JsonValue) -> bool {
        match other {
            JsonValue::Short(value)  => value == *self,
            JsonValue::String(value) => value.as_ref() == *self,
            _ => false
        }
    }
}

impl PartialEq<str> for JsonValue<'_> {
    fn eq(&self, other: &str) -> bool {
        match self {
            JsonValue::Short(value)  => value == other,
            JsonValue::String(value) => value.as_ref() == other,
            _ => false
        }
    }
}

impl PartialEq<JsonValue<'_>> for str {
    fn eq(&self, other: &JsonValue) -> bool {
        match other {
            JsonValue::Short(value)  => value == self,
            JsonValue::String(value) => value.as_ref() == self,
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
implement!(Object, Object<'json>);
implement!(Boolean, bool);
