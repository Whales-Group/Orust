#[test]
fn emits_throwing_async_results_and_question_marks() {
    let program =
        orust_syntax::parse("Future<int> load() async throws { return await other(); }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("async fn load() -> Result<i64, orust_runtime::Error>"));
    assert!(output.contains("other().await?"));
}

#[test]
fn lowers_async_try_catch_to_result_match() {
    let program = orust_syntax::parse(
        "Future<int> risky() async throws { return 1; } async void main() { try { await risky(); } catch (error) { print(\"failed\"); } }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("match (async {"));
    assert!(output.contains("risky().await?;"));
    assert!(output.contains("Err(error) =>"));
}
