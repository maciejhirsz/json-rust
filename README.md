# 🦄 JSON in Rust

Parse and serialize JSON with ease.

**[Complete Documentation](http://terhix.com/doc/json/) - [Cargo](https://crates.io/crates/json) - [Repository](https://github.com/maciejhirsz/json-rust)**

## Easily access data without using structs.

```rust
#[macro_use]
extern crate json;
use json::JsonValue;

fn main() {
    let data = object!{
        "a" => "bar",
        "b" => array![1, false, "foo"]
    };

    // Quickly access values without creating structs
    assert!(data["a"].is("bar"));
    assert!(data["b"].is_array());
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

## Installation

Just add it to your `Cargo.toml` file:

```toml
[dependencies]
json = "*"
```

Then import it in your `main.rs` / `lib.rs` file:

```rust
#[macro_use]
extern crate json;
```
