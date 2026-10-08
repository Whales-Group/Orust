use orust_syntax::{parse, DocTag, Item};

#[test]
fn acceptance_program_preserves_docs_and_emits_rustdoc_sections() {
    let source = include_str!("../../tests/cases/phase6-acceptance.or");
    let program = parse(source).expect("acceptance program must parse");
    assert!(program.doc_blocks().iter().any(|doc| {
        doc.tags
            .iter()
            .any(|tag| matches!(tag, DocTag::Example { .. }))
    }));
    let rust = orust_emit::emit(&program);
    assert!(rust.contains("# Examples"));
    assert!(rust.contains("# Authors"));
    assert!(rust.contains("DiscountError"));
    assert!(program
        .items
        .iter()
        .any(|item| matches!(item.node, Item::Class(_))));
}

#[test]
fn unknown_doc_tag_is_preserved_and_diagnosed() {
    let program = parse("/// Summary.\n/// @authro Ada\nclass User {}").unwrap();
    assert!(program.lints.iter().any(|lint| lint.code == "OR0603"));
    let doc = program.doc_blocks().pop().unwrap();
    assert!(matches!(doc.tags[0], DocTag::Unknown { .. }));
    let rust = orust_emit::emit(&program);
    assert!(rust.contains("/// @authro Ada"), "{rust}");
}
