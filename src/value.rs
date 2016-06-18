use std::collections::BTreeMap;
use std::ops::Index;
use iterators::{ Members, MembersMut, Entries, EntriesMut };
use { JsonResult, JsonError };

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
    pub fn is<T>(&self, other: T) -> bool where T: Into<JsonValue> {
        *self == other.into()
    }

    pub fn is_string(&self) -> bool {
        match *self {
            JsonValue::String(_) => true,
            _                    => false,
        }
    }

    pub fn as_string(&self) -> JsonResult<&String> {
        match *self {
            JsonValue::String(ref value) => Ok(value),
            _ => Err(JsonError::wrong_type("String"))
        }
    }

    pub fn is_number(&self) -> bool {
        match *self {
            JsonValue::Number(_) => true,
            _                    => false,
        }
    }

    pub fn as_number(&self) -> JsonResult<&f64> {
        match *self {
            JsonValue::Number(ref value) => Ok(value),
            _ => Err(JsonError::wrong_type("Number"))
        }
    }

    pub fn is_boolean(&self) -> bool {
        match *self {
            JsonValue::Boolean(_) => true,
            _                     => false
        }
    }

    pub fn as_boolean(&self) -> JsonResult<&bool> {
        match *self {
            JsonValue::Boolean(ref value) => Ok(value),
            _ => Err(JsonError::wrong_type("Boolean"))
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
    pub fn with(&mut self, key: &str) -> &mut JsonValue {
        match *self {
            JsonValue::Object(ref mut btree) => {
                if !btree.contains_key(key) {
                    btree.insert(key.to_string(), JsonValue::Null);
                }
                btree.get_mut(key).unwrap()
            },
            _ => {
                *self = JsonValue::new_object();
                self.put(key, JsonValue::Null).unwrap();
                return self.get_mut(key).unwrap();
            }
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

    pub fn members(&self) -> Members {
        match *self {
            JsonValue::Array(ref vec) => {
                Members::Some(vec.iter())
            },
            _ => Members::None
        }
    }

    pub fn members_mut(&mut self) -> MembersMut {
        match *self {
            JsonValue::Array(ref mut vec) => {
                MembersMut::Some(vec.iter_mut())
            },
            _ => MembersMut::None
        }
    }

    /// Works on `JsonValue::Object` - returns an iterator over entries.
    pub fn entries(&self) -> Entries {
        match *self {
            JsonValue::Object(ref btree) => {
                Entries::Some(btree.iter())
            },
            _ => Entries::None
        }
    }

    /// Works on `JsonValue::Object` - returns a mutable iterator over entries.
    pub fn entries_mut(&mut self) -> EntriesMut {
        match *self {
            JsonValue::Object(ref mut btree) => {
                EntriesMut::Some(btree.iter_mut())
            },
            _ => EntriesMut::None
        }
    }
}

/// Implements indexing by `usize` to easily access members of an array:
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

    fn index<'a>(&'a self, index: usize) -> &'a JsonValue {
        self.at(index).unwrap_or(&NULL)
    }
}

/// Implements indexing by `&str` to easily access object members:
///
/// ```
/// # use json::JsonValue;
/// let mut object = JsonValue::new_object();
///
/// object.put("foo", "bar");
///
/// assert!(object["foo"].is("bar"));
/// ```
impl<'b> Index<&'b str> for JsonValue {
    type Output = JsonValue;

    fn index<'a>(&'a self, index: &str) -> &'a JsonValue {
        self.get(index).unwrap_or(&NULL)
    }
}
