#[test]
fn emits_trait_objects_borrowed_values_lists_and_box_insertion() {
    let source = "interface Speak { String speak(); } class Dog implements Speak { String speak() { return \"woof\"; } } void use(Speak value, lend Speak borrowed, List<Speak> many) { } void main() { use(new Dog(), lend new Dog(), [new Dog()]); }";
    let program = orust_syntax::parse(source).unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("value: Box<dyn Speak>"));
    assert!(output.contains("borrowed: &dyn Speak"));
    assert!(output.contains("many: Vec<Box<dyn Speak>>"));
    assert!(
        output.contains("use(Box::new(Dog::new()), &Dog::new(), vec![Box::new(Dog::new())])"),
        "{output}"
    );
}

#[test]
fn adds_send_sync_bounds_to_spawned_interface_objects() {
    let source = "interface Speak { String speak(); } void consume(Speak value) { } void main() { spawn consume(new Dog()); }";
    let program = orust_syntax::parse(source).unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("trait Speak: Send + Sync"));
    assert!(output.contains("Box<dyn Speak + Send + Sync>"));
}
