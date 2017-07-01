extern crate json;

use json::number::Number;

#[test]
fn issue_107() {
    let n = Number::from_parts(true, 1, -32768);
    assert_eq!(format!("{}", n), "1e-32768");
}
