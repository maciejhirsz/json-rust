![](http://terhix.com/doc/json-rust-logo-small.png)

# json-rust

![](https://img.shields.io/travis/maciejhirsz/json-rust.svg)
![](https://img.shields.io/crates/v/json.svg)
![](https://img.shields.io/crates/l/json.svg)

Parse and serialize [JSON](http://json.org/) with ease.

**[Complete Documentation](http://terhix.com/doc/json/) -**
**[Cargo](https://crates.io/crates/json) -**
**[Repository](https://github.com/maciejhirsz/json-rust)**

## Why?

JSON is a very loose format where anything goes - arrays can hold mixed
types, object keys can change types between API calls or not include
some keys under some conditions. Mapping that to idiomatic Rust structs
introduces friction.

This crate intends to avoid that friction.

```rust
let parsed = json::parse(r#"

{
    "code": 200,
    "success": true,
    "payload": {
        "features": [
            "awesome",
            "easyAPI",
            "lowLearningCurve"
        ]
    }
}

"#).unwrap();

let instantiated = object!{
    "code" => 200,
    "success" => true,
    "payload" => object!{
        "features" => array![
            "awesome",
            "easyAPI",
            "lowLearningCurve"
        ]
    }
};

assert_eq!(parsed, instantiated);
```

## First class citizen

Using macros and indexing, it's easy to work with the data.

```rust
let mut data = object!{
    "foo" => false,
    "bar" => json::Null,
    "answer" => 42,
    "list" => array![json::Null, "world", true]
};

// Partial equality is implemented for most raw types:
assert!(data["foo"] == false);

// And it's type aware, `null` and `false` are different values:
assert!(data["bar"] != false);

// But you can use any Rust number types:
assert!(data["answer"] == 42);
assert!(data["answer"] == 42.0);
assert!(data["answer"] == 42isize);

// Access nested structures, arrays and objects:
assert!(data["list"][0].is_null());
assert!(data["list"][1] == "world");
assert!(data["list"][2] == true);

// Error resilient - accessing properties that don't exist yield null:
assert!(data["this"]["does"]["not"]["exist"].is_null());

// Mutate by assigning:
data["list"][0] = "Hello".into();

// Use the `dump` method to serialize the data:
assert_eq!(data.dump(), r#"{"answer":42,"bar":null,"foo":false,"list":["Hello","world",true]}"#);

// Or pretty print it out:
println!("{:#}", data);
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

## Performance

While performance is not the main goal of this crate, it is still relevant, and it's doing pretty well in the company:

![](http://terhix.com/json-perf-9.png)

[The benchmarks](https://github.com/maciejhirsz/json-rust/blob/benches/benches/log.rs) were run on 2012 MacBook Air, your results may vary. Many thanks to [@dtolnay](https://github.com/dtolnay) for providing the baseline struct and test data the tests could be run on.

While this is not necessarily a be-all end-all benchmark, the main takeaway from this is that Serde parsing is much faster when parsing to a struct, since the parser knows exactly the kind of data it needs, and doesn't pay the (re)allocation costs of pushing data to a map. Also worth noting, rustc-serialize suffers since it first has to parse JSON to generic enum-based values, and only then map those onto structs.
