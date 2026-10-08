use orust_diag::{translate, Diagnostic};
use orust_syntax::Span;
use std::fs;

#[test]
fn all_error_templates_match_golden_files() {
    let cases = [
        ("E0382", "use-after-move"),
        ("E0499", "multiple-mut-borrows"),
        ("E0502", "mut-borrow-while-borrowed"),
        ("E0106", "missing-lifetime"),
        ("E0505", "move-while-borrowed"),
        ("E0373", "spawn-borrowed-value"),
        ("E0277", "spawn-send-failure"),
        ("E0597", "async-borrow-too-short"),
        ("E0521", "async-borrow-escapes"),
        ("E0728", "await-outside-async"),
        ("E0038", "dyn-compatibility"),
        ("E0277", "generic-trait-bound"),
        ("E0599", "generic-missing-method"),
        ("E0004", "missing-enum-case"),
    ];
    for (code, name) in cases {
        let source = fs::read_to_string(format!("../../tests/cases/{name}.or")).unwrap();
        let expected = fs::read_to_string(format!("../../tests/cases/{name}.err")).unwrap();
        let diagnostic = Diagnostic {
            level: "error".into(),
            code: Some(code.into()),
            message: if code == "E0277" && name == "generic-trait-bound" {
                "trait bound missing for `a`".to_string()
            } else {
                "compiler error involving `a`".to_string()
            },
            source_span: Some(Span { start: 0, end: 1 }),
        };
        assert_eq!(
            translate(&diagnostic, &source, "main.or").trim_end(),
            expected.trim_end(),
            "template {code}"
        );
    }
}
