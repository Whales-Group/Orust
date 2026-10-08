use orust_syntax::parse;

#[test]
fn emits_doc_comments_as_rustdoc_with_canonical_sections() {
    let program = parse(
        "/// An invoice.\n///\n/// @param amount cents\n/// @returns the invoice\n/// @author Ada Lovelace\nexport void invoice(int amount) { return; }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("/// An invoice."));
    assert!(output.contains("/// # Arguments"));
    assert!(output.contains("/// - `amount`: cents"));
    assert!(output.contains("/// # Returns"));
    assert!(output.contains("/// the invoice"));
    assert!(output.contains("/// # Authors"));
    assert!(output.contains("/// - Ada Lovelace"));
    assert!(output.find("# Arguments").unwrap() < output.find("# Returns").unwrap());
    assert!(output.find("# Returns").unwrap() < output.find("# Authors").unwrap());
}

#[test]
fn rewrites_doc_links_through_the_rust_name_bridge() {
    let program = parse(
        "/// Invoice\n/// @see [Invoice.applyDiscount]\nexport @rustName(\"BillingInvoice\") class Invoice {}",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(
        output.contains("[`BillingInvoice::apply_discount`]"),
        "{output}"
    );
}

#[test]
fn rewrites_class_method_links_with_balanced_rustdoc_code_spans() {
    let program = parse(
        "export class Invoice { void applyDiscount() {}\n /// Applies discounts.\n /// @deprecated since 0.3.0 Use [applyDiscount].\n void discount() {} }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("[`Invoice::applyDiscount`]"), "{output}");
    assert!(
        output.contains("note = \"Use Invoice::applyDiscount.\"")
            || output.contains("note = \"Use Invoice::applyDiscount\"")
    );
}

#[test]
fn comments_do_not_change_non_comment_rust_output() {
    let plain = parse("export void run(int value) { print(value); }").unwrap();
    let commented = parse(
        "/// Run the operation.\n/// @param value input\nexport void run(int value) { /* inline */ print(value); }",
    )
    .unwrap();
    let strip_docs = |value: String| {
        value
            .lines()
            .filter(|line| !line.trim_start().starts_with("///"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(
        strip_docs(orust_emit::emit(&plain)),
        strip_docs(orust_emit::emit(&commented))
    );
}

#[test]
fn emits_deprecated_and_internal_doc_metadata() {
    let program = parse(
        "/// Old API\n/// @deprecated since 0.3.0 Use `new_api`.\nvoid old_api() {}\n/// Internal API\n/// @internal\nvoid hidden_api() {}",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("#[deprecated(since = \"0.3.0\", note = \"Use `new_api`.\")]"));
    assert!(output.contains("#[doc(hidden)]"));
}

#[test]
fn emits_version_and_legal_doc_metadata() {
    let program = parse(
        "/// API summary.\n/// @version 1.2.0\n/// @license MIT\n/// @copyright 2026 Example\nvoid api() {}",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("/// *Version 1.2.0*"));
    assert!(output.contains("/// License: MIT"));
    assert!(output.contains("/// Copyright: 2026 Example"));
}

#[test]
fn emits_inherited_interface_method_documentation() {
    let program = parse(
        "interface Greeter { /// Greets a person.\n /// @param name the person\n String greet(String name); } class Friendly implements Greeter { /// @inheritDoc\n String greet(String name) { return name; } }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("/// Greets a person."), "{output}");
    assert!(output.contains("/// - `name`: the person"), "{output}");
    assert!(!output.contains("@inheritDoc"), "{output}");
}

#[test]
fn optional_comment_emission_keeps_plain_comments_near_items() {
    let program = parse("// User storage.\nclass User { // name field\n String name; }").unwrap();
    let without = orust_emit::emit(&program);
    let with = orust_emit::emit_with_comments(&program);
    assert!(!without.contains("// User storage."), "{without}");
    assert!(with.contains("// User storage."), "{with}");
    assert!(with.contains("// name field"), "{with}");
}
