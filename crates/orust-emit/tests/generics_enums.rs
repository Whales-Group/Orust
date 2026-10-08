#[test]
fn emits_generic_classes_functions_and_bounds() {
    let source =
        "class Box<T extends Clone> { T value; } int identity<T>(T value) { return value; }";
    let program = orust_syntax::parse(source).unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("struct Box<T: Clone>"));
    assert!(output.contains("impl<T: Clone> Box<T>"));
    assert!(output.contains("fn identity<T>(value: T) -> i64"));
}

#[test]
fn infers_clone_bound_for_copying_a_generic_value() {
    let source = "T duplicate<T>(T value) { return copy value; }";
    let program = orust_syntax::parse(source).unwrap();
    let output = orust_emit::emit(&program);
    assert!(
        output.contains("fn duplicate<T: Clone>(value: T) -> T"),
        "{output}"
    );
}

#[test]
fn emits_data_enums_switch_and_if_let_patterns() {
    let source = "enum Maybe<T> { Some(T), None } void inspect(Maybe<int> value) { switch (value) { case Some(item): { print(item); } default: { print(0); } } if (value case Some(item)) { print(item); } }";
    let program = orust_syntax::parse(source).unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("enum Maybe<T>"));
    assert!(output.contains("Some(T)"));
    assert!(output.contains("match value"));
    assert!(output.contains("if let Maybe::Some(item) = value"));
}
