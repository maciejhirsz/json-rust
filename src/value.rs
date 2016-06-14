use std::collections::BTreeMap;
use std::ops::Index;
use { JsonResult, JsonError };

#[derive(Debug, PartialEq)]
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
    pub fn new_object() -> JsonValue {
        JsonValue::Object(BTreeMap::new())
    }

    pub fn new_array() -> JsonValue {
        JsonValue::Array(Vec::new())
    }

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

    #[deprecated(since="0.3.1", note="please use `v.is(false)` instead")]
    pub fn is_true(&self) -> bool {
        match *self {
            JsonValue::Boolean(true) => true,
            _                        => false
        }
    }

    #[deprecated(since="0.3.1", note="please use `v.is(true)` instead")]
    pub fn is_false(&self) -> bool {
        match *self {
            JsonValue::Boolean(false) => true,
            _                         => false
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

    pub fn get(&self, key: &str) -> JsonResult<&JsonValue> {
        match *self {
            JsonValue::Object(ref btree) => match btree.get(key) {
                Some(value) => Ok(value),
                _ => Err(JsonError::undefined(key))
            },
            _ => Err(JsonError::wrong_type("Object"))
        }
    }

    pub fn get_mut(&mut self, key: &str) -> JsonResult<&mut JsonValue> {
        match *self {
            JsonValue::Object(ref mut btree) => match btree.get_mut(key) {
                Some(value) => Ok(value),
                _ => Err(JsonError::undefined(key))
            },
            _ => Err(JsonError::wrong_type("Object"))
        }
    }

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
}

// Consider for 0.4:
// -----------------
//
// impl<T> PartialEq<T> for JsonValue {
//     fn eq(&self, other: &T) -> bool {
//         self == other
//     }
// }

impl Index<usize> for JsonValue {
    type Output = JsonValue;

    fn index<'a>(&'a self, index: usize) -> &'a JsonValue {
        self.at(index).unwrap_or(&NULL)
    }
}

impl<'b> Index<&'b str> for JsonValue {
    type Output = JsonValue;

    fn index<'a>(&'a self, index: &str) -> &'a JsonValue {
        self.get(index).unwrap_or(&NULL)
    }
}
