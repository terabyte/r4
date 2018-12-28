use record::Record;

fn test_xform(i: &str, c: &str, o: &str) {
    let r = Record::parse(i);
    let r = super::load(c)(r);
    assert_eq!(r.deparse(), o);
}

#[test]
fn test_simple() {
    test_xform("{}", r#"{{x}} = "y""#, r#"{"x":"y"}"#);
}

#[test]
fn test_reassign() {
    test_xform(r#"{"a":[1,2]}"#, r#"{{b}} = {{a}}"#, r#"{"a":[1,2],"b":[1,2]}"#);
}

#[test]
fn test_assign_hash() {
    test_xform("{}", r#"{{x}} = {a: "b"}"#, r#"{"x":{"a":"b"}}"#);
}

#[test]
fn test_array_literal() {
    test_xform("{}", r#"{{x}} = [1, "b"]"#, r#"{"x":[1,"b"]}"#);
}

#[test]
fn test_deep_index() {
    test_xform(r#"{"a":{"b":[{"c":"x"}]}}"#, r#"{{a}} = {{a/b/#0/c}}"#, r#"{"a":"x"}"#);
}

#[test]
fn test_del() {
    test_xform(r#"{"a":[{"b":"c"}]}"#, r#"{{x}} = d{{a/#0/b}}"#, r#"{"a":[{}],"x":"c"}"#);
}

#[test]
fn test_diamond() {
    test_xform("{}", r#"{{a}} = {{b}} = {}; {{a/c}} = "d""#, r#"{"a":{"c":"d"},"b":{"c":"d"}}"#);
}

#[test]
fn test_get_fill() {
    test_xform("{}", r#"{{x}} = {{a/b}}; {{x/y}} = "z""#, r#"{"x":{"y":"z"}}"#);
    test_xform("{}", r#"{{x}} = f{{a/b}}; {{x/y}} = "z""#, r#"{"a":{"b":{"y":"z"}},"x":{"y":"z"}}"#);
}

#[test]
fn test_vars() {
    test_xform("{}", r#" $a = {}; {{a:b}} = "c"; {{x}} = $a; {{r:y/z}} = $a; {{a:d/e}} = "f"; "#, r#"{"x":{"b":"c","d":{"e":"f"}},"y":{"z":{"b":"c","d":{"e":"f"}}}}"#);
}
