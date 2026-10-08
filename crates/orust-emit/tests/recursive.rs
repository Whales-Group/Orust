use orust_emit::emit;
use orust_syntax::parse;

#[test]
fn auto_boxes_a_recursive_optional_field() {
    let program = parse(
        r#"
        class Node {
            int value;
            Node? next;
            Node(this.value, this.next);
        }
        "#,
    )
    .expect("recursive class should parse");

    let generated = emit(&program);
    assert!(generated.contains("next: Option<Box<Node>>"), "{generated}");
    assert!(
        generated.contains("next: next.map(Box::new)"),
        "{generated}"
    );
}

#[test]
fn auto_boxes_a_mutually_recursive_edge() {
    let program = parse(
        r#"
        class Tree { Forest forest; }
        class Forest { Tree tree; }
        "#,
    )
    .expect("mutually recursive classes should parse");

    let generated = emit(&program);
    assert!(
        generated.contains("forest: Box<Forest>") || generated.contains("tree: Box<Tree>"),
        "{generated}"
    );
}

#[test]
fn emits_checked_slice_bounds() {
    let program = parse("void main() { var part = values[1..=3]; }").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("&values[orust_runtime::checked_index(1, values.len())..=orust_runtime::checked_index(3, values.len())]"),
        "{generated}"
    );
}

#[test]
fn maps_borrowed_strings_and_lists_to_slice_types() {
    let program = parse("void inspect(lend String text, lend List<int> values) { print(text.length); print(values.length); }").unwrap();
    let generated = emit(&program);
    assert!(generated.contains("text: &str"), "{generated}");
    assert!(generated.contains("values: &[i64]"), "{generated}");
}

#[test]
fn widens_mutable_borrowed_lists_when_the_body_grows_them() {
    let program = parse(
        "void append(lend mut List<int> values) { values.add(1); } void inspect(lend mut List<int> values) { values[0] = 2; }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("fn append(values: &mut Vec<i64>)"),
        "{generated}"
    );
    assert!(
        generated.contains("fn inspect(values: &mut [i64])"),
        "{generated}"
    );
}

#[test]
fn emits_borrowing_for_in_loops() {
    let program = parse("void main() { for (var item in items) { print(item); } }").unwrap();
    let generated = emit(&program);
    assert!(generated.contains("for item in &items"), "{generated}");
}

#[test]
fn emits_interface_supertraits_and_associated_types() {
    let program = parse(
        "interface Animal { void speak(); } interface Pet extends Animal { type Kind; Kind value(); }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(generated.contains("trait Pet: Animal"), "{generated}");
    assert!(generated.contains("type Kind;"), "{generated}");
}

#[test]
fn emits_associated_type_assignments_in_trait_impls() {
    let program = parse(
        "interface Container { type Item; Item get(int index); } class Names implements Container { type Item = String; String get(int index) { return \"name\"; } }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("impl Container for Names"),
        "{generated}"
    );
    assert!(generated.contains("type Item = String;"), "{generated}");
}

#[test]
fn emits_aliases_and_distinct_newtypes() {
    let program = parse("typedef UserId = int; type EmailId(String);").unwrap();
    let generated = emit(&program);
    assert!(generated.contains("type UserId = i64;"), "{generated}");
    assert!(
        generated.contains("struct EmailId(pub String);"),
        "{generated}"
    );
    assert!(
        generated.contains("impl EmailId { pub fn value"),
        "{generated}"
    );
}

#[test]
fn rewrites_newtype_value_accesses_to_the_generated_accessor() {
    let program = parse("type UserId(int); int raw(UserId id) { return id.value; }").unwrap();
    let generated = emit(&program);
    assert!(generated.contains("id.value()"), "{generated}");
}

#[test]
fn emits_lazy_collection_adaptors_from_borrowing_iterators() {
    let program = parse("void main() { var result = values.map(|x| x); }").unwrap();
    let generated = emit(&program);
    assert!(generated.contains("values.iter().map"), "{generated}");
}

#[test]
fn keeps_lazy_iterator_chains_on_the_iterator_receiver() {
    let program =
        parse("void main() { var result = values.where(|_| true).map(|x| x).toList(); }").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("values.iter().filter(|_| true).map(|x| x).collect::<Vec<_>>()"),
        "{generated}"
    );
    assert!(
        !generated.contains(".iter().collect::<Vec<_>>()"),
        "{generated}"
    );
}

#[test]
fn emits_match_guards_or_patterns_and_ranges() {
    let program = parse(
        "void inspect(int value) { switch (value) { case 1..=5: { print(value); } case 6 | 7 when value > 0: { print(value); } default: { print(0); } } }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(generated.contains("1..=5 =>"), "{generated}");
    assert!(generated.contains("6 | 7 if value > 0 =>"), "{generated}");
}

#[test]
fn emits_iterator_trait_from_iterator_implementation() {
    let program =
        parse("class Countdown implements Iterator<int> { int? next() { return null; } }").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("impl Iterator for Countdown"),
        "{generated}"
    );
    assert!(generated.contains("type Item = i64;"), "{generated}");
}

#[test]
fn emits_default_trait_from_default_implementation() {
    let program = parse("class Settings implements Default { int retries = 3; }").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("impl Default for Settings")
            && generated.contains("fn default() -> Self { Self::new() }"),
        "{generated}"
    );
}

#[test]
fn emits_from_trait_for_named_constructor() {
    let program = parse("class UserId implements From<int> { UserId.from(int value); }").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("impl From<i64> for UserId"),
        "{generated}"
    );
    assert!(
        generated.contains("fn from(value: i64) -> Self"),
        "{generated}"
    );
}

#[test]
fn emits_tuples_and_positional_access() {
    let program = parse(
        "(int, String) make() { return (1, \"ok\"); } int first((int, String) pair) { return pair.$1; }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("fn make() -> (i64, String)"),
        "{generated}"
    );
    assert!(generated.contains("(1, \"ok\".to_string())"), "{generated}");
}

#[test]
fn emits_string_operations_with_rust_string_apis() {
    let program = parse(
        r#"void inspect(String value) {
            print(value.length);
            print(value.charCount());
            print(value.startsWith("OR"));
            print(value.endsWith("st"));
            print(value.replaceAll("old", "new"));
            print(value.toUpperCase());
            print(value.toLowerCase());
            print(value.indexOf("rust"));
            print(value.substring(1, 3));
            print(value.contains("u"));
            print(value.split(","));
            print(value.trim());
            print(value.chars());
            print(value.bytes());
            print(value.lines());
        }"#,
    )
    .unwrap();
    let generated = emit(&program);
    assert!(generated.contains("value.len() as i64"), "{generated}");
    assert!(
        generated.contains("value.chars().count() as i64"),
        "{generated}"
    );
    assert!(
        generated.contains("value.starts_with(&\"OR\".to_string())"),
        "{generated}"
    );
    assert!(
        generated.contains("value.ends_with(&\"st\".to_string())"),
        "{generated}"
    );
    assert!(
        generated.contains("value.replace(&\"old\".to_string(), &\"new\".to_string())"),
        "{generated}"
    );
    assert!(generated.contains("value.to_uppercase()"), "{generated}");
    assert!(generated.contains("value.to_lowercase()"), "{generated}");
    assert!(
        generated.contains("value.find(&\"rust\".to_string())"),
        "{generated}"
    );
    assert!(
        generated.contains("chars().skip(1 as usize).take((3 - 1) as usize)"),
        "{generated}"
    );
    assert!(
        generated.contains("value.contains(&\"u\".to_string())"),
        "{generated}"
    );
    assert!(
        generated.contains("value.split(&\",\".to_string())"),
        "{generated}"
    );
    assert!(generated.contains("value.trim()"), "{generated}");
    assert!(generated.contains("value.chars()"), "{generated}");
    assert!(generated.contains("value.bytes()"), "{generated}");
    assert!(generated.contains("value.lines()"), "{generated}");
}

#[test]
fn emits_named_records_as_canonical_generated_structs() {
    let program = parse(
        "void main() { ({int x, String label}) point = (x: 1, label: \"ok\"); print(point.x); }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(generated.contains("pub mod orust_records"), "{generated}");
    assert!(
        generated.contains("pub struct Record_x_int_label_String"),
        "{generated}"
    );
    assert!(
        generated.contains(
            "crate::orust_records::Record_x_int_label_String { x: 1, label: \"ok\".to_string() }"
        ),
        "{generated}"
    );
}

#[test]
fn emits_builtin_comparable_hashable_and_clone_traits() {
    let program =
        parse("class Key implements Comparable, Hashable, Clone { int value; Key(this.value); }")
            .unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("#[derive(Debug, Clone, PartialEq, Eq, Hash)]"),
        "{generated}"
    );
    assert!(
        !generated.contains("impl Comparable for Key"),
        "{generated}"
    );
}

#[test]
fn lowers_comparable_to_real_ord_and_partial_ord_impls() {
    let program = parse(
        "class Score implements Comparable<Score> { int value; Score(this.value); int compareTo(Score other) { return value - other.value; } }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(generated.contains("impl Ord for Score"), "{generated}");
    assert!(
        generated.contains("self.compareTo(other).cmp(&0)"),
        "{generated}"
    );
    assert!(
        generated.contains("impl PartialOrd for Score"),
        "{generated}"
    );
}

#[test]
fn filters_extra_methods_out_of_trait_impl_blocks() {
    let program = parse(
        "interface Speak { void speak(); } class Dog implements Speak { void speak() {} void wag() {} }",
    )
    .unwrap();
    let generated = emit(&program);
    let trait_impl = generated
        .split("impl Speak for Dog")
        .nth(1)
        .expect("trait implementation");
    assert!(trait_impl.contains("fn speak"), "{generated}");
    assert!(!trait_impl.contains("fn wag"), "{generated}");
}

#[test]
fn emits_numeric_checked_wrapping_saturating_and_conversion_helpers() {
    let program = parse(
        "void inspect(int value, double decimal) { print(value.checkedAdd(1)); print(value.wrappingAdd(1)); print(value.saturatingAdd(1)); print(value.isEven()); print(value.toDouble()); print(decimal.round()); }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(generated.contains("value.checked_add(1)"), "{generated}");
    assert!(generated.contains("value.wrapping_add(1)"), "{generated}");
    assert!(generated.contains("value.saturating_add(1)"), "{generated}");
    assert!(generated.contains("value % 2 == 0"), "{generated}");
    assert!(generated.contains("value as f64"), "{generated}");
    assert!(generated.contains("decimal.round() as i64"), "{generated}");
}

#[test]
fn emits_checked_numeric_conversion_with_a_type_argument() {
    let program = parse("void inspect(int value) { print(value.toChecked<u8>()); }").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("u8::try_from(value).ok()"),
        "{generated}"
    );
}

#[test]
fn emits_iterable_into_iterator_and_explicit_consuming_loops() {
    let program = parse(
        "class Library implements Iterable<int> { Iterator<int> iterator() { return values; } } void main() { for (var item in values.consume()) { print(item); } var result = values.where(|x| x > 0).map(|x| x).toList(); }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("impl<'a> IntoIterator for &'a Library"),
        "{generated}"
    );
    assert!(generated.contains("type Item = i64;"), "{generated}");
    assert!(
        generated.contains("for item in values.into_iter()"),
        "{generated}"
    );
    assert!(generated.contains("values.iter().filter"), "{generated}");
    assert!(generated.contains("collect::<Vec<_>>()"), "{generated}");
}

#[test]
fn emits_iterator_returns_as_impl_iterator() {
    let program = parse("Iterator<int> values() { return items.map(|x| x); }").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("fn values() -> impl Iterator<Item = i64>"),
        "{generated}"
    );
}

#[test]
fn emits_tuple_destructuring_declarations() {
    let program = parse("void main() { var (left, right) = (1, 2); print(left); } ").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("let (left,right) = (1, 2);"),
        "{generated}"
    );
}

#[test]
fn boxes_recursive_enum_variant_payloads() {
    let program = parse("enum Expr { Num(int), Add(Expr, Expr), Done }").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("Add(Box<Expr>, Box<Expr>)"),
        "{generated}"
    );
}

#[test]
fn boxes_mutually_recursive_enum_payloads_deterministically() {
    let program = parse("enum A { Next(B) } enum B { Next(A) }").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("Next(Box<B>)") || generated.contains("Next(Box<A>)"),
        "{generated}"
    );
}

#[test]
fn boxes_mixed_class_enum_recursive_components() {
    let program = parse("class Node { Expr expression; } enum Expr { Node(Node) }").unwrap();
    let generated = emit(&program);
    assert!(generated.contains("expression: Box<Expr>"), "{generated}");
    assert!(generated.contains("Node(Box<Node>)"), "{generated}");
}

#[test]
fn boxes_recursive_enum_constructors_at_call_sites() {
    let program = parse(
        "enum Expr { Num(int), Add(Expr, Expr) } Expr make(Expr left, Expr right) { return new Expr.Add(left, right); }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("Expr::Add(Box::new(left), Box::new(right))"),
        "{generated}"
    );
}

#[test]
fn accepts_named_recursive_enum_payloads() {
    let program = parse(
        "enum Expr { Num(int n), Add(Expr left, Expr right) } Expr build(Expr left, Expr right) { return new Expr.Add(left, right); }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(generated.contains("Num(i64)"), "{generated}");
    assert!(
        generated.contains("Add(Box<Expr>, Box<Expr>)"),
        "{generated}"
    );
    assert!(
        generated.contains("Expr::Add(Box::new(left), Box::new(right))"),
        "{generated}"
    );
}

#[test]
fn dereferences_boxed_recursive_pattern_bindings() {
    let program = parse(
        "enum Expr { Num(int), Add(Expr, Expr) } int inspect(Expr value) { switch (value) { case Add(left, right): { return inspect(right); } default: { return 0; } } }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(generated.contains("let right = *right;"), "{generated}");
}

#[test]
fn emits_while_case_as_rust_while_let() {
    let program = parse(
        "void consume(Iterator<int> it) { while (it.next() case var item?) { print(item); } }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("while let Some(item) = it.next()"),
        "{generated}"
    );
}

#[test]
fn emits_refutable_binding_as_rust_let_else() {
    let program = parse(
        "enum Shape { Circle(int), Square(int) } int radius(Shape shape) { var Circle(radius) = shape else { return 0; } return radius; }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("let Circle(radius) = shape else"),
        "{generated}"
    );
    assert!(generated.contains("return 0;"), "{generated}");
}

#[test]
fn emits_generic_function_aliases() {
    let program = parse("typedef Callback<T> = void Function(T);").unwrap();
    let generated = emit(&program);
    assert!(
        generated.contains("type Callback<T> = Box<dyn Fn(T) -> ()>;"),
        "{generated}"
    );
}

#[test]
fn infers_clone_for_copy_usage_on_cloneable_classes() {
    let program = parse(
        "class Counter { int value; } Counter duplicate(Counter value) { return copy value; }",
    )
    .unwrap();
    let generated = emit(&program);
    assert!(generated.contains("#[derive(Debug, Clone)]"), "{generated}");
    assert!(generated.contains("value.clone()"), "{generated}");
}
