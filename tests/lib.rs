#[macro_use]
extern crate json;
use std::collections::BTreeMap;
use std::collections::HashMap;

use json::{ stringify, parse, JsonValue, Null };

#[test]
fn is_as_string() {
    let string = JsonValue::String("foo".to_string());

    assert!(string.is_string());
    assert_eq!(*string.as_string().unwrap(), "foo".to_string());
}

#[test]
fn is_as_number() {
    let number = JsonValue::Number(42.0);

    assert!(number.is_number());
    assert_eq!(*number.as_number().unwrap(), 42.0);
}

#[test]
fn is_as_boolean() {
    let boolean = JsonValue::Boolean(true);

    assert!(boolean.is_boolean());
    assert_eq!(*boolean.as_boolean().unwrap(), true);
}

#[test]
fn is_true() {
    let boolean = JsonValue::Boolean(true);

    assert!(boolean.is_true());
}

#[test]
fn is_false() {
    let boolean = JsonValue::Boolean(false);

    assert!(boolean.is_false());
}

#[test]
fn is_nul() {
    let null = JsonValue::Null;

    assert!(null.is_null());
}

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
    let mut array: Vec<JsonValue> = Vec::new();

    array.push(10.into());
    array.push("Foo".into());

    assert_eq!(stringify(array), "[10,\"Foo\"]");
}

#[test]
fn stringify_typed_vec() {
    let array = vec![1, 2, 3];

    assert_eq!(stringify(array), "[1,2,3]");
}

#[test]
fn stringify_typed_opt_vec() {
    let array = vec![Some(1), None, Some(2), None, Some(3)];

    assert_eq!(stringify(array), "[1,null,2,null,3]");
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
    let mut object = JsonValue::new_object();

    object.put("a", 100).unwrap();
    object.put("b", false).unwrap();

    assert_eq!(stringify(object), "{\"a\":100,\"b\":false}");
}

#[test]
fn stringify_array_with_push() {
    let mut array = JsonValue::new_array();

    array.push(100).unwrap();
    array.push(Null).unwrap();
    array.push(false).unwrap();
    array.push(Some("foo".to_string())).unwrap();

    assert_eq!(stringify(array), "[100,null,false,\"foo\"]");
}

#[test]
fn parse_true() {
    assert_eq!(parse("true").unwrap(), true.into());
}

#[test]
fn parse_false() {
    assert_eq!(parse("false").unwrap(), false.into());
}

#[test]
fn parse_null() {
    assert_eq!(parse("null").unwrap(), Null);
}

#[test]
fn parse_number() {
    assert_eq!(parse("3.14").unwrap(), 3.14.into())
}

#[test]
fn parse_integer() {
    assert_eq!(parse("42").unwrap(), 42.into());
}

#[test]
fn parse_array() {
    assert_eq!(parse("[10, \"foo\", true, null]").unwrap(), array![
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

    ").unwrap(), object!{
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

    ").unwrap(), object!{
        "foo" => array![1, 2, 3]
    });
}

#[test]
fn parse_nested_object() {
    assert_eq!(parse("

    {
        \"l10n\": [ {
          \"product\": {
            \"inStock\": {
              \"DE\": \"Lieferung innerhalb von 1-3 Werktagen\"
            }
          }
        } ]
    }

    ").unwrap(), object!{
        "l10n" => array![ object!{
            "product" => object!{
                "inStock" => object!{
                    "DE" => "Lieferung innerhalb von 1-3 Werktagen"
                }
            }
        } ]
    });
}

#[test]
fn parse_and_get_from_object() {
    let object = parse("{ \"pi\": 3.14 }").unwrap();
    let pi = object.get("pi").unwrap();

    assert_eq!(*pi, 3.14.into());
}

#[test]
fn parse_and_index_from_object() {
    let object = parse("{ \"pi\": 3.14 }").unwrap();
    let ref pi = object["pi"];

    assert_eq!(*pi, 3.14.into());
}

#[test]
fn parse_and_get_from_array() {
    let array = parse("[100, 200, false, null, \"foo\"]").unwrap();

    assert_eq!(*array.at(0).unwrap(), 100.into());
    assert_eq!(*array.at(1).unwrap(), 200.into());
    assert_eq!(*array.at(2).unwrap(), false.into());
    assert_eq!(*array.at(3).unwrap(), Null);
    assert_eq!(*array.at(4).unwrap(), "foo".into());
}

#[test]
fn parse_and_index_from_array() {
    let array = parse("[100, 200, false, null, \"foo\"]").unwrap();

    assert_eq!(array[0], 100.into());
    assert_eq!(array[1], 200.into());
    assert_eq!(array[2], false.into());
    assert_eq!(array[3], Null);
    assert_eq!(array[4], "foo".into());
}

#[test]
fn parse_and_use_with() {
    let mut data = parse("{\"a\":{\"b\": 100}}").unwrap();

    assert_eq!(*data.with("a").with("b"), 100.into());
}

#[test]
fn parse_and_use_with_on_null() {
    let mut data = parse("null").unwrap();

    assert!(data.is_null());
    assert!(data.with("a").with("b").is_null());
    assert!(data.get("a").unwrap().is_object());
    assert!(data.get("a").unwrap().get("b").unwrap().is_null());
}

#[test]
fn parse_error() {
    assert!(parse("10 20").is_err());
}
