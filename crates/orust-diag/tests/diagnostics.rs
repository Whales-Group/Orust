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
fn preserves_warning_severity_when_rendering_diagnostics() {
    let json = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused variable `value`","code":{"code":"unused_variables"},"spans":[{"line_start":1,"column_start":5}]}}"#;
    let diagnostics = orust_diag::parse_json_diagnostics(
        json,
        "fn main() {}",
        &[SpanMapping {
            generated_start: 0,
            generated_end: 12,
            source_span: Span { start: 0, end: 3 },
        }],
    );
    assert_eq!(diagnostics.len(), 1);
    let rendered = orust_diag::translate(&diagnostics[0], "var value = 1;", "main.or");
    assert!(rendered.starts_with("warning:"), "{rendered}");
    assert!(!rendered.starts_with("error:"), "{rendered}");
}

#[test]
fn prefers_the_primary_rust_span_when_multiple_spans_are_present() {
    let json = r#"{"reason":"compiler-message","message":{"level":"error","message":"cannot borrow","code":{"code":"E0502"},"spans":[{"line_start":1,"column_start":1,"is_primary":false},{"line_start":1,"column_start":5,"is_primary":true}]}}"#;
    let diagnostics = orust_diag::parse_json_diagnostics(
        json,
        "fn main() {}",
        &[
            SpanMapping {
                generated_start: 0,
                generated_end: 3,
                source_span: Span { start: 0, end: 1 },
            },
            SpanMapping {
                generated_start: 3,
                generated_end: 12,
                source_span: Span { start: 8, end: 12 },
            },
        ],
    );
    assert_eq!(diagnostics[0].source_span, Some(Span { start: 8, end: 12 }));
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

#[test]
fn does_not_claim_line_one_when_a_generated_location_is_unmapped() {
    let diagnostic = orust_diag::Diagnostic {
        level: "error".into(),
        code: Some("E0425".into()),
        message: "cannot find value `generated_helper`".into(),
        source_span: None,
    };
    let output = orust_diag::translate(&diagnostic, "void main() {}", "main.or");
    assert!(output.contains("generated Rust location could not be mapped"));
    assert!(!output.contains("main.or:1:1"));
}
