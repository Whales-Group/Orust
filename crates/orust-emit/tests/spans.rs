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

#[test]
fn emitted_item_mappings_are_ordered_and_inside_generated_code() {
    let source =
        "class Box { int size = 1; }\nvoid main() { print(1); }\nint answer() { return 42; }";
    let program = orust_syntax::parse(source).unwrap();
    let generated = orust_emit::emit_with_spans(&program);

    assert_eq!(generated.spans.len(), program.items.len());
    for mapping in &generated.spans {
        assert!(mapping.generated_start < mapping.generated_end);
        assert!(mapping.generated_end <= generated.code.len());
    }
    for pair in generated.spans.windows(2) {
        assert!(pair[0].generated_end <= pair[1].generated_start);
    }
}
