//! ![](https://raw.githubusercontent.com/maciejhirsz/json-rust/master/json-rust-logo-small.png)
//!
//! # json-rust
//!
//! Parse and serialize [JSON](http://json.org/) with ease.
//!
//! **[Changelog](https://github.com/maciejhirsz/json-rust/releases) -**
//! **[Complete Documentation](https://docs.rs/json/) -**
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
//! let instantiated = object!{
//!     // quotes on keys are optional
//!     "code": 200,
//!     success: true,
//!     payload: {
//!         features: [
//!             "awesome",
//!             "easyAPI",
//!             "lowLearningCurve"
//!         ]
//!     }
//! };
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
//! let mut data = object!{
//!     foo: false,
//!     bar: null,
//!     answer: 42,
//!     list: [null, "world", true]
//! };
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
//! `array!` macro:
//!
//! ```
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let data = array!["foo", "bar", 100, true, null];
//! assert_eq!(data.dump(), r#"["foo","bar",100,true,null]"#);
//! # }
//! ```
//!
//! `object!` macro:
//!
//! ```
//! # #[macro_use] extern crate json;
//! # fn main() {
//! let data = object!{
//!     name: "John Doe",
//!     age: 30,
//!     canJSON: true
//! };
//! assert_eq!(
//!     data.dump(),
//!     r#"{"name":"John Doe","age":30,"canJSON":true}"#
//! );
//! # }
//! ```

use std::result;

pub mod codegen;
mod parser;
mod value;
mod error;
mod util;

pub mod short;
pub mod object;
pub mod number;

pub use error::Error;
pub use value::JsonValue;
pub use value::JsonValue::Null;

/// Result type used by this crate.
///
///
/// *Note:* Since 0.9.0 the old `JsonResult` type is deprecated. Always use
/// `json::Result` instead.
pub type Result<T> = result::Result<T, Error>;

pub mod iterators {
    /// Iterator over members of `JsonValue::Array`.
    pub type Members<'a> = ::std::slice::Iter<'a, super::JsonValue>;

    /// Mutable iterator over members of `JsonValue::Array`.
    pub type MembersMut<'a> = ::std::slice::IterMut<'a, super::JsonValue>;

    /// Iterator over key value pairs of `JsonValue::Object`.
    pub type Entries<'a> = super::object::Iter<'a>;

    /// Mutable iterator over key value pairs of `JsonValue::Object`.
    pub type EntriesMut<'a> = super::object::IterMut<'a>;
}

#[deprecated(since="0.9.0", note="use `json::Error` instead")]
pub use Error as JsonError;

#[deprecated(since="0.9.0", note="use `json::Result` instead")]
pub use crate::Result as JsonResult;

pub use parser::parse;

pub type Array = Vec<JsonValue>;

/// Convenience for `JsonValue::from(value)`
pub fn from<T>(value: T) -> JsonValue where T: Into<JsonValue> {
    value.into()
}

/// Pretty prints out the value as JSON string.
pub fn stringify<T>(root: T) -> String where T: Into<JsonValue> {
    let root: JsonValue = root.into();
    root.dump()
}

/// Pretty prints out the value as JSON string. Second argument is a
/// number of spaces to indent new blocks with.
pub fn stringify_pretty<T>(root: T, spaces: u16) -> String where T: Into<JsonValue> {
    let root: JsonValue = root.into();
    root.pretty(spaces)
}

/// Helper macro for creating instances of `JsonValue::Array`.
///
/// ```
/// # #[macro_use] extern crate json;
/// # fn main() {
/// let data = array!["foo", 42, false];
///
/// assert_eq!(data[0], "foo");
/// assert_eq!(data[1], 42);
/// assert_eq!(data[2], false);
///
/// assert_eq!(data.dump(), r#"["foo",42,false]"#);
/// # }
/// ```
#[macro_export]
macro_rules! array {
    [] => ($crate::JsonValue::new_array());

    // Handles for token tree items
    [@ITEM($( $i:expr, )*) $item:tt, $( $cont:tt )+] => {
        $crate::array!(
            @ITEM($( $i, )* $crate::value!($item), )
            $( $cont )*
        )
    };
    (@ITEM($( $i:expr, )*) $item:tt,) => ({
        $crate::array!(@END $( $i, )* $crate::value!($item), )
    });
    (@ITEM($( $i:expr, )*) $item:tt) => ({
        $crate::array!(@END $( $i, )* $crate::value!($item), )
    });

    // Handles for expression items
    [@ITEM($( $i:expr, )*) $item:expr, $( $cont:tt )+] => {
        $crate::array!(
            @ITEM($( $i, )* $crate::value!($item), )
            $( $cont )*
        )
    };
    (@ITEM($( $i:expr, )*) $item:expr,) => ({
        $crate::array!(@END $( $i, )* $crate::value!($item), )
    });
    (@ITEM($( $i:expr, )*) $item:expr) => ({
        $crate::array!(@END $( $i, )* $crate::value!($item), )
    });

    // Construct the actual array
    (@END $( $i:expr, )*) => ({
        let size = 0 $( + {let _ = &$i; 1} )*;
        let mut array = Vec::with_capacity(size);

        $(
            array.push($i.into());
        )*

        $crate::JsonValue::Array(array)
    });

    // Entry point to the macro
    ($( $cont:tt )+) => {
        $crate::array!(@ITEM() $($cont)*)
    };
}

#[macro_export]
/// Helper crate for converting types into `JsonValue`. It's used
/// internally by the `object!` and `array!` macros.
macro_rules! value {
    ( null ) => { $crate::Null };
    ( [$( $token:tt )*] ) => {
        // 10
        $crate::array![ $( $token )* ]
    };
    ( {$( $token:tt )*} ) => {
        $crate::object!{ $( $token )* }
    };
    { $value:expr } => { $value };
}

/// Helper macro for creating instances of `JsonValue::Object`.
///
/// ```
/// # #[macro_use] extern crate json;
/// # fn main() {
/// let data = object!{
///     foo: 42,
///     bar: false,
/// };
///
/// assert_eq!(data["foo"], 42);
/// assert_eq!(data["bar"], false);
///
/// assert_eq!(data.dump(), r#"{"foo":42,"bar":false}"#);
/// # }
/// ```
#[macro_export]
macro_rules! object {
    // Empty object.
    {} => ($crate::JsonValue::new_object());

    // Handles for different types of keys
    (@ENTRY($( $k:expr => $v:expr, )*) $key:ident: $( $cont:tt )*) => {
        $crate::object!(@ENTRY($( $k => $v, )*) stringify!($key) => $($cont)*)
    };
    (@ENTRY($( $k:expr => $v:expr, )*) $key:literal: $( $cont:tt )*) => {
        $crate::object!(@ENTRY($( $k => $v, )*) $key => $($cont)*)
    };
    (@ENTRY($( $k:expr => $v:expr, )*) [$key:expr]: $( $cont:tt )*) => {
        $crate::object!(@ENTRY($( $k => $v, )*) $key => $($cont)*)
    };

    // Handles for token tree values
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:tt, $( $cont:tt )+) => {
        $crate::object!(
            @ENTRY($( $k => $v, )* $key => $crate::value!($value), )
            $( $cont )*
        )
    };
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:tt,) => ({
        $crate::object!(@END $( $k => $v, )* $key => $crate::value!($value), )
    });
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:tt) => ({
        $crate::object!(@END $( $k => $v, )* $key => $crate::value!($value), )
    });

    // Handles for expression values
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:expr, $( $cont:tt )+) => {
        $crate::object!(
            @ENTRY($( $k => $v, )* $key => $crate::value!($value), )
            $( $cont )*
        )
    };
    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:expr,) => ({
        $crate::object!(@END $( $k => $v, )* $key => $crate::value!($value), )
    });

    (@ENTRY($( $k:expr => $v:expr, )*) $key:expr => $value:expr) => ({
        $crate::object!(@END $( $k => $v, )* $key => $crate::value!($value), )
    });

    // Construct the actual object
    (@END $( $k:expr => $v:expr, )*) => ({
        let size = 0 $( + {let _ = &$k; 1} )*;
        let mut object = $crate::object::Object::with_capacity(size);

        $(
            object.insert($k, $v.into());
        )*

        $crate::JsonValue::Object(object)
    });

    // Entry point to the macro
    ($key:tt: $( $cont:tt )+) => {
        $crate::object!(@ENTRY() $key: $($cont)*)
    };

    // Legacy macro
    ($( $k:expr => $v:expr, )*) => {
        $crate::object!(@END $( $k => $crate::value!($v), )*)
    };
    ($( $k:expr => $v:expr ),*) => {
        $crate::object!(@END $( $k => $crate::value!($v), )*)
    };
}
