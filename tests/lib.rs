#[macro_use(array, object)]
extern crate json;
use std::collections::BTreeMap;
use std::collections::HashMap;

use json::{ stringify, parse, JsonValue, Null };

#[test]
fn stringify_null() {
    assert_eq!(stringify(Null), "null");
}

#[test]
fn stringify_option_none() {
    let foo: Option<String> = None;
    assert_eq!(stringify(foo), "null");
}

#[test]
fn stringify_option_integer() {
    let foo = Some(100);
    assert_eq!(stringify(foo), "100");
}

#[test]
fn stringify_str_slice() {
    assert_eq!(stringify("Foo"), "\"Foo\"");
}

#[test]
fn stringify_string() {
    assert_eq!(stringify("Foo".to_string()), "\"Foo\"");
}

#[test]
fn stringify_number() {
    assert_eq!(stringify(3.14), "3.14");
}

#[test]
fn stringify_integer() {
    assert_eq!(stringify(42), "42");
}

#[test]
fn stringify_true() {
    assert_eq!(stringify(true), "true");
}

#[test]
fn stringify_false() {
    assert_eq!(stringify(false), "false");
}

#[test]
fn stringify_array() {
    assert_eq!(stringify(array![10, false, Null]), "[10,false,null]");
}

#[test]
fn stringify_vec() {
    let mut array = Vec::new();

    array.push(10.into());
    array.push("Foo".into());

    assert_eq!(stringify(array), "[10,\"Foo\"]");
}

#[test]
fn stringify_object() {
    let object = object!{
        "name" => "Maciej",
        "age" => 30
    };

    assert_eq!(stringify(object), "{\"age\":30,\"name\":\"Maciej\"}");
}

#[test]
fn stringify_btree_map() {
    let mut object = BTreeMap::new();

    object.insert("name".into(), "Maciej".into());
    object.insert("age".into(), 30.into());

    assert_eq!(stringify(object), "{\"age\":30,\"name\":\"Maciej\"}");
}

#[test]
fn stringify_hash_map() {
    let mut object = HashMap::new();

    object.insert("name".into(), "Maciej".into());
    object.insert("age".into(), 30.into());

    assert_eq!(stringify(object), "{\"age\":30,\"name\":\"Maciej\"}");
}

#[test]
fn stringify_object_with_put() {
    let mut object = object!{};

    object.put("a", 100);
    object.put("b", false);

    assert_eq!(stringify(object), "{\"a\":100,\"b\":false}");
}

#[test]
fn parse_true() {
    assert_eq!(parse("true"), true.into());
}

#[test]
fn parse_false() {
    assert_eq!(parse("false"), false.into());
}

#[test]
fn parse_null() {
    assert_eq!(parse("null"), Null);
}

#[test]
fn parse_number() {
    assert_eq!(parse("3.14"), 3.14.into())
}

#[test]
fn parse_integer() {
    assert_eq!(parse("42"), 42.into());
}

#[test]
fn parse_array() {
    assert_eq!(parse("[10, \"foo\", true, null]"), array![
        10,
        "foo",
        true,
        Null
    ]);
}

#[test]
fn parse_object() {
    assert_eq!(parse("

    {
        \"foo\": \"bar\",
        \"num\": 10
    }

    "), object!{
        "foo" => "bar",
        "num" => 10
    });
}

#[test]
fn parse_object_with_array(){
    assert_eq!(parse("

    {
        \"foo\": [1, 2, 3]
    }

    "), object!{
        "foo" => array![1, 2, 3]
    });
}
