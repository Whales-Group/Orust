#[test]
fn emits_extension_traits_and_impls() {
    let program =
        orust_syntax::parse("extension on String { int length() { return 1; } }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("trait StringExtensions"));
    assert!(output.contains("impl StringExtensions for String"));
    assert!(output.contains("fn length(&self) -> i64"));
}
