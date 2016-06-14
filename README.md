# JSON in Rust

Parse and serialize JSON with ease.

```rust
#[macro_use]
extern crate json;
use json::JsonValue;

fn main() {
    let data = object!{
        "a" => "bar",
        "b" => array![1,false,"foo"]
    };

    // Quickly access values without creating structs
    assert!(data["a"].is("bar"));
    assert!(data["b"][0].is(1));
    assert!(data["b"][1].is(false));
    assert!(data["b"][2].is("foo"));

    // Missing data defaults to null
    assert!(data["b"][3].is_null());
    assert!(data["c"].is_null());

    // Even nested data
    assert!(data["c"]["d"]["e"].is_null());

    assert_eq!(json::stringify(data), "{\"a\":\"bar\",\"b\":[1,false,\"foo\"]}");
}
```

## Serialize with `json::stringify(value)`

Primitives:

```rust
// str slices
assert_eq!(json::stringify("foobar"), "\"foobar\"");

// Owned strings
assert_eq!(json::stringify("foobar".to_string()), "\"foobar\"");

// Any number types
assert_eq!(json::stringify(42), "42");

// Booleans
assert_eq!(json::stringify(true), "true");
assert_eq!(json::stringify(false), "false");
```

Explicit `null` type `json::Null`:

```rust
assert_eq!(json::stringify(json::Null), "null");
```

Optional types:

```rust
let value: Option<String> = Some("foo".to_string());
assert_eq!(json::stringify(value), "\"foo\"");

let no_value: Option<String> = None;
assert_eq!(json::stringify(no_value), "null");
```

Vector:

```rust
let data = vec![1,2,3];
assert_eq!(json::stringify(data), "[1,2,3]");
```

Vector with optional values:

```rust
let data = vec![Some(1), None, Some(2), None, Some(3)];
assert_eq!(json::stringify(data), "[1,null,2,null,3]");
```

Pushing to arrays:

```rust
let mut data = json::JsonValue::new_array();

data.push(10);
data.push("foo");
data.push(false);

assert_eq!(json::stringify(data), "[10,\"foo\",false]");
```

Putting fields on objects:

```rust
let mut data = json::JsonValue::new_object();

data.put("answer", 42);
data.put("foo", "bar");

assert_eq!(json::stringify(data), "{\"answer\":42,\"foo\":\"bar\"}");
```

`array!` macro:

```rust
let data = array!["foo", "bar", 100, true, json::Null];
assert_eq!(json::stringify(data), "[\"foo\",\"bar\",100,true,null]");
```

`object!` macro:

```rust
let data = object!{
    "name"    => "John Doe",
    "age"     => 30,
    "canJSON" => true
};
assert_eq!(
    json::stringify(data),
    // Because object is internally using a BTreeMap,
    // the key order is alphabetical
    "{\"age\":30,\"canJSON\":true,\"name\":\"John Doe\"}"
);
```
