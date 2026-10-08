#[test]
fn emits_wait_any_and_spawn() {
    let source = "Future<int> one() async { return 1; } async void main() { var values = await Future.wait([one(), one()]); var task = spawn one(); print(await task); }";
    let program = orust_syntax::parse(source).unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("join_all(vec![one(), one()])"));
    assert!(output.contains("orust_runtime::spawn(one())"));
    assert!(output.contains("task.await_task().await"));
}
