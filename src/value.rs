use std::collections::BTreeMap;
use std::ops::{ Index, IndexMut };
use iterators::{ Members, MembersMut, Entries, EntriesMut };
use { JsonResult, JsonError };
use std::{ usize, u8, u16, u32, u64, isize, i8, i16, i32, i64, f32 };

macro_rules! f64_to_unsinged {
    ($unsigned:ident, $value:expr) => {
        if $value < 0.0 || $value > $unsigned::MAX as f64 {
            None
        } else {
            Some($value as $unsigned)
        }
    }
}

macro_rules! f64_to_singed {
    ($signed:ident, $value:expr) => {
        if $value < $signed::MIN as f64 || $value > $signed::MAX as f64 {
            None
        } else {
            Some($value as $signed)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Object(BTreeMap<String, JsonValue>),
    Array(Vec<JsonValue>),
}

static NULL: JsonValue = JsonValue::Null;

impl JsonValue {
    /// Create an empty `JsonValue::Object` instance.
    /// When creating an object with data, consider using the `object!` macro.
    pub fn new_object() -> JsonValue {
        JsonValue::Object(BTreeMap::new())
    }

    /// Create an empty `JsonValue::Array` instance.
    /// When creating array with data, consider using the `array!` macro.
    pub fn new_array() -> JsonValue {
        JsonValue::Array(Vec::new())
    }

    /// Checks if the value stored matches `other`.
    #[deprecated(since="0.7.0", note="Use `value == other` instead")]
    pub fn is<T>(&self, other: T) -> bool where T: Into<JsonValue> {
        *self == other.into()
    }

    pub fn is_string(&self) -> bool {
        match *self {
            JsonValue::String(_) => true,
            _                    => false,
        }
    }

    /// Deprecated because the return type is planned to change to
    /// `Option<String>` eventually down the road.
    #[deprecated(since="0.6.1", note="Use `as_str` instead")]
    pub fn as_string(&self) -> JsonResult<&String> {
        match *self {
            JsonValue::String(ref value) => Ok(value),
            _ => Err(JsonError::wrong_type("String"))
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            JsonValue::String(ref value) => Some(value.as_ref()),
            _                            => None
        }
    }

    pub fn is_number(&self) -> bool {
        match *self {
            JsonValue::Number(_) => true,
            _                    => false,
        }
    }

    #[deprecated(since="0.6.1", note="Use `as_f64` instead")]
    pub fn as_number(&self) -> JsonResult<&f64> {
        match *self {
            JsonValue::Number(ref value) => Ok(value),
            _ => Err(JsonError::wrong_type("Number"))
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            JsonValue::Number(ref value) => Some(*value),
            _                            => None
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        self.as_f64().and_then(|value| f64_to_singed!(f32, value))
    }

    pub fn as_u64(&self) -> Option<u64> {
        self.as_f64().and_then(|value| f64_to_unsinged!(u64, value))
    }

    pub fn as_u32(&self) -> Option<u32> {
        self.as_f64().and_then(|value| f64_to_unsinged!(u32, value))
    }

    pub fn as_u16(&self) -> Option<u16> {
        self.as_f64().and_then(|value| f64_to_unsinged!(u16, value))
    }

    pub fn as_u8(&self) -> Option<u8> {
        self.as_f64().and_then(|value| f64_to_unsinged!(u8, value))
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.as_f64().and_then(|value| f64_to_unsinged!(usize, value))
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.as_f64().and_then(|value| f64_to_singed!(i64, value))
    }

    pub fn as_i32(&self) -> Option<i32> {
        self.as_f64().and_then(|value| f64_to_singed!(i32, value))
    }

    pub fn as_i16(&self) -> Option<i16> {
        self.as_f64().and_then(|value| f64_to_singed!(i16, value))
    }

    pub fn as_i8(&self) -> Option<i8> {
        self.as_f64().and_then(|value| f64_to_singed!(i8, value))
    }

    pub fn as_isize(&self) -> Option<isize> {
        self.as_f64().and_then(|value| f64_to_singed!(isize, value))
    }

    pub fn is_boolean(&self) -> bool {
        match *self {
            JsonValue::Boolean(_) => true,
            _                     => false
        }
    }

    #[deprecated(since="0.6.1", note="Use `as_bool` instead")]
    pub fn as_boolean(&self) -> JsonResult<&bool> {
        match *self {
            JsonValue::Boolean(ref value) => Ok(value),
            _ => Err(JsonError::wrong_type("Boolean"))
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            JsonValue::Boolean(ref value) => Some(*value),
            _                             => None
        }
    }

    pub fn is_null(&self) -> bool {
        match *self {
            JsonValue::Null => true,
            _               => false,
        }
    }

    pub fn is_object(&self) -> bool {
        match *self {
            JsonValue::Object(_) => true,
            _                    => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match *self {
            JsonValue::Array(_) => true,
            _                   => false,
        }
    }

    /// Works on `JsonValue::Object` - create or override key with value.
    #[must_use]
    #[deprecated(since="0.6.0", note="Use `object[key] = value.into()` instead")]
    pub fn put<T>(&mut self, key: &str, value: T) -> JsonResult<()>
    where T: Into<JsonValue> {
        match *self {
            JsonValue::Object(ref mut btree) => {
                btree.insert(key.into(), value.into());
                Ok(())
            },
            _ => Err(JsonError::wrong_type("Object"))
        }
    }

    /// Works on `JsonValue::Object` - get a reference to a value behind key.
    /// For most purposes consider using `object[key]` instead.
    #[deprecated(since="0.6.0", note="Use `object[key]` instead")]
    pub fn get(&self, key: &str) -> JsonResult<&JsonValue> {
        match *self {
            JsonValue::Object(ref btree) => match btree.get(key) {
                Some(value) => Ok(value),
                _ => Err(JsonError::undefined(key))
            },
            _ => Err(JsonError::wrong_type("Object"))
        }
    }

    /// Works on `JsonValue::Object` - get a mutable reference to a value behind
    /// the key.
    #[deprecated(since="0.6.0", note="Use `object[key]` instead")]
    pub fn get_mut(&mut self, key: &str) -> JsonResult<&mut JsonValue> {
        match *self {
            JsonValue::Object(ref mut btree) => match btree.get_mut(key) {
                Some(value) => Ok(value),
                _ => Err(JsonError::undefined(key))
            },
            _ => Err(JsonError::wrong_type("Object"))
        }
    }

    /// Attempts to get a mutable reference to the value behind a key on an
    /// object. If the reference doesn't exists, it will be created and
    /// assigned a null. If `self` is not an object, an empty object with
    /// null key will be created.
    #[deprecated(since="0.6.0", note="Use `object[key]` instead")]
    pub fn with(&mut self, key: &str) -> &mut JsonValue {
        if !self.is_object() {
            *self = JsonValue::new_object();
        }

        match *self {
            JsonValue::Object(ref mut btree) => {
                if !btree.contains_key(key) {
                    btree.insert(key.to_string(), JsonValue::Null);
                }
                btree.get_mut(key).unwrap()
            },
            _ => unreachable!()
        }
    }

    /// Works on `JsonValue::Array` - pushes a new value to the array.
    #[must_use]
    pub fn push<T>(&mut self, value: T) -> JsonResult<()>
    where T: Into<JsonValue> {
        match *self {
            JsonValue::Array(ref mut vec) => {
                vec.push(value.into());
                Ok(())
            },
            _ => Err(JsonError::wrong_type("Array"))
        }
    }

    /// Works on `JsonValue::Array` - gets a reference to a value at index.
    /// For most purposes consider using `array[index]` instead.
    #[deprecated(since="0.6.0", note="Use `array[index]` instead")]
    pub fn at(&self, index: usize) -> JsonResult<&JsonValue> {
        match *self {
            JsonValue::Array(ref vec) => {
                if index < vec.len() {
                    Ok(&vec[index])
                } else {
                    Err(JsonError::ArrayIndexOutOfBounds)
                }
            },
            _ => Err(JsonError::wrong_type("Array"))
        }
    }

    /// Works on `JsonValue::Array` - gets a mutable reference to a value
    /// at index.
    #[deprecated(since="0.6.0", note="Use `array[index]` instead")]
    pub fn at_mut(&mut self, index: usize) -> JsonResult<&mut JsonValue> {
        match *self {
            JsonValue::Array(ref mut vec) => {
                if index < vec.len() {
                    Ok(&mut vec[index])
                } else {
                    Err(JsonError::ArrayIndexOutOfBounds)
                }
            },
            _ => Err(JsonError::wrong_type("Array"))
        }
    }

    /// Works on `JsonValue::Array` - checks if the array contains a value
    pub fn contains<T>(&self, item: T) -> bool where T: Into<JsonValue> {
        match *self {
            JsonValue::Array(ref vec) => {
                vec.contains(&item.into())
            },
            _ => false
        }
    }

    /// Returns length of array or object (number of keys), defaults to `0` for
    /// other types.
    pub fn len(&self) -> usize {
        match *self {
            JsonValue::Array(ref vec) => {
                vec.len()
            },
            JsonValue::Object(ref btree) => {
                btree.len()
            },
            _ => 0
        }
    }

    /// Works on `JsonValue::Array` - returns an iterator over members.
    pub fn members(&self) -> Members {
        match *self {
            JsonValue::Array(ref vec) => {
                Members::Some(vec.iter())
            },
            _ => Members::None
        }
    }

    /// Works on `JsonValue::Array` - returns a mutable iterator over members.
    pub fn members_mut(&mut self) -> MembersMut {
        match *self {
            JsonValue::Array(ref mut vec) => {
                MembersMut::Some(vec.iter_mut())
            },
            _ => MembersMut::None
        }
    }

    /// Works on `JsonValue::Object` - returns an iterator over key value pairs.
    pub fn entries(&self) -> Entries {
        match *self {
            JsonValue::Object(ref btree) => {
                Entries::Some(btree.iter())
            },
            _ => Entries::None
        }
    }

    /// Works on `JsonValue::Object` - returns a mutable iterator over
    /// key value pairs.
    pub fn entries_mut(&mut self) -> EntriesMut {
        match *self {
            JsonValue::Object(ref mut btree) => {
                EntriesMut::Some(btree.iter_mut())
            },
            _ => EntriesMut::None
        }
    }
}

/// Implements indexing by `usize` to easily access array members:
///
/// ```
/// # use json::JsonValue;
/// let mut array = JsonValue::new_array();
///
/// array.push("foo");
///
/// assert!(array[0].is("foo"));
/// ```
impl Index<usize> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: usize) -> &JsonValue {
        match *self {
            JsonValue::Array(ref vec) => vec.get(index).unwrap_or(&NULL),
            _ => &NULL
        }
    }
}

/// Implements mutable indexing by `usie` to easily modify array members:
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// #
/// # fn main() {
/// let mut array = array!["foo", 3.14];
///
/// array[1] = "bar".into();
///
/// assert!(array[1].is("bar"));
/// # }
/// ```
impl IndexMut<usize> for JsonValue {
    fn index_mut(&mut self, index: usize) -> &mut JsonValue {
        match *self {
            JsonValue::Array(ref mut vec) => {
                let in_bounds = index < vec.len();

                if in_bounds {
                    &mut vec[index]
                } else {
                    vec.push(JsonValue::Null);
                    vec.last_mut().unwrap()
                }
            }
            _ => {
                *self = JsonValue::new_array();
                self.push(JsonValue::Null).unwrap();
                &mut self[0]
            }
        }
    }
}

/// Implements indexing by `&str` to easily access object members:
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// #
/// # fn main() {
/// let object = object!{
///     "foo" => "bar"
/// };
///
/// assert!(object["foo"].is("bar"));
/// # }
/// ```
impl<'a> Index<&'a str> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: &str) -> &JsonValue {
        match *self {
            JsonValue::Object(ref btree) => match btree.get(index) {
                Some(value) => value,
                _ => &NULL
            },
            _ => &NULL
        }
    }
}

/// Implements mutable indexing by `&str` to easily modify object members:
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// #
/// # fn main() {
/// let mut object = object!{};
///
/// object["foo"] = 42.into();
///
/// assert!(object["foo"].is(42));
/// # }
/// ```
impl<'a> IndexMut<&'a str> for JsonValue {
    fn index_mut(&mut self, index: &str) -> &mut JsonValue {
        match *self {
            JsonValue::Object(ref mut btree) => {
                if !btree.contains_key(index) {
                    btree.insert(index.to_string(), JsonValue::Null);
                }
                btree.get_mut(index).unwrap()
            },
            _ => {
                *self = JsonValue::new_object();
                &mut self[index]
            }
        }
    }
}
