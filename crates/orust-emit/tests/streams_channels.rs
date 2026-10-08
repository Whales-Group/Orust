#[test]
fn lowers_await_for_over_a_stream() {
    let program = orust_syntax::parse(
        "Stream<int> values() async { rust { todo!() } } async void main() { await for (var value in values()) { print(value); } }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("use orust_runtime::StreamExt;"));
    assert!(output.contains("while let Some(value) = values().next().await"));
}

#[test]
fn emits_tokio_channel_type_and_constructor() {
    let program = orust_syntax::parse("Channel<int> open() { return Channel.create(8); }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("fn open() -> orust_runtime::Channel<i64>"));
    assert!(output.contains("return orust_runtime::channel(8);"));
}
