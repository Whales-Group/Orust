use rustplain_diag::Diagnostic;
use rustplain_explain::{explain, Explanation};

pub fn render(diagnostic: &Diagnostic, show_original: bool) -> String {
    let explanation = explain(diagnostic);
    render_explanation(&explanation, show_original)
}

pub fn render_explanation(explanation: &Explanation, show_original: bool) -> String {
    let mut output = format!(
        "error{}: {}\n",
        format_code(explanation),
        explanation.headline
    );
    if let Some(span) = explanation.story.first() {
        if !span.span.file_name.is_empty() {
            output.push_str(&format!(
                " --> {}:{}:{}\n",
                span.span.file_name, span.span.line_start, span.span.column_start
            ));
        }
        for line in &span.span.text {
            output.push_str(&format!(" {} | {}\n", span.span.line_start, line.text));
            output.push_str(&format!(
                "   | {}^ {}\n",
                " ".repeat(line.highlight_start),
                span.note
            ));
        }
    }
    output.push_str(&format!("\nWhat happened: {}\n", explanation.what_happened));
    output.push_str(&format!("Why Rust cares: {}\n", explanation.why_rust_cares));
    for (index, fix) in explanation.fixes.iter().enumerate() {
        output.push_str(&format!("Fix {}: {}\n", index + 1, fix));
    }
    output.push_str(&format!("Confidence: {:?}\n", explanation.confidence));
    if show_original {
        if let Some(original) = &explanation.original.rendered {
            output.push_str(&format!("\nOriginal rustc output:\n{original}"));
        }
    }
    output
}

fn format_code(explanation: &Explanation) -> String {
    explanation
        .codes
        .first()
        .map(|code| format!("[{code}]"))
        .unwrap_or_default()
}
