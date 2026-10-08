#[test]
fn emits_collection_types_and_helpers() {
    let source = "void use(List<int> items, Map<String, int> counts, Set<String> tags) { print(items.length); print(items.isEmpty); items.add(1); counts.put(\"one\", 1); tags.remove(\"old\"); }";
    let program = orust_syntax::parse(source).unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("items: Vec<i64>"));
    assert!(output.contains("counts: std::collections::HashMap<String, i64>"));
    assert!(output.contains("tags: std::collections::HashSet<String>"));
    assert!(output.contains("items.len() as i64"));
    assert!(output.contains("items.is_empty()"));
    assert!(output.contains("items.push(1)"));
    assert!(output.contains("counts.insert(\"one\".to_string(), 1)"));
}
