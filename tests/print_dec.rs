extern crate json;

use json::number::Number;

#[test]
fn issue_107() {
    let n = Number::from_parts(true, 1, -32768);
    assert_eq!(format!("{}", n), "1e-32768");
}

#[test]
fn issue_108_exponent_positive() {
    let n = Number::from_parts(true, 10_000_000_000_000_000_001, -18);
    assert_eq!(format!("{}", n), "1.0000000000000000001e+1");
}

#[test]
fn issue_108_exponent_0() {
    let n = Number::from_parts(true, 10_000_000_000_000_000_001, -19);
    assert_eq!(format!("{}", n), "1.0000000000000000001");
}
