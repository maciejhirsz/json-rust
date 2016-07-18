#[macro_use]
extern crate json;

mod unit {
    use super::json;

    use std::f64;
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use json::number::Number;
    use json::{ stringify, stringify_pretty, parse, JsonValue, JsonError, Null };

    #[test]
    fn is_as_string() {
        let string = JsonValue::from("foo");

        assert!(string.is_string());
        assert_eq!(string.as_str().unwrap(), "foo");
    }

    #[test]
    fn is_as_number() {
        let number = JsonValue::from(42);

        assert!(number.is_number());
        assert_eq!(number.as_f64().unwrap(), 42.0f64);
        assert_eq!(number.as_f32().unwrap(), 42.0f32);
        assert_eq!(number.as_u64().unwrap(), 42u64);
        assert_eq!(number.as_u32().unwrap(), 42u32);
        assert_eq!(number.as_u16().unwrap(), 42u16);
        assert_eq!(number.as_u8().unwrap(), 42u8);
        assert_eq!(number.as_usize().unwrap(), 42usize);
        assert_eq!(number.as_i64().unwrap(), 42i64);
        assert_eq!(number.as_i32().unwrap(), 42i32);
        assert_eq!(number.as_i16().unwrap(), 42i16);
        assert_eq!(number.as_i8().unwrap(), 42i8);
        assert_eq!(number.as_isize().unwrap(), 42isize);

        let number = JsonValue::from(-1);

        assert_eq!(number.as_u64(), None);
        assert_eq!(number.as_u32(), None);
        assert_eq!(number.as_u16(), None);
        assert_eq!(number.as_u8(), None);
        assert_eq!(number.as_usize(), None);
        assert_eq!(number.as_i64(), Some(-1));
        assert_eq!(number.as_i32(), Some(-1));
        assert_eq!(number.as_i16(), Some(-1));
        assert_eq!(number.as_i8(), Some(-1));
        assert_eq!(number.as_isize(), Some(-1));

        let number = JsonValue::from(40_000);

        assert_eq!(number.as_u8(), None);
        assert_eq!(number.as_u16(), Some(40_000));
        assert_eq!(number.as_i8(), None);
        assert_eq!(number.as_i16(), None);
        assert_eq!(number.as_i32(), Some(40_000));
    }

    #[test]
    fn is_as_boolean() {
        let boolean = JsonValue::Boolean(true);

        assert!(boolean.is_boolean());
        assert_eq!(boolean.as_bool().unwrap(), true);
    }

    #[test]
    fn is_true() {
        let boolean = JsonValue::Boolean(true);

        assert_eq!(boolean, true);
    }

    #[test]
    fn is_false() {
        let boolean = JsonValue::Boolean(false);

        assert_eq!(boolean, false);
    }

    #[test]
    fn is_null() {
        let null = JsonValue::Null;

        assert!(null.is_null());
    }

    #[test]
    fn is_empty() {
        assert!(Null.is_empty());
        assert!(json::from(0).is_empty());
        assert!(json::from("").is_empty());
        assert!(json::from(false).is_empty());
        assert!(array![].is_empty());
        assert!(object!{}.is_empty());

        assert!(!json::from(1).is_empty());
        assert!(!json::from("foo").is_empty());
        assert!(!json::from(true).is_empty());
        assert!(!array![0].is_empty());
        assert!(!object!{ "foo" => false }.is_empty());
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
        assert_eq!(stringify(3.141592653589793), "3.141592653589793");
    }

    #[test]
    fn stringify_precise_positive_number() {
        assert_eq!(JsonValue::from(1.2345f64).dump(), "1.2345");
    }

    #[test]
    fn stringify_precise_negative_number() {
        assert_eq!(JsonValue::from(-1.2345f64).dump(), "-1.2345");
    }

    #[test]
    fn stringify_zero() {
        assert_eq!(JsonValue::from(0.0).dump(), "0");
    }

    #[test]
    fn stringify_nan() {
        assert_eq!(JsonValue::from(f64::NAN).dump(), "null");
    }

    #[test]
    fn stringify_infinity() {
        assert_eq!(JsonValue::from(f64::INFINITY).dump(), "null");
        assert_eq!(JsonValue::from(f64::NEG_INFINITY).dump(), "null");
    }

    #[test]
    fn stringify_negative_zero() {
        assert_eq!(JsonValue::from(-0f64).dump(), "-0");
    }

    #[test]
    fn stringify_integer() {
        assert_eq!(stringify(42), "42");
    }

    #[test]
    fn stringify_small_number() {
        assert_eq!(stringify(0.0001), "0.0001");
    }

    #[test]
    fn stringify_large_number() {
        assert_eq!(stringify(1e19), "10000000000000000000");
    }

    #[test]
    fn stringify_very_large_number() {
        assert_eq!(stringify(3.141592653589793e50), "3.141592653589793e50");
    }

    #[test]
    fn stringify_very_small_number() {
        assert_eq!(stringify(3.141592653589793e-16), "3.141592653589793e-16");
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

        assert_eq!(stringify(array), r#"[10,"Foo"]"#);
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

        assert_eq!(stringify(object), r#"{"name":"Maciej","age":30}"#);
    }

    #[test]
    fn stringify_raw_object() {
        let mut object = json::object::Object::new();

        object.insert("name", "Maciej".into());
        object.insert("age", 30.into());

        assert_eq!(stringify(object), r#"{"name":"Maciej","age":30}"#);
    }

    #[test]
    fn stringify_btree_map() {
        let mut map = BTreeMap::new();

        map.insert("name".into(), "Maciej".into());
        map.insert("age".into(), 30.into());

        // BTreeMap will sort keys
        assert_eq!(stringify(map), r#"{"age":30,"name":"Maciej"}"#);
    }

    #[test]
    fn stringify_hash_map() {
        let mut map = HashMap::new();

        map.insert("name".into(), "Maciej".into());
        map.insert("age".into(), 30.into());

        // HashMap does not sort keys, but depending on hashing used the
        // order can be different. Safe bet is to parse the result and
        // compare parsed objects.
        let parsed = parse(&stringify(map)).unwrap();

        assert_eq!(parsed, object!{
            "name" => "Maciej",
            "age" => 30
        });
    }

    #[test]
    fn stringify_object_with_put() {
        let mut object = JsonValue::new_object();

        object["a"] = 100.into();
        object["b"] = false.into();

        assert_eq!(stringify(object), r#"{"a":100,"b":false}"#);
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
    fn stringify_escaped_characters() {
        assert_eq!(stringify("\r____\n___\t\u{8}\u{c}\\\"__"), r#""\r____\n___\t\b\f\\\"__""#);
    }

    #[test]
    fn stringify_dont_escape_forward_slash() {
        assert_eq!(stringify("foo/bar"), r#""foo/bar""#);
    }

    #[test]
    fn stringify_escaped() {
        assert_eq!(stringify("http://www.google.com/\t"), r#""http://www.google.com/\t""#);
    }

    #[test]
    fn stringify_control_escaped() {
        assert_eq!(stringify("foo\u{1f}bar\u{0}baz"), r#""foo\u001fbar\u0000baz""#);
    }

    #[test]
    fn stringify_pretty_object() {
        let object = object!{
            "name" => "Urlich",
            "age" => 50,
            "parents" => object!{
                "mother" => "Helga",
                "father" => "Brutus"
            },
            "cars" => array![ "Golf", "Mercedes", "Porsche" ]
        };

        assert_eq!(stringify_pretty(object, 2),
                   "{\n  \"name\": \"Urlich\",\n  \"age\": 50,\n  \"parents\": {\n    \"mother\": \"Helga\",\n    \"father\": \"Brutus\"\n  },\n  \"cars\": [\n    \"Golf\",\n    \"Mercedes\",\n    \"Porsche\"\n  ]\n}");
    }

    #[test]
    fn parse_true() {
        assert_eq!(parse("true").unwrap(), true);
    }

    #[test]
    fn parse_false() {
        assert_eq!(parse("false").unwrap(), false);
    }

    #[test]
    fn parse_null() {
        assert!(parse("null").unwrap().is_null());
    }

    #[test]
    fn parse_number() {
        assert_eq!(parse("3.141592653589793").unwrap(), 3.141592653589793);
    }

    #[test]
    fn parse_small_number() {
        assert_eq!(parse("0.05").unwrap(), 0.05);
    }

    #[test]
    fn parse_very_long_float() {
        assert_eq!(parse("2.22507385850720113605740979670913197593481954635164564e-308").unwrap(), 2.225073858507201e-308);
    }

    #[test]
    fn parse_integer() {
        assert_eq!(parse("42").unwrap(), 42);
    }

    #[test]
    fn parse_negative_zero() {
        assert_eq!(parse("-0").unwrap(), JsonValue::from(-0f64));
    }

    #[test]
    fn parse_negative_integer() {
        assert_eq!(parse("-42").unwrap(), -42);
    }

    #[test]
    fn parse_number_with_leading_zero() {
        assert!(parse("01").is_err());
    }

    #[test]
    fn parse_negative_number_with_leading_zero() {
        assert!(parse("-01").is_err());
    }

    #[test]
    fn parse_number_with_e() {
        assert_eq!(parse("5e2").unwrap(), 500);
        assert_eq!(parse("5E2").unwrap(), 500);
    }

    #[test]
    fn parse_number_with_positive_e() {
        assert_eq!(parse("5e+2").unwrap(), 500);
        assert_eq!(parse("5E+2").unwrap(), 500);
    }

    #[test]
    fn parse_number_with_negative_e() {
        assert_eq!(parse("5e-2").unwrap(), 0.05);
        assert_eq!(parse("5E-2").unwrap(), 0.05);
    }

    #[test]
    fn parse_number_with_invalid_e() {
        assert!(parse("0e").is_err());
    }

    #[test]
    fn parse_large_number() {
        assert_eq!(parse("18446744073709551616").unwrap(), 18446744073709552000f64);
    }

    #[test]
    fn parse_array() {
        assert_eq!(parse(r#"[10, "foo", true, null]"#).unwrap(), array![
            10,
            "foo",
            true,
            Null
        ]);
    }

    #[test]
    fn parse_object() {
        assert_eq!(parse(r#"

        {
            "foo": "bar",
            "num": 10
        }

        "#).unwrap(), object!{
            "foo" => "bar",
            "num" => 10
        });
    }

    #[test]
    fn parse_object_with_array(){
        assert_eq!(parse(r#"

        {
            "foo": [1, 2, 3]
        }

        "#).unwrap(), object!{
            "foo" => array![1, 2, 3]
        });
    }

    #[test]
    fn parse_nested_object() {
        assert_eq!(parse(r#"

        {
            "l10n": [ {
                "product": {
                    "inStock": {
                        "DE": "Lieferung innerhalb von 1-3 Werktagen"
                    }
                }
            } ]
        }

        "#).unwrap(), object!{
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
    fn parse_and_index_from_object() {
        let data = parse("{ \"pi\": 3.14 }").unwrap();
        let ref pi = data["pi"];

        assert_eq!(pi, 3.14);
    }

    #[test]
    fn parse_and_index_mut_from_object() {
        let mut data = parse(r#"

        {
            "foo": 100
        }

        "#).unwrap();

        assert_eq!(data["foo"], 100);

        data["foo"] = 200.into();

        assert_eq!(data["foo"], 200);
    }

    #[test]
    fn parse_and_index_mut_from_null() {
        let mut data = parse("null").unwrap();

        assert!(data["foo"]["bar"].is_null());

        // test that data didn't coerece to object
        assert!(data.is_null());

        data["foo"]["bar"] = 100.into();

        assert!(data.is_object());
        assert_eq!(data["foo"]["bar"], 100);

        assert_eq!(data.dump(), r#"{"foo":{"bar":100}}"#);
    }

    #[test]
    fn parse_and_index_from_array() {
        let data = parse(r#"[100, 200, false, null, "foo"]"#).unwrap();

        assert_eq!(data[0], Number::from(100));
        assert_eq!(data[1], 200);
        assert_eq!(data[2], false);
        assert_eq!(data[3], Null);
        assert_eq!(data[4], "foo");
        assert_eq!(data[5], Null);
    }

    #[test]
    fn parse_and_index_mut_from_array() {
        let mut data = parse(r#"[100, 200, false, null, "foo"]"#).unwrap();

        assert!(data[3].is_null());
        assert!(data[5].is_null());

        data[3] = "modified".into();
        data[5] = "implicid push".into();

        assert_eq!(data[3], "modified");
        assert_eq!(data[5], "implicid push");
    }

    #[test]
    fn parse_escaped_characters() {
        let data = parse(r#"

        "\r\n\t\b\f\\\/\""

        "#).unwrap();

        assert!(data.is_string());
        assert_eq!(data, "\r\n\t\u{8}\u{c}\\/\"");
    }

    #[test]
    fn parse_escaped_unicode() {
        let data = parse(r#"

        "\u2764\ufe0f"

        "#).unwrap();

        assert_eq!(data, "❤️");
    }

    #[test]
    fn parse_escaped_unicode_surrogate() {
        let data = parse(r#"

        "\uD834\uDD1E"

        "#).unwrap();

        assert_eq!(data, "𝄞");
    }

    #[test]
    fn parse_escaped_unicode_surrogate_fail() {
        let err = parse(r#"

        "\uD834 \uDD1E"

        "#);

        assert!(err.is_err());
    }

    #[test]
    fn array_len() {
        let data = array![0, 1, 2, 3];

        assert_eq!(data.len(), 4);
    }

    #[test]
    fn array_contains() {
        let data = array![true, Null, 3.14, "foo"];

        assert!(data.contains(true));
        assert!(data.contains(Null));
        assert!(data.contains(3.14));
        assert!(data.contains("foo"));

        assert!(!data.contains(false));
        assert!(!data.contains(42));
        assert!(!data.contains("bar"));
    }

    #[test]
    fn array_push() {
        let mut data = array![1, 2];

        data.push(3).unwrap();

        assert_eq!(data, array![1, 2, 3]);
    }

    #[test]
    fn array_pop() {
        let mut data = array![1, 2, 3];

        assert_eq!(data.pop(), 3);
        assert_eq!(data, array![1, 2]);
    }

    #[test]
    fn array_members() {
        let data = array![1, "foo"];

        for member in data.members() {
            assert!(!member.is_null());
        }

        let mut members = data.members();

        assert_eq!(members.next().unwrap(), 1);
        assert_eq!(members.next().unwrap(), "foo");
        assert!(members.next().is_none());
    }

    #[test]
    fn array_members_rev() {
        let data = array![1, "foo"];

        for member in data.members() {
            assert!(!member.is_null());
        }

        let mut members = data.members().rev();

        assert_eq!(members.next().unwrap(), "foo");
        assert_eq!(members.next().unwrap(), 1);
        assert!(members.next().is_none());
    }

    #[test]
    fn array_members_mut() {
        let mut data = array![Null, Null];

        for member in data.members_mut() {
            assert!(member.is_null());
            *member = 100.into();
        }

        assert_eq!(data, array![100, 100]);
    }

    #[test]
    fn array_members_mut_rev() {
        let mut data = array![Null, Null];
        let mut item = 100;

        for member in data.members_mut().rev() {
            assert!(member.is_null());
            *member = item.into();
            item += 1;
        }

        assert_eq!(data, array![item - 1, item - 2]);
    }

    #[test]
    fn object_len() {
        let data = object!{
            "a" => true,
            "b" => false
        };

        assert_eq!(data.len(), 2);
    }

    #[test]
    fn object_remove() {
        let mut data = object!{
            "foo" => "bar",
            "answer" => 42
        };

        assert_eq!(data.remove("foo"), "bar");
        assert_eq!(data, object!{ "answer" => 42 });
    }

    #[test]
    fn object_entries() {
        let data = object!{
            "a" => 1,
            "b" => "foo"
        };

        for (_, value) in data.entries() {
            assert!(!value.is_null());
        }

        let mut entries = data.entries();

        let (key, value) = entries.next().unwrap();
        assert_eq!(key, "a");
        assert_eq!(value, 1);

        let (key, value) = entries.next().unwrap();
        assert_eq!(key, "b");
        assert_eq!(value, "foo");

        assert!(entries.next().is_none());
    }

    #[test]
    fn object_entries_rev() {
        let data = object!{
            "a" => 1,
            "b" => "foo"
        };

        for (_, value) in data.entries().rev() {
            assert!(!value.is_null());
        }

        let mut entries = data.entries().rev();

        let (key, value) = entries.next().unwrap();
        assert_eq!(key, "b");
        assert_eq!(value, "foo");

        let (key, value) = entries.next().unwrap();
        assert_eq!(key, "a");
        assert_eq!(value, 1);

        assert!(entries.next().is_none());
    }

    #[test]
    fn object_entries_mut() {
        let mut data = object!{
            "a" => Null,
            "b" => Null
        };

        for (_, value) in data.entries_mut() {
            assert!(value.is_null());
            *value = 100.into();
        }

        assert_eq!(data, object!{
            "a" => 100,
            "b" => 100
        });
    }

    #[test]
    fn object_entries_mut_rev() {
        let mut data = object!{
            "a" => Null,
            "b" => Null
        };
        let mut item = 100;

        for (_, value) in data.entries_mut().rev() {
            assert!(value.is_null());
            *value = item.into();
            item += 1;
        }

        assert_eq!(data, object!{
            "a" => item - 1,
            "b" => item - 2
        });
    }

    #[test]
    fn object_dump_minified() {
        let object = object!{
            "name" => "Maciej",
            "age" => 30
        };

        assert_eq!(object.dump(), "{\"name\":\"Maciej\",\"age\":30}");
    }

    #[test]
    fn object_dump_pretty() {
        let object = object!{
            "name" => "Urlich",
            "age" => 50,
            "parents" => object!{
                "mother" => "Helga",
                "father" => "Brutus"
            },
            "cars" => array![ "Golf", "Mercedes", "Porsche" ]
        };

        assert_eq!(object.pretty(2),
                   "{\n  \"name\": \"Urlich\",\n  \"age\": 50,\n  \"parents\": {\n    \"mother\": \"Helga\",\n    \"father\": \"Brutus\"\n  },\n  \"cars\": [\n    \"Golf\",\n    \"Mercedes\",\n    \"Porsche\"\n  ]\n}");
    }

    #[test]
    fn null_len() {
        let data = json::Null;

        assert_eq!(data.len(), 0);
    }

    #[test]
    fn index_by_str() {
        let data = object!{
            "foo" => "bar"
        };

        assert_eq!(data["foo"], "bar");
    }

    #[test]
    fn index_by_string() {
        let data = object!{
            "foo" => "bar"
        };

        assert_eq!(data["foo".to_string()], "bar");
    }

    #[test]
    fn index_by_string_ref() {
        let data = object!{
            "foo" => "bar"
        };

        let key = "foo".to_string();
        let ref key_ref = key;

        assert_eq!(data[key_ref], "bar");
    }

    #[test]
    fn index_mut_by_str() {
        let mut data = object!{
            "foo" => Null
        };

        data["foo"] = "bar".into();

        assert_eq!(data["foo"], "bar");
    }

    #[test]
    fn index_mut_by_string() {
        let mut data = object!{
            "foo" => Null
        };

        data["foo".to_string()] = "bar".into();

        assert_eq!(data["foo"], "bar");
    }

    #[test]
    fn index_mut_by_string_ref() {
        let mut data = object!{
            "foo" => Null
        };

        let key = "foo".to_string();
        let ref key_ref = key;

        data[key_ref] = "bar".into();

        assert_eq!(data["foo"], "bar");
    }

    #[test]
    fn fmt_string() {
        let data: JsonValue = "foobar".into();

        assert_eq!(format!("{}", data), "foobar");
        assert_eq!(format!("{:#}", data), r#""foobar""#);
    }

    #[test]
    fn fmt_number() {
        let data: JsonValue = 42.into();

        assert_eq!(format!("{}", data), "42");
        assert_eq!(format!("{:#}", data), "42");
    }

    #[test]
    fn fmt_boolean() {
        let data: JsonValue = true.into();

        assert_eq!(format!("{}", data), "true");
        assert_eq!(format!("{:#}", data), "true");
    }

    #[test]
    fn fmt_null() {
        let data = Null;

        assert_eq!(format!("{}", data), "null");
        assert_eq!(format!("{:#}", data), "null");
    }

    #[test]
    fn fmt_array() {
        let data = array![1, true, "three"];

        assert_eq!(format!("{}", data), r#"[1,true,"three"]"#);
        assert_eq!(format!("{:#}", data), "[\n    1,\n    true,\n    \"three\"\n]");
    }

    #[test]
    fn fmt_object() {
        let data = object!{
            "foo" => "bar",
            "answer" => 42
        };

        assert_eq!(format!("{}", data), r#"{"foo":"bar","answer":42}"#);
        assert_eq!(format!("{:#}", data), "{\n    \"foo\": \"bar\",\n    \"answer\": 42\n}");
    }

    #[test]
    fn error_unexpected_character() {
        let err = parse("\n\nnulX\n").unwrap_err();

        assert_eq!(err, JsonError::UnexpectedCharacter {
            ch: 'X',
            line: 3,
            column: 4,
        });

        assert_eq!(format!("{}", err), "Unexpected character: X at (3:4)");
    }

    #[test]
    fn error_unexpected_unicode_character() {
        let err = parse("\n\nnul🦄\n").unwrap_err();

        assert_eq!(err, JsonError::UnexpectedCharacter {
            ch: '🦄',
            line: 3,
            column: 4,
        });

        assert_eq!(format!("{}", err), "Unexpected character: 🦄 at (3:4)");
    }

    #[test]
    fn error_unexpected_token() {
        let err = parse("\n  [\n    null,\n  ]  \n").unwrap_err();

        assert_eq!(err, JsonError::UnexpectedCharacter {
            ch: ']',
            line: 4,
            column: 3,
        });

        assert_eq!(format!("{}", err), "Unexpected character: ] at (4:3)");
    }

    #[test]
    fn writer_generator() {
        let data = object!{
            "foo" => array!["bar", 100, true]
        };

        let mut buf = Vec::new();

        data.to_writer(&mut buf);

        assert_eq!(String::from_utf8(buf).unwrap(), r#"{"foo":["bar",100,true]}"#);
    }
}

mod json_checker_fail {
    use super::json::parse;

    #[test]
    fn unclosed_array() {
        assert!(parse(r#"["Unclosed array""#).is_err());
    }

    #[test]
    fn unquoted_key() {
        assert!(parse(r#"{unquoted_key: "keys must be quoted"}"#).is_err());
    }

    #[test]
    fn extra_comma_arr() {
        assert!(parse(r#"["extra comma",]"#).is_err());
    }

    #[test]
    fn double_extra_comma() {
        assert!(parse(r#"["double extra comma",,]"#).is_err());
    }

    #[test]
    fn missing_value() {
        assert!(parse(r#"[   , "<-- missing value"]"#).is_err());
    }

    #[test]
    fn comma_after_close() {
        assert!(parse(r#"["Comma after the close"],"#).is_err());
    }

    #[test]
    fn extra_close() {
        assert!(parse(r#"["Extra close"]]"#).is_err());
    }

    #[test]
    fn extra_comma_obj() {
        assert!(parse(r#"{"Extra comma": true,}"#).is_err());
    }

    #[test]
    fn extra_value_after_close() {
        assert!(parse(r#"{"Extra value after close": true} "misplaced quoted value""#).is_err());
    }

    #[test]
    fn illegal_expression() {
        assert!(parse(r#"{"Illegal expression": 1 + 2}"#).is_err());
    }

    #[test]
    fn illegal_invocation() {
        assert!(parse(r#"{"Illegal invocation": alert()}"#).is_err());
    }

    #[test]
    fn numbers_cannot_have_leading_zeroes() {
        assert!(parse(r#"{"Numbers cannot have leading zeroes": 013}"#).is_err());
    }

    #[test]
    fn numbers_cannot_be_hex() {
        assert!(parse(r#"{"Numbers cannot be hex": 0x14}"#).is_err());
    }

    #[test]
    fn illegal_backslash_escape() {
        assert!(parse(r#"["Illegal backslash escape: \x15"]"#).is_err());
    }

    #[test]
    fn naked() {
        assert!(parse(r#"[\naked]"#).is_err());
    }

    #[test]
    fn illegal_backslash_escape_2() {
        assert!(parse(r#"["Illegal backslash escape: \017"]"#).is_err());
    }

    #[test]
    fn missing_colon() {
        assert!(parse(r#"{"Missing colon" null}"#).is_err());
    }

    #[test]
    fn double_colon() {
        assert!(parse(r#"{"Double colon":: null}"#).is_err());
    }

    #[test]
    fn comma_instead_of_colon() {
        assert!(parse(r#"{"Comma instead of colon", null}"#).is_err());
    }

    #[test]
    fn colon_instead_of_comma() {
        assert!(parse(r#"["Colon instead of comma": false]"#).is_err());
    }

    #[test]
    fn bad_value() {
        assert!(parse(r#"["Bad value", truth]"#).is_err());
    }

    #[test]
    fn single_quote() {
        assert!(parse(r#"['single quote']"#).is_err());
    }

    #[test]
    fn tab_character_in_string() {
        assert!(parse("[\"\ttab\tcharacter\tin\tstring\t\"]").is_err());
    }

    #[test]
    fn tab_character_in_string_esc() {
        assert!(parse("[\"tab\\\tcharacter\\\tin\\\tstring\\\t\"]").is_err());
    }

    #[test]
    fn line_break() {
        assert!(parse("[\"line\nbreak\"]").is_err());
    }

    #[test]
    fn line_break_escaped() {
        assert!(parse("[\"line\\\nbreak\"]").is_err());
    }

    #[test]
    fn no_exponent() {
        assert!(parse(r#"[0e]"#).is_err());
    }

    #[test]
    fn no_exponent_plus() {
        assert!(parse(r#"[0e+]"#).is_err());
    }

    #[test]
    fn exponent_both_signs() {
        assert!(parse(r#"[0e+-1]"#).is_err());
    }

    #[test]
    fn comma_instead_of_closing_brace() {
        assert!(parse(r#"{"Comma instead if closing brace": true,"#).is_err());
    }

    #[test]
    fn missmatch() {
        assert!(parse(r#"["mismatch"}"#).is_err());
    }
}

mod json_checker_pass {
    use super::json::parse;

    #[test]
    fn pass_1() {
        assert!(parse(r##"

        [
            "JSON Test Pattern pass1",
            {"object with 1 member":["array with 1 element"]},
            {},
            [],
            -42,
            true,
            false,
            null,
            {
                "integer": 1234567890,
                "real": -9876.543210,
                "e": 0.123456789e-12,
                "E": 1.234567890E+34,
                "":  23456789012E66,
                "zero": 0,
                "one": 1,
                "space": " ",
                "quote": "\"",
                "backslash": "\\",
                "controls": "\b\f\n\r\t",
                "slash": "/ & \/",
                "alpha": "abcdefghijklmnopqrstuvwyz",
                "ALPHA": "ABCDEFGHIJKLMNOPQRSTUVWYZ",
                "digit": "0123456789",
                "0123456789": "digit",
                "special": "`1~!@#$%^&*()_+-={':[,]}|;.</>?",
                "hex": "\u0123\u4567\u89AB\uCDEF\uabcd\uef4A",
                "true": true,
                "false": false,
                "null": null,
                "array":[  ],
                "object":{  },
                "address": "50 St. James Street",
                "url": "http://www.JSON.org/",
                "comment": "// /* <!-- --",
                "# -- --> */": " ",
                " s p a c e d " :[1,2 , 3

        ,

        4 , 5        ,          6           ,7        ],"compact":[1,2,3,4,5,6,7],
                "jsontext": "{\"object with 1 member\":[\"array with 1 element\"]}",
                "quotes": "&#34; \u0022 %22 0x22 034 &#x22;",
                "\/\\\"\uCAFE\uBABE\uAB98\uFCDE\ubcda\uef4A\b\f\n\r\t`1~!@#$%^&*()_+-=[]{}|;:',./<>?"
        : "A key can be any string"
            },
            0.5 ,98.6
        ,
        99.44
        ,

        1066,
        1e1,
        0.1e1,
        1e-1,
        1e00,2e+00,2e-00
        ,"rosebud"]

        "##).is_ok());
    }

    #[test]
    fn pass_2() {
        assert!(parse(r#"[[[[[[[[[[[[[[[[[[["Not too deep"]]]]]]]]]]]]]]]]]]]"#).is_ok());
    }

    #[test]
    fn pass_3() {
        assert!(parse(r#"

        {
            "JSON Test Pattern pass3": {
                "The outermost value": "must be an object or array.",
                "In this test": "It is an object."
            }
        }

        "#).is_ok());
    }
}

mod number {
    use super::json::number::Number;

    #[test]
    fn parse_small_float() {
        assert_eq!(Number::from(0.05), Number::from_parts(true, 5, -2));
    }


    #[test]
    fn parse_very_small_float() {
        assert_eq!(Number::from(5e-50), Number::from_parts(true, 5, -50));
    }

    #[test]
    fn parse_big_float() {
        assert_eq!(Number::from(500), Number::from_parts(true, 500, 0));
    }

    #[test]
    fn parse_very_big_float() {
        assert_eq!(Number::from(5e50), Number::from_parts(true, 5, 50));
    }
}
