use serde::{Deserialize, Deserializer, Serialize};
use std::io::BufRead;
use thiserror::Error;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Diagnostic {
    pub message: String,
    #[serde(deserialize_with = "deserialize_code")]
    pub code: Option<DiagnosticCode>,
    pub level: String,
    pub spans: Vec<Span>,
    pub children: Vec<Diagnostic>,
    pub rendered: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DiagnosticCode {
    pub code: Option<String>,
    pub explanation: Option<String>,
}

fn deserialize_code<'de, D>(deserializer: D) -> Result<Option<DiagnosticCode>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_json::Value::Null => None,
        serde_json::Value::String(code) => Some(DiagnosticCode {
            code: Some(code),
            explanation: None,
        }),
        serde_json::Value::Object(_) => serde_json::from_value(value).ok(),
        _ => None,
    })
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Span {
    pub file_name: String,
    pub byte_start: u64,
    pub byte_end: u64,
    pub line_start: usize,
    pub line_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub is_primary: bool,
    pub text: Vec<SpanText>,
    pub label: Option<String>,
    pub suggested_replacement: Option<String>,
    pub suggestion_applicability: Option<String>,
    pub expansion: Option<Expansion>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpanText {
    pub text: String,
    pub highlight_start: usize,
    pub highlight_end: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Expansion {
    pub span: Option<Box<Span>>,
    pub macro_decl_name: Option<String>,
    pub def_site_span: Option<Box<Span>>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Artifact {
    pub executable: Option<String>,
    pub package_id: Option<String>,
    pub target: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BuildFinished {
    pub success: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JsonMessage {
    Diagnostic(Diagnostic),
    Artifact(Artifact),
    BuildFinished(BuildFinished),
    Other(serde_json::Value),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid JSON diagnostic line: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn parse_line(line: &str) -> Result<JsonMessage, ParseError> {
    let value: serde_json::Value = serde_json::from_str(line)?;
    if value.get("reason").and_then(|v| v.as_str()) == Some("compiler-message") {
        if let Some(message) = value.get("message") {
            return Ok(JsonMessage::Diagnostic(serde_json::from_value(
                message.clone(),
            )?));
        }
    }
    match value.get("reason").and_then(|v| v.as_str()) {
        Some("compiler-artifact") => Ok(JsonMessage::Artifact(serde_json::from_value(value)?)),
        Some("build-finished") => Ok(JsonMessage::BuildFinished(BuildFinished {
            success: value
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })),
        _ if value.get("message").and_then(|v| v.as_str()).is_some()
            || value.get("spans").is_some() =>
        {
            Ok(JsonMessage::Diagnostic(serde_json::from_value(value)?))
        }
        _ => Ok(JsonMessage::Other(value)),
    }
}

pub fn read_lines<R: BufRead>(reader: R) -> Result<Vec<JsonMessage>, ParseError> {
    reader
        .lines()
        .filter_map(|line| match line {
            Ok(line) if !line.trim().is_empty() => Some(Ok(line)),
            Ok(_) => None,
            Err(error) => Some(Err(ParseError::Json(serde_json::Error::io(error)))),
        })
        .map(|line| parse_line(&line?))
        .collect()
}

pub fn code(diagnostic: &Diagnostic) -> Option<&str> {
    diagnostic
        .code
        .as_ref()
        .and_then(|code| code.code.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_and_raw_diagnostics() {
        let cargo = r#"{"reason":"compiler-message","message":{"message":"borrow of moved value: `x`","code":{"code":"E0382"},"level":"error","spans":[],"children":[]}}"#;
        let JsonMessage::Diagnostic(diagnostic) = parse_line(cargo).unwrap() else {
            panic!()
        };
        assert_eq!(code(&diagnostic), Some("E0382"));

        let raw = r#"{"message":"warning","level":"warning","spans":[]}"#;
        assert!(matches!(
            parse_line(raw).unwrap(),
            JsonMessage::Diagnostic(_)
        ));
    }

    #[test]
    fn tolerates_missing_and_unknown_fields() {
        let JsonMessage::Diagnostic(diagnostic) =
            parse_line(r#"{"message":"x","level":"mystery","unknown":true}"#).unwrap()
        else {
            panic!()
        };
        assert_eq!(diagnostic.message, "x");
        assert_eq!(diagnostic.level, "mystery");
    }
}
