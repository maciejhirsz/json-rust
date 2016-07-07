use std::collections::BTreeMap;
use std::ops::{ Index, IndexMut, Deref };
use iterators::{ Members, MembersMut, Entries, EntriesMut };
use { JsonResult, JsonError };
use std::{ mem, usize, u8, u16, u32, u64, isize, i8, i16, i32, i64, f32 };

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

    pub fn is_string(&self) -> bool {
        match *self {
            JsonValue::String(_) => true,
            _                    => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match *self {
            JsonValue::Number(_) => true,
            _                    => false,
        }
    }

    pub fn is_boolean(&self) -> bool {
        match *self {
            JsonValue::Boolean(_) => true,
            _                     => false
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

    /// Checks whether the value is empty. Returns true for:
    ///
    /// - empty string (`""`)
    /// - number `0`
    /// - boolean `false`
    /// - null
    /// - empty array (`array![]`)
    /// - empty object (`object!{}`)
    pub fn is_empty(&self) -> bool {
        match *self {
            JsonValue::String(ref value)  => value.is_empty(),
            JsonValue::Number(ref value)  => !value.is_normal(),
            JsonValue::Boolean(ref value) => !value,
            JsonValue::Null               => true,
            JsonValue::Array(ref value)   => value.is_empty(),
            JsonValue::Object(ref value)  => value.is_empty(),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            JsonValue::String(ref value) => Some(value.as_ref()),
            _                            => None
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

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            JsonValue::Boolean(ref value) => Some(*value),
            _                             => None
        }
    }

    /// Take over the ownership of the value, leaving `Null` in it's place.
    ///
    /// ## Example
    ///
    /// ```
    /// # #[macro_use] extern crate json;
    /// # fn main() {
    /// let mut data = array!["Foo", 42];
    ///
    /// let first = data[0].take();
    /// let second = data[1].take();
    ///
    /// assert!(first == "Foo");
    /// assert!(second == 42);
    ///
    /// assert!(data[0].is_null());
    /// assert!(data[1].is_null());
    /// # }
    /// ```
    pub fn take(&mut self) -> JsonValue {
        mem::replace(self, JsonValue::Null)
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

    /// Works on `JsonValue::Array` - remove and return last element from
    /// an array. On failure returns a null.
    pub fn pop(&mut self) -> JsonValue {
        match *self {
            JsonValue::Array(ref mut vec) => {
                vec.pop().unwrap_or(JsonValue::Null)
            },
            _ => JsonValue::Null
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

    /// Works on `JsonValue::Object` - remove a key and return the value it held.
    /// If the key was not present, the method is called on anything but an
    /// object, it will return a null.
    pub fn remove(&mut self, key: &str) -> JsonValue {
        match *self {
            JsonValue::Object(ref mut btree) => {
                btree.remove(key).unwrap_or(JsonValue::Null)
            },
            _ => JsonValue::Null
        }
    }

    /// When called on an array or an object, will wipe them clean. When called
    /// on a string will clear the string. Numbers and booleans become null.
    pub fn clear(&mut self) {
        match *self {
            JsonValue::String(ref mut string) => string.clear(),
            JsonValue::Object(ref mut btree)  => btree.clear(),
            JsonValue::Array(ref mut vec)     => vec.clear(),
            _                                 => *self = JsonValue::Null,
        }
    }
}

/// Implements indexing by `usize` to easily access array members:
///
/// ## Example
///
/// ```
/// # use json::JsonValue;
/// let mut array = JsonValue::new_array();
///
/// array.push("foo");
///
/// assert!(array[0] == "foo");
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

/// Implements mutable indexing by `usize` to easily modify array members:
///
/// ## Example
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
/// assert!(array[1] == "bar");
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
                self.index_mut(index)
            }
        }
    }
}

/// Implements indexing by `&str` to easily access object members:
///
/// ## Example
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
/// assert!(object["foo"] == "bar");
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

impl Index<String> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: String) -> &JsonValue {
        self.index(index.deref())
    }
}

impl<'a> Index<&'a String> for JsonValue {
    type Output = JsonValue;

    fn index(&self, index: &String) -> &JsonValue {
        self.index(index.deref())
    }
}

/// Implements mutable indexing by `&str` to easily modify object members:
///
/// ## Example
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
/// assert!(object["foo"] == 42);
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
                self.index_mut(index)
            }
        }
    }
}

impl IndexMut<String> for JsonValue {
    fn index_mut(&mut self, index: String) -> &mut JsonValue {
        self.index_mut(index.deref())
    }
}

impl<'a> IndexMut<&'a String> for JsonValue {
    fn index_mut(&mut self, index: &String) -> &mut JsonValue {
        self.index_mut(index.deref())
    }
}
