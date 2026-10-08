#[test]
fn emits_expression_block_and_spawn_move_closures() {
    let program = orust_syntax::parse(
        "Fn<int, int> apply(Fn<int, int> transform) { return transform; } async void main() { var inc = |value| value + 1; var task = spawn |value| { print(value); }; }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("Box<dyn Fn(i64) -> i64>"), "{output}");
    assert!(output.contains("let inc = |value| value + 1;"));
    assert!(output.contains("orust_runtime::spawn(move |value| {"));
}

#[test]
fn emits_arithmetic_operator_traits_from_operator_methods() {
    let program =
        orust_syntax::parse("class Counter { int operator_add(int other) { return other; } }")
            .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("impl std::ops::Add for Counter"));
    assert!(output.contains("type Output = i64;"));
    assert!(output.contains("self.operator_add(rhs)"));
}

#[test]
fn emits_comparison_equality_and_index_traits() {
    let program = orust_syntax::parse(
        "class Values { bool operator_eq(lend Values other) { return true; } bool operator_lt(lend Values other) { return false; } lend int operator_index(int index) { rust { todo!() } } }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("impl PartialEq for Values"));
    assert!(output.contains("impl std::cmp::PartialOrd for Values"));
    assert!(output.contains("impl std::ops::Index<i64> for Values"));
}

#[test]
fn emits_named_optional_parameters_as_parameter_structs() {
    let program = orust_syntax::parse(
        r#"
        int greet(String name, String punctuation = "!") { return 1; }
        void main() { greet(name: "Jesse"); }
        "#,
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("struct greetArgs"));
    assert!(output.contains("fn greet(args: greetArgs)"));
    assert!(output.contains("let punctuation = args.punctuation.unwrap_or(\"!\".to_string())"));
    assert!(
        output.contains("greet(greetArgs { name: Some(\"Jesse\".to_string()), punctuation: None")
    );
}
