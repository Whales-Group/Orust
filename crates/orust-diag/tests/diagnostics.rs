use orust_emit::SpanMapping;
use orust_syntax::Span;

#[test]
fn maps_json_compiler_spans_back_to_orust() {
    let json = r#"{"reason":"compiler-message","message":{"level":"error","message":"cannot find value `x`","code":{"code":"E0425"},"spans":[{"line_start":1,"column_start":2}]}}"#;
    let diagnostics = orust_diag::parse_json_diagnostics(
        json,
        "fn main() {}",
        &[SpanMapping {
            generated_start: 0,
            generated_end: 11,
            source_span: Span { start: 4, end: 9 },
        }],
    );
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code.as_deref(), Some("E0425"));
    assert_eq!(diagnostics[0].source_span, Some(Span { start: 4, end: 9 }));
}

#[test]
fn maps_borrow_errors_to_stable_orust_codes() {
    assert_eq!(orust_diag::orust_code(Some("E0515")), Some("OR0005"));
    assert_eq!(orust_diag::orust_code(Some("E0597")), Some("OR0005"));
    assert_eq!(orust_diag::orust_code(Some("E0506")), Some("OR0005"));
}

#[test]
fn maps_trait_implementation_errors_to_a_stable_orust_code() {
    for code in [
        "E0046", "E0050", "E0053", "E0119", "E0191", "E0220", "E0407",
    ] {
        assert_eq!(orust_diag::orust_code(Some(code)), Some("OR0015"));
    }
}

#[test]
fn maps_pattern_diagnostics_to_stable_orust_codes() {
    assert_eq!(orust_diag::orust_code(Some("E0005")), Some("OR0016"));
    assert_eq!(
        orust_diag::orust_code(Some("unreachable_patterns")),
        Some("OR0017")
    );
}

#[test]
fn translates_borrow_and_drop_check_errors_for_orust_users() {
    let source = "void main() { var value = 1; }";
    let cases = [
        ("E0515", "returns a borrowed value", "return an owned value"),
        (
            "E0506",
            "changed `value`",
            "finish using the borrowed value",
        ),
        (
            "E0597",
            "does not live long enough",
            "async operation finish",
        ),
    ];
    for (code, headline, help) in cases {
        let diagnostic = orust_diag::Diagnostic {
            level: "error".into(),
            code: Some(code.into()),
            message: "borrowed value `value` does not live long enough".into(),
            source_span: None,
        };
        let output = orust_diag::translate(&diagnostic, source, "main.or");
        assert!(output.contains(headline), "{code}: {output}");
        assert!(output.contains(help), "{code}: {output}");
        assert!(orust_diag::orust_code(Some(code)).is_some());
    }

    let drop = orust_diag::Diagnostic {
        level: "error".into(),
        code: Some("E0597".into()),
        message: "borrowed value dropped due to a destructor running".into(),
        source_span: None,
    };
    let output = orust_diag::translate(&drop, source, "main.or");
    assert!(output.contains("runs cleanup code"));
    assert!(output.contains("declare the borrowed source before the object"));
}

#[test]
fn translates_lazy_iterator_borrow_conflicts() {
    let diagnostic = orust_diag::Diagnostic {
        level: "error".into(),
        code: Some("E0502".into()),
        message: "cannot borrow `items` as mutable because it is also borrowed as immutable by an iterator".into(),
        source_span: None,
    };
    let output = orust_diag::translate(&diagnostic, "void main() {}", "main.or");
    assert!(output.contains("lazy iterator chain is still reading `items`"));
    assert!(output.contains("materialize the chain"));
}
