use debugger_test::*;
use json::*;

#[inline(never)]
fn __break() { }

#[debugger_test(
    debugger = "cdb",
    commands = r#"
.nvlist
dx json.__0
dx -r2 json.__0["\"positive_number\""]
dx -r2 json.__0["\"negative_number\""]
dx -r2 json.__0["\"float_number\""]
dx -r3 json.__0["\"string\""]
dx -r4 json.__0["\"array\""]
    "#,
    expected_statements = r#"
json.__0         [Type: json::object::Object]
    [<Raw View>]     [Type: json::object::Object]
    ["\"positive_number\""] : Number [Type: enum2$<json::value::JsonValue>]
    ["\"negative_number\""] : Number [Type: enum2$<json::value::JsonValue>]
    ["\"float_number\""] : Number [Type: enum2$<json::value::JsonValue>]
    ["\"string\""]   : String [Type: enum2$<json::value::JsonValue>]
    ["\"array\""]    : Array [Type: enum2$<json::value::JsonValue>]
json.__0["\"positive_number\""] : Number [Type: enum2$<json::value::JsonValue>]
    [<Raw View>]     [Type: enum2$<json::value::JsonValue>]
    [+0x008] __0              : 10 [Type: json::number::Number]
        [<Raw View>]     [Type: json::number::Number]
        [category]       : 1 [Type: unsigned char]
        [exponent]       : 0 [Type: short]
        [mantissa]       : 10 [Type: unsigned __int64]
json.__0["\"negative_number\""] : Number [Type: enum2$<json::value::JsonValue>]
    [<Raw View>]     [Type: enum2$<json::value::JsonValue>]
    [+0x008] __0              : -250 [Type: json::number::Number]
        [<Raw View>]     [Type: json::number::Number]
        [category]       : 0 [Type: unsigned char]
        [exponent]       : 0 [Type: short]
        [mantissa]       : 250 [Type: unsigned __int64]
json.__0["\"float_number\""] : Number [Type: enum2$<json::value::JsonValue>]
    [<Raw View>]     [Type: enum2$<json::value::JsonValue>]
    [+0x008] __0              : 19.210000 [Type: json::number::Number]
        [<Raw View>]     [Type: json::number::Number]
        [category]       : 1 [Type: unsigned char]
        [exponent]       : -2 [Type: short]
        [mantissa]       : 1921 [Type: unsigned __int64]
json.__0["\"string\""] : String [Type: enum2$<json::value::JsonValue>]
    [<Raw View>]     [Type: enum2$<json::value::JsonValue>]
    [+0x008] __0              : "Loooooooooooooooooooooooooooong String" [Type: alloc::string::String]
        [<Raw View>]     [Type: alloc::string::String]
        [len]            : 0x26 [Type: unsigned __int64]
        [capacity]       : 0x26 [Type: unsigned __int64]
        [chars]          : "Loooooooooooooooooooooooooooong String"
            [0]              : 76 'L' [Type: char]
            [1]              : 111 'o' [Type: char]
            [2]              : 111 'o' [Type: char]
            [3]              : 111 'o' [Type: char]
            [4]              : 111 'o' [Type: char]
            [5]              : 111 'o' [Type: char]
            [6]              : 111 'o' [Type: char]
            [7]              : 111 'o' [Type: char]
            [8]              : 111 'o' [Type: char]
            [9]              : 111 'o' [Type: char]
            [10]             : 111 'o' [Type: char]
            [11]             : 111 'o' [Type: char]
            [12]             : 111 'o' [Type: char]
            [13]             : 111 'o' [Type: char]
            [14]             : 111 'o' [Type: char]
            [15]             : 111 'o' [Type: char]
            [16]             : 111 'o' [Type: char]
            [17]             : 111 'o' [Type: char]
            [18]             : 111 'o' [Type: char]
            [19]             : 111 'o' [Type: char]
            [20]             : 111 'o' [Type: char]
            [21]             : 111 'o' [Type: char]
            [22]             : 111 'o' [Type: char]
            [23]             : 111 'o' [Type: char]
            [24]             : 111 'o' [Type: char]
            [25]             : 111 'o' [Type: char]
            [26]             : 111 'o' [Type: char]
            [27]             : 111 'o' [Type: char]
            [28]             : 111 'o' [Type: char]
            [29]             : 110 'n' [Type: char]
            [30]             : 103 'g' [Type: char]
            [31]             : 32 ' ' [Type: char]
            [32]             : 83 'S' [Type: char]
            [33]             : 116 't' [Type: char]
            [34]             : 114 'r' [Type: char]
            [35]             : 105 'i' [Type: char]
            [36]             : 110 'n' [Type: char]
            [37]             : 103 'g' [Type: char]

json.__0["\"array\""] : Array [Type: enum2$<json::value::JsonValue>]
    [<Raw View>]     [Type: enum2$<json::value::JsonValue>]
    [+0x008] __0              : { len=0x3 } [Type: alloc::vec::Vec<enum2$<json::value::JsonValue>,alloc::alloc::Global>]
        [<Raw View>]     [Type: alloc::vec::Vec<enum2$<json::value::JsonValue>,alloc::alloc::Global>]
        [len]            : 0x3 [Type: unsigned __int64]
        [capacity]       : 0x3 [Type: unsigned __int64]
        [0]              : Object [Type: enum2$<json::value::JsonValue>]
            [<Raw View>]     [Type: enum2$<json::value::JsonValue>]
            [+0x008] __0              [Type: json::object::Object]
                [<Raw View>]     [Type: json::object::Object]
                ["\"short\""]    : Short [Type: enum2$<json::value::JsonValue>]
        [1]              : Boolean [Type: enum2$<json::value::JsonValue>]
            [<Raw View>]     [Type: enum2$<json::value::JsonValue>]
            [+0x001] __0              : true [Type: bool]
        [2]              : Array [Type: enum2$<json::value::JsonValue>]
            [<Raw View>]     [Type: enum2$<json::value::JsonValue>]
            [+0x008] __0              : { len=0x1 } [Type: alloc::vec::Vec<enum2$<json::value::JsonValue>,alloc::alloc::Global>]
                [<Raw View>]     [Type: alloc::vec::Vec<enum2$<json::value::JsonValue>,alloc::alloc::Global>]
                [len]            : 0x1 [Type: unsigned __int64]
                [capacity]       : 0x1 [Type: unsigned __int64]
                [0]              : Null [Type: enum2$<json::value::JsonValue>]
    "#
)]
fn test_debugger_visualizer() {
    let json = object!{
        positive_number: 10,
        negative_number: -250,
        float_number: 19.21,
        string: "Loooooooooooooooooooooooooooong String",
        array: [
            object!{ short: "short string" },
            true,
            [null],
        ],
    };

    assert!(json["array"][1] == true);
    __break();
}