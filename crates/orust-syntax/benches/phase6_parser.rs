use std::hint::black_box;

use orust_syntax::parse;

fn main() {
    let source = r#"
/// Parses a documented class.
/// @param value input value
class Example {
    String value;
    Example(String value) { this.value = value; }
}
    "#;
    let iterations = 1_000;
    let mut comments = 0;
    for _ in 0..iterations {
        let program = parse(black_box(source)).expect("benchmark fixture must parse");
        comments += program.comments.len();
        black_box(program);
    }
    println!("phase6_parser: {iterations} deterministic parses ({comments} comments observed)");
}
