#[test]
fn emits_data_and_debug_derives() {
    let data = orust_syntax::parse("@data class User { String name; }").unwrap();
    let output = orust_emit::emit(&data);
    assert!(output.contains("#[derive(Debug, Clone, PartialEq)]"));

    let plain = orust_syntax::parse("class User { String name; }").unwrap();
    assert!(orust_emit::emit(&plain).contains("#[derive(Debug)]"));
}

#[test]
fn emits_top_level_rust_use_declarations() {
    let program = orust_syntax::parse(
        "rust use serde_json::Value; rust use tokio::time::sleep as pause; rust use std::fmt::{self, Display}; void main() {}",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.starts_with(
        "use serde_json::Value;\nuse tokio::time::sleep as pause;\nuse std::fmt::{self, Display};\n"
    ));
}

#[test]
fn emits_rust_type_aliases_transparently() {
    let program =
        orust_syntax::parse("export @rustType(\"serde_json::Value\") type JsonValue;").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("pub type JsonValue = serde_json::Value;"));
}

#[test]
fn emits_qualified_rust_types_in_signatures() {
    let program = orust_syntax::parse(
        "void inspect(serde_json::Value value, tokio::sync::Mutex<String> lock) {}",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(
        output.contains("fn inspect(value: serde_json::Value, lock: tokio::sync::Mutex<String>)")
    );
}

#[test]
fn emits_rust_import_type_aliases_as_use_items() {
    let program =
        orust_syntax::parse("export @rustImport(\"serde_json::Value\") type JsonValue;").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("pub use serde_json::Value as JsonValue;"));
}

#[test]
fn emits_rust_imported_function_wrappers() {
    let program =
        orust_syntax::parse("export @rustImport(\"crate::native::load\") String loadValue();")
            .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("pub fn loadValue() -> String {"));
    assert!(output.contains("return crate::native::load();"));
}

#[test]
fn preserves_result_boundaries_for_imported_throwing_functions() {
    let program =
        orust_syntax::parse("@rustImport(\"crate::native::load\") String loadValue() throws;")
            .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("-> Result<String, orust_runtime::Error>"));
    assert!(output.contains("return crate::native::load();"));
}

#[test]
fn emits_extern_rust_functions_through_the_same_boundary() {
    let program =
        orust_syntax::parse("@externRust(\"crate::native::version\") String version();").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("fn version() -> String"));
    assert!(output.contains("return crate::native::version();"));
}

#[test]
fn emits_rust_item_attributes() {
    let program = orust_syntax::parse(
        "@derive(\"Serialize\") @repr(\"C\") @cfg(\"unix\") @orustExport class Config {}",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("#[cfg(unix)]"));
    assert!(output.contains("#[repr(C)]"));
    assert!(output.contains("#[derive(Debug, Serialize)]"));
    assert!(output.contains("pub struct Config"));
}

#[test]
fn emits_ownership_boundary_metadata() {
    let program = orust_syntax::parse(
        "@borrowed(\"'a\") lend String first(lend String value) { return value; } @owned String normalize(String value) { return value; }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("ORust borrowed boundary: lifetime 'a is enforced by rustc."));
    assert!(output.contains("ORust owned boundary: Rust move semantics are authoritative."));
    assert!(output.contains("fn first<'a>(value: &'a str) -> &'a str"));
}

#[test]
fn applies_cfg_to_generated_default_parameter_structs() {
    let program =
        orust_syntax::parse("@cfg(\"unix\") void inspect(String value = \"default\") {}").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("#[cfg(unix)]"));
    assert!(output.contains("struct inspectArgs"));
    assert!(output.contains("#[cfg(unix)]\nfn inspect"));
}

#[test]
fn preserves_escaped_quotes_in_rust_attributes_and_strings() {
    let program = orust_syntax::parse(
        "@cfg(\"feature = \\\"tokio\\\"\") class Config {} void main() { print(\"say \\\"hi\\\"\"); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("#[cfg(feature = \"tokio\")]"));
    assert!(output.contains("println!(\"{}\", \"say \\\"hi\\\"\".to_string())"));
}

#[test]
fn rewrites_rust_names_in_source_references() {
    let program = orust_syntax::parse(
        "export @rustName(\"ExternalUser\") class User {} void main() { var user = new User(); print(user); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("pub struct ExternalUser"));
    assert!(output.contains("let user = ExternalUser::new()"));
}

#[test]
fn emits_rust_expressions_verbatim() {
    let program = orust_syntax::parse(
        "@rustType(\"serde_json::Value\") type JsonValue; void main() { JsonValue value = rust serde_json::json!({ \"source\": \"Cargo.toml\" }); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output
        .contains("let value: JsonValue = serde_json::json!({ \"source\": \"Cargo.toml\" });"));
}

#[test]
fn emits_rust_let_in_the_shared_function_scope() {
    let program = orust_syntax::parse(
        "void main() { rust let mut value: serde_json::Value = serde_json::json!({ \"ok\": true }); print(value); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(
        output.contains("let mut value: serde_json::Value = serde_json::json!({ \"ok\": true });")
    );
    assert!(output.contains("println!(\"{}\", value);"));
}

#[test]
fn emits_unsafe_rust_blocks_only_when_marked() {
    let program =
        orust_syntax::parse("void main() { @unsafeRust rust { native_handle.release(); } }")
            .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("unsafe {  native_handle.release();  }"));
}

#[test]
fn emits_explicit_rust_block_without_changing_its_scope() {
    let program = orust_syntax::parse("void main() { @rustBlock rust { value += 1; } }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("value += 1;"));
    assert!(!output.contains("unsafe {"));
}

#[test]
fn maps_to_string_to_display() {
    let program =
        orust_syntax::parse("class User { String toString() { return \"user\"; } }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("impl std::fmt::Display for User"));
    assert!(output.contains("write!(f, \"{}\", self.toString())"));
}

#[test]
fn emits_drop_impl_for_on_drop_block() {
    let program = orust_syntax::parse(
        "class Guard { String label; void close() { print(label); } on drop { close(); } }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("impl Drop for Guard"));
    assert!(output.contains("fn drop(&mut self)"));
    assert!(output.contains("self.close();"));
}

#[test]
fn emits_an_inferred_lifetime_for_borrowed_fields() {
    let program = orust_syntax::parse("class Reader { lend String text; int pos; }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("struct Reader<'a>"));
    assert!(output.contains("text: &'a str"));
    assert!(output.contains("impl<'a> Reader<'a>"));
}

#[test]
fn emits_initializing_and_named_constructors() {
    let program = orust_syntax::parse(
        "class Reader { lend String text; int pos = 0; Reader(this.text); Reader.empty(this.text) { return; } }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("fn new(text: &'a str) -> Self"));
    assert!(output.contains("text: text"));
    assert!(output.contains("pos: 0"));
    assert!(output.contains("fn empty(text: &'a str) -> Self"));
}

#[test]
fn adds_elided_lifetimes_to_borrowed_class_references() {
    let program = orust_syntax::parse(
        "class Reader { lend String text; } Reader make(lend Reader value) { return value; }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("value: &Reader<'_>"));
}

#[test]
fn emits_borrowed_constructor_arguments() {
    let program = orust_syntax::parse(
        "class Reader { lend String text; Reader(this.text); } void main() { var s = \"x\"; var r = new Reader(lend s); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("Reader::new(&s)"));
}

#[test]
fn widens_integer_literals_for_double_locals() {
    let program = orust_syntax::parse("void main() { double value = 5; print(value); }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("let value: f64 = (5 as f64);"));
}

#[test]
fn emits_checked_integer_indexing() {
    let program =
        orust_syntax::parse("void main() { var values = [1, 2]; print(values[5]); }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("orust_runtime::checked_index(5, values.len())"));
}

#[test]
fn prints_nullable_locals_as_null_or_value() {
    let program =
        orust_syntax::parse("void main() { String? value = null; print(value); }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("orust_runtime::display_option(&value)"));
}

#[test]
fn emits_single_case_errors_and_typed_results() {
    let program = orust_syntax::parse(
        "error NotFound { String path } String load(String path) throws NotFound { return path; }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("struct NotFound"));
    assert!(output.contains("impl std::error::Error for NotFound"));
    assert!(output.contains("fn load(path: String) -> Result<String, NotFound>"));
}

#[test]
fn emits_multi_case_errors_as_rust_enums() {
    let program = orust_syntax::parse(
        "error LoadError { Missing(String path) => \"missing\"; Permission(String user); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("enum LoadError"));
    assert!(output.contains("Missing {"));
    assert!(output.contains("path: String"));
    assert!(output.contains("LoadError::Missing { .. } => write!(f, \"missing\")"));
}

#[test]
fn constructs_typed_error_values() {
    let program = orust_syntax::parse(
        "error Missing { String path } String load(String path) throws Missing { throw new Missing(path); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("return Err(Missing { path: path }.into());"));
}

#[test]
fn emits_builtin_tests_module_and_expect_assertions() {
    let program = orust_syntax::parse("test \"adds numbers\" { expect(1 == 1); }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("#[cfg(test)]"));
    assert!(output.contains("#[test]"));
    assert!(output.contains("fn orust_test_adds_numbers()"));
    assert!(output.contains("assert!(1 == 1);"));
}

#[test]
fn resolves_std_math_imports_to_runtime_helpers() {
    let program = orust_syntax::parse(
        "import 'std:math' show sqrt, pi; void main() { print(sqrt(9)); print(pi); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("use orust_runtime::std_math::{sqrt, pi};"));
}

#[test]
fn resolves_std_env_imports_to_runtime_helpers() {
    let program =
        orust_syntax::parse("import 'std:env' show cwd; void main() { print(cwd()); }").unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("use orust_runtime::std_env::{cwd};"));
}

#[test]
fn resolves_std_io_types_and_camel_case_methods() {
    let program = orust_syntax::parse(
        "import 'std:io' show File; void main() { print(File.exists(\"Cargo.toml\")); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("use orust_runtime::std_io::{File};"));
    assert!(output.contains("orust_runtime::std_io::File::exists"));
}

#[test]
fn lowers_constructor_defaults_through_generated_argument_structs() {
    let program = orust_syntax::parse(
        "class Greeter { String name; Greeter(String name = \"world\"); } void main() { var value = new Greeter(); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("struct GreeterNewArgs"));
    assert!(output.contains("args.name.unwrap_or(\"world\".to_string())"));
    assert!(output.contains("Greeter::new(GreeterNewArgs"));
}

#[test]
fn lowers_named_constructor_arguments() {
    let program = orust_syntax::parse(
        "class Greeter { String name; String punctuation; Greeter(String name = \"world\", String punctuation = \"!\"); } void main() { var value = new Greeter(punctuation: \"?\"); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("punctuation: Some(\"?\".to_string())"));
    assert!(output.contains("..Default::default()"));
}

#[test]
fn lowers_named_constructor_calls() {
    let program = orust_syntax::parse(
        "class Reader { String text; Reader.empty(String text); } void main() { var reader = new Reader.empty(\"ok\"); }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("Reader::empty(\"ok\".to_string())"));
}

#[test]
fn emits_factory_constructors_as_named_associated_builders() {
    let program = orust_syntax::parse(
        "class Reader { String text; Reader(String text); factory Reader.from(String text) { return new Reader(text); } }",
    )
    .unwrap();
    let output = orust_emit::emit(&program);
    assert!(output.contains("fn from(text: String) -> Self"));
    assert!(output.contains("return Reader::new(text);"));
}
