#[test]
fn only_shared_classes_get_reference_counted_storage() {
    let shared = orust_syntax::parse("shared class Counter { int count = 0; int get() { return count; } void inc() { count = count + 1; } }").unwrap();
    let output = orust_emit::emit(&shared);
    assert!(output.contains("Rc<RefCell<CounterData>>"));
    assert!(output.contains("fn borrow(&self)"));
    assert!(output.contains("fn borrow_mut(&self)"));
    assert!(output.contains("self.0.borrow().count"));
    assert!(output.contains("self.0.borrow_mut().count"));
    let ordinary = orust_syntax::parse("class Counter { int count = 0; }").unwrap();
    assert!(!orust_emit::emit(&ordinary).contains("Rc<RefCell"));
}

#[test]
fn shared_classes_use_async_mutex_when_spawn_is_present() {
    let program = orust_syntax::parse(
        "shared class Counter { int count = 0; int get() async { return count; } } async void main() { var counter = new Counter(); spawn counter.get(); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("Arc<Mutex<CounterData>>"));
    assert!(output.contains("use orust_runtime::tokio::sync::Mutex;"));
    assert!(output.contains("async fn get(&self"));
    assert!(output.contains("self.0.lock().await.count"));
    assert!(!output.contains("Rc<RefCell<CounterData>>"));
}
