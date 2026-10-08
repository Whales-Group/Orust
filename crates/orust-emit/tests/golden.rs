use std::fs;

#[test]
fn hello_emits_expected_rust() {
    let input = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/cases/hello.or"
    ))
    .unwrap();
    let expected = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/cases/hello.rs"
    ))
    .unwrap();
    let program = orust_syntax::parse(&input).unwrap();
    assert_eq!(orust_emit::emit(&program), expected);
}

#[test]
fn counter_emits_expected_rust() {
    let input = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/cases/counter.or"
    ))
    .unwrap();
    let expected = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/cases/counter.rs"
    ))
    .unwrap();
    let program = orust_syntax::parse(&input).unwrap();
    assert_eq!(orust_emit::emit(&program), expected);
}

fn assert_fixture(name: &str) {
    let input = fs::read_to_string(format!("../../tests/cases/{name}.or")).unwrap();
    let expected = fs::read_to_string(format!("../../tests/cases/{name}.rs")).unwrap();
    let program = orust_syntax::parse(&input).unwrap();
    assert_eq!(orust_emit::emit(&program), expected, "fixture {name}");
}

#[test]
fn language_feature_fixtures_emit_expected_rust() {
    for name in [
        "async-basic",
        "operators",
        "named-optional",
        "interfaces",
        "borrowing",
        "types-collections",
        "options",
        "shared",
        "passthrough",
    ] {
        assert_fixture(name);
    }
}
