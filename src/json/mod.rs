mod into_json;

use arena::Arena;
use list::List;
use number::Number;
use std::fmt::{self, Debug};

pub use json::into_json::IntoJson;

pub use object::Object;
pub type Array<'json> = List<'json, JsonValue<'json>>;


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum JsonValue<'json> {
    Null,
    String(&'json str),
    Number(Number),
    Boolean(bool),
    Object(Object<'json>),
    Array(Array<'json>)
}

pub struct Json {
    pub(crate) arena: Arena,
    pub(crate) root: usize,
}

impl Json {
    #[inline]
    pub fn new<'arena, F>(f: F) -> Self
        where F: FnOnce(&Arena) -> &JsonValue
    {
        let arena = Arena::new();
        let root = {
            f(&arena) as *const JsonValue as usize
        };

        Json {
            arena,
            root,
        }
    }

    #[inline]
    pub fn value<'json>(&'json self) -> &'json JsonValue<'json> {
        unsafe { &*(self.root as *const JsonValue) }
    }
}

impl PartialEq<Json> for Json {
    fn eq(&self, other: &Json) -> bool {
        self.value() == other.value()
    }
}

impl Debug for Json {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.value(), f)
    }
}
