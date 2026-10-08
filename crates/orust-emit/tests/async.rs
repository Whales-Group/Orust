#[test]
fn emits_basic_async_program() {
    let input = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/cases/async-basic.or"
    ))
    .unwrap();
    let program = orust_syntax::parse(&input).unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("async fn fetch() -> i64"));
    assert!(output.contains("#[tokio::main]"));
    assert!(output.contains("orust_runtime::sleep(orust_runtime::seconds(1)).await"));
}

#[test]
fn emits_async_trait_interfaces() {
    let source = "interface Fetch { Future<int> get() async; } class Loader implements Fetch { int get() async { return 1; } }";
    let program = orust_syntax::parse(source).unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("#[async_trait]"));
    assert!(output.contains("trait Fetch"));
    assert!(output.contains("async fn get(&self) -> i64;"));
    assert!(output.contains("#[async_trait]\nimpl Fetch for Loader"));
}
