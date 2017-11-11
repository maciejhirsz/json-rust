//! ![](http://terhix.com/doc/json-rust-logo-small.png)
//!
//! # json-rust
//!
//! Parse and serialize [JSON](http://json.org/) with ease.
//!
//! **[Changelog](https://github.com/maciejhirsz/json-rust/releases) -**
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
//! let instantiated = json!({
//!     code: 200,
//!     success: true,
//!     payload: {
//!         features: [
//!             "awesome",
//!             "easyAPI",
//!             "lowLearningCurve"
//!         ]
//!     }
//! });
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
//! let mut data = json!({
//!     foo: false,
//!     bar: null,
//!     answer: 42,
//!     list: [null, "world", true]
//! });
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
//! assert_eq!(data.dump(), r#"{"foo":false,"bar":null,"answer":42,"list":["Hello","world",true]}"#);
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
//! Creating arrays with `json!` macro:
//!
//! ```
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let data = json!(["foo", "bar", 100, true, null]);
//! assert_eq!(data.dump(), r#"["foo","bar",100,true,null]"#);
//! # }
//! ```
//!
//! Creating objects with `json!` macro:
//!
//! ```
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let data = json!({
//!     name: "John Doe",
//!     age: 30,
//!     canJSON: true
//! });
//! assert_eq!(
//!     data.dump(),
//!     r#"{"name":"John Doe","age":30,"canJSON":true}"#
//! );
//! # }
//! ```
use std::result;

mod arena;
mod cell;
// mod codegen;
mod parser;
mod json;
mod error;
mod util;

pub mod list;
pub mod object;
pub mod number;

pub use error::Error;
pub use json::{Json, JsonValue, IntoJson, Array};

/// Result type used by this crate.
///
///
/// *Note:* Since 0.9.0 the old `JsonResult` type is deprecated. Always use
/// `json::Result` instead.
pub type Result<T> = result::Result<T, Error>;

// pub mod iterators {
//     /// Iterator over members of `JsonValue::Array`.
//     pub type Members<'a> = ::std::slice::Iter<'a, super::JsonValue>;

//     /// Mutable iterator over members of `JsonValue::Array`.
//     pub type MembersMut<'a> = ::std::slice::IterMut<'a, super::JsonValue>;

//     /// Iterator over key value pairs of `JsonValue::Object`.
//     pub type Entries<'a> = super::object::Iter<'a>;

//     /// Mutable iterator over key value pairs of `JsonValue::Object`.
//     pub type EntriesMut<'a> = super::object::IterMut<'a>;
// }

pub use parser::parse;

// pub type Array = Vec<JsonValue>;

// /// Convenience for `JsonValue::from(value)`
// pub fn from<T>(value: T) -> JsonValue where T: Into<JsonValue> {
//     value.into()
// }

// /// Pretty prints out the value as JSON string.
// pub fn stringify<T>(root: T) -> String where T: Into<JsonValue> {
//     let root: JsonValue = root.into();
//     root.dump()
// }

// /// Pretty prints out the value as JSON string. Second argument is a
// /// number of spaces to indent new blocks with.
// pub fn stringify_pretty<T>(root: T, spaces: u16) -> String where T: Into<JsonValue> {
//     let root: JsonValue = root.into();
//     root.pretty(spaces)
// }

#[macro_export]
macro_rules! json {
    (@ $arena:expr => { $( $key:ident: $value:tt, )* }) => ({
        let object = $crate::object::Object::new();

        $(
            object.insert_allocated($arena, stringify!($key), json!(@ $arena => $value));
        )*

        $arena.alloc($crate::JsonValue::Object(object))
    });
    (@ $arena:expr => [ $first:tt, $( $item:tt, )* ]) => ({
        let mut builder = $crate::list::ListBuilder::new($arena, json!(@ $arena => $first));

        $(
            builder.push(json!(@ $arena => $item));
        )*

        $arena.alloc($crate::JsonValue::Array(builder.into_list()))
    });
    (@ $arena:expr => []) => ({
        $arena.alloc($crate::JsonValue::Array($crate::list::List::empty()))
    });
    (@ $arena:expr => [ $( $item:tt ),* ]) => { json!(@ $arena => [ $( $item, )* ]) };
    (@ $arena:expr => { $( $key:ident: $value:tt ),* }) => { json!(@ $arena => { $( $key: $value, )* }) };
    (@ $arena:expr => null) => { &$crate::JsonValue::Null };
    (@ $arena:expr => $item:expr) => { $crate::IntoJson::into_json($item, $arena) };
    ({ $( $key:ident: $value:tt ),* }) => {
        $crate::Json::new(|arena| json!(@ arena => { $( $key: $value, )* }))
    };
    ([ $( $item:tt ),* ]) => {
        $crate::Json::new(|arena| json!(@ arena => [ $( $item, )* ]))
    };
    ($item:expr) => {
        $crate::Json::new(|arena| json!(@ arena => $item))
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_simple_object() {
        json!({
            foo: "foo",
            bar: "bar"
        });
    }

    #[test]
    fn crete_simple_array() {
        json!([ "foo", "bar" ]);
    }

    #[test]
    fn create_nested_object() {
        json!({
            name: "json-rust",
            stuff: ["foobar", 20, null]
        });
    }

    #[test]
    fn create_nested_array() {
        json!([ { id: 1 }, { id: 2 } ]);
    }
}
