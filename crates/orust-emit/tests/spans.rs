#[test]
fn emitted_items_have_source_span_mappings() {
    let source = "class Box { int size = 1; }\nvoid main() { print(1); }";
    let program = orust_syntax::parse(source).unwrap();
    let generated = orust_emit::emit_with_spans(&program);
    assert_eq!(generated.spans.len(), 2);
    assert!(generated
        .spans
        .iter()
        .all(|mapping| mapping.generated_start < mapping.generated_end));
    assert_eq!(generated.spans[0].source_span, program.items[0].span);
    assert_eq!(generated.spans[1].source_span, program.items[1].span);
}
