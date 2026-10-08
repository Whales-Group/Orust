#[test]
fn infers_a_shared_lifetime_for_borrowed_return() {
    let program = orust_syntax::parse("lend(a) lend int first(lend int a) { return a; }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("fn first<'a>(a: &'a i64) -> &'a i64"));
}

#[test]
fn emits_raw_rust_passthrough() {
    let program = orust_syntax::parse("void main() { rust { let x: i32 = 1; } }").unwrap();
    assert!(orust_emit::emit(&program).contains("let x: i32 = 1;"));
}
