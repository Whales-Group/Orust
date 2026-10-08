use rustplain_diag::{code, Diagnostic, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub span: Span,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Explanation {
    pub headline: String,
    pub what_happened: String,
    pub story: Vec<Event>,
    pub why_rust_cares: String,
    pub fixes: Vec<String>,
    pub confidence: Confidence,
    pub codes: Vec<String>,
    pub original: Diagnostic,
}

pub fn explain(diagnostic: &Diagnostic) -> Explanation {
    let rust_code = code(diagnostic).map(str::to_owned);
    let headline = rust_code
        .as_deref()
        .map(|code| format!("Rust reported {code}: {}", diagnostic.message))
        .unwrap_or_else(|| diagnostic.message.clone());
    Explanation {
        headline,
        what_happened: diagnostic.message.clone(),
        story: diagnostic
            .spans
            .iter()
            .filter(|span| span.is_primary || span.label.is_some())
            .map(|span| Event {
                span: span.clone(),
                note: span
                    .label
                    .clone()
                    .unwrap_or_else(|| diagnostic.message.clone()),
            })
            .collect(),
        why_rust_cares: "rustc made this decision to preserve Rust's safety guarantees.".into(),
        fixes: diagnostic
            .children
            .iter()
            .filter(|child| child.level == "help")
            .map(|child| child.message.clone())
            .collect(),
        confidence: if rust_code.is_some() {
            Confidence::Medium
        } else {
            Confidence::Low
        },
        codes: rust_code.into_iter().collect(),
        original: diagnostic.clone(),
    }
}
