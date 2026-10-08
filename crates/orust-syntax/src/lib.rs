use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    /// Crate-level Rust `use` declarations that should be emitted alongside
    /// the generated ORust items.
    pub rust_uses: Vec<String>,
    pub attributes: Vec<ItemAttributes>,
    pub items: Vec<Spanned<Item>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemAttributes {
    pub derives: Vec<String>,
    pub repr: Option<String>,
    pub cfg: Option<String>,
    pub unsafe_rust: bool,
    pub orust_export: bool,
    pub borrowed_lifetime: Option<String>,
    pub owned: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
    pub show: Vec<String>,
    pub hide: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    pub path: String,
    pub show: Vec<String>,
    pub hide: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Class(Class),
    Function(Function),
    Interface(Interface),
    Enum(Enum),
    Error(Error),
    Extension(Extension),
    TypeAlias(TypeAlias),
    Newtype(Newtype),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeAlias {
    pub exported: bool,
    pub name: String,
    pub rust_name: Option<String>,
    pub rust_import: Option<String>,
    pub generics: Vec<String>,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Newtype {
    pub exported: bool,
    pub name: String,
    pub rust_name: Option<String>,
    pub inner: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub exported: bool,
    pub name: String,
    pub rust_name: Option<String>,
    pub fields: Vec<Field>,
    pub cases: Vec<ErrorCase>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ErrorCase {
    pub name: String,
    pub fields: Vec<Field>,
    pub message: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Extension {
    pub exported: bool,
    pub target: String,
    pub methods: Vec<Function>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Enum {
    pub exported: bool,
    pub name: String,
    pub rust_name: Option<String>,
    pub generics: Vec<String>,
    pub bounds: Vec<String>,
    pub variants: Vec<Variant>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Interface {
    pub exported: bool,
    pub name: String,
    pub rust_name: Option<String>,
    pub extends: Vec<String>,
    pub associated_types: Vec<String>,
    pub methods: Vec<Function>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub exported: bool,
    pub name: String,
    pub rust_name: Option<String>,
    pub generics: Vec<String>,
    pub bounds: Vec<String>,
    pub data: bool,
    pub shared: bool,
    pub implements: Vec<String>,
    pub associated_types: Vec<(String, String)>,
    pub fields: Vec<Field>,
    pub methods: Vec<Function>,
    pub constructors: Vec<Constructor>,
    pub drop_body: Option<Vec<Spanned<Stmt>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Constructor {
    pub exported: bool,
    pub factory: bool,
    pub name: String,
    pub params: Vec<Param>,
    pub initializing_fields: Vec<String>,
    pub body: Vec<Spanned<Stmt>>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub ty: String,
    pub name: String,
    pub initializer: Option<Expr>,
    pub span: Span,
    pub visibility: Visibility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub exported: bool,
    pub visibility: Visibility,
    pub rust_name: Option<String>,
    pub rust_import: Option<String>,
    pub return_type: String,
    pub name: String,
    pub generics: Vec<String>,
    pub bounds: Vec<String>,
    pub is_async: bool,
    pub throws: bool,
    pub throws_type: Option<String>,
    pub test_name: Option<String>,
    pub params: Vec<Param>,
    pub body: Vec<Spanned<Stmt>>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub ty: String,
    pub name: String,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Var {
        name: String,
        initializer: Expr,
        declared_type: Option<String>,
    },
    PatternVar {
        pattern: String,
        initializer: Expr,
        else_body: Vec<Spanned<Stmt>>,
    },
    Print(Expr),
    Return(Option<Expr>),
    Expr(Expr),
    Rust(String),
    Throw(Expr),
    TryCatch {
        body: Vec<Spanned<Stmt>>,
        error: String,
        catch_body: Vec<Spanned<Stmt>>,
    },
    If {
        condition: Expr,
        then_body: Vec<Spanned<Stmt>>,
        else_body: Vec<Spanned<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Spanned<Stmt>>,
    },
    WhileCase {
        value: Expr,
        pattern: String,
        body: Vec<Spanned<Stmt>>,
    },
    For {
        initializer: Option<Box<Stmt>>,
        condition: Option<Expr>,
        step: Option<Expr>,
        body: Vec<Spanned<Stmt>>,
    },
    ForIn {
        name: String,
        iterable: Expr,
        body: Vec<Spanned<Stmt>>,
    },
    Switch {
        value: Expr,
        cases: Vec<SwitchCase>,
    },
    IfCase {
        value: Expr,
        pattern: String,
        bindings: Vec<String>,
        then_body: Vec<Spanned<Stmt>>,
        else_body: Vec<Spanned<Stmt>>,
    },
    AwaitFor {
        name: String,
        stream: Expr,
        body: Vec<Spanned<Stmt>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SwitchCase {
    pub pattern: String,
    pub bindings: Vec<String>,
    pub guard: Option<String>,
    pub body: Vec<Spanned<Stmt>>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Name(String),
    New(String),
    NewArgs {
        name: String,
        args: Vec<Expr>,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Slice {
        object: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
    },
    Null,
    OptionalMember {
        object: Box<Expr>,
        name: String,
    },
    Coalesce {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Await(Box<Expr>),
    List(Vec<Expr>),
    Spawn(Box<Expr>),
    This,
    Borrow {
        mutable: bool,
        value: Box<Expr>,
    },
    Copy(Box<Expr>),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    NamedArg {
        name: String,
        value: Box<Expr>,
    },
    Member {
        object: Box<Expr>,
        name: String,
    },
    Binary {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Closure {
        params: Vec<String>,
        body: ClosureBody,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClosureBody {
    Expr(Box<Expr>),
    Block(Vec<Spanned<Stmt>>),
}

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Ident(String),
    Number(String),
    String(String),
    Symbol(char),
    Operator(String),
    Keyword(String),
    RawRust(String),
    RawRustExpr(String),
    Eof,
}

fn token_text(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Ident(value)
        | TokenKind::Number(value)
        | TokenKind::String(value)
        | TokenKind::Operator(value)
        | TokenKind::Keyword(value)
        | TokenKind::RawRust(value)
        | TokenKind::RawRustExpr(value) => value.clone(),
        TokenKind::Symbol(value) => value.to_string(),
        TokenKind::Eof => String::new(),
    }
}

fn valid_rust_repr(value: &str) -> bool {
    value.split(',').map(str::trim).all(|part| {
        matches!(
            part,
            "C" | "transparent"
                | "packed"
                | "simd"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
        ) || part
            .strip_prefix("packed(")
            .and_then(|value| value.strip_suffix(')'))
            .is_some_and(|value| value.parse::<u32>().is_ok())
            || part
                .strip_prefix("align(")
                .and_then(|value| value.strip_suffix(')'))
                .is_some_and(|value| value.parse::<u32>().is_ok())
    })
}

fn decode_string_literal(value: &str) -> String {
    let mut decoded = String::new();
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            decoded.push(character);
            continue;
        }
        match characters.next() {
            Some('n') => decoded.push('\n'),
            Some('r') => decoded.push('\r'),
            Some('t') => decoded.push('\t'),
            Some('\\') => decoded.push('\\'),
            Some('"') => decoded.push('"'),
            Some('\'') => decoded.push('\''),
            Some(other) => {
                decoded.push('\\');
                decoded.push(other);
            }
            None => decoded.push('\\'),
        }
    }
    decoded
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}..{}",
            self.message, self.span.start, self.span.end
        )
    }
}
impl std::error::Error for ParseError {}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = lex(source)?;
    Parser { tokens, pos: 0 }.program()
}

fn numeric_body(value: &str) -> &str {
    const SUFFIXES: [&str; 16] = [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "f32", "f64", "int", "double",
    ];
    SUFFIXES
        .iter()
        .find_map(|suffix| value.strip_suffix(suffix))
        .unwrap_or(value)
        .trim_matches('_')
}

fn parse_integer_literal(value: &str) -> i64 {
    let body = numeric_body(value).replace('_', "");
    if let Some(digits) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
        i64::from_str_radix(digits, 16).unwrap_or_default()
    } else if let Some(digits) = body.strip_prefix("0b").or_else(|| body.strip_prefix("0B")) {
        i64::from_str_radix(digits, 2).unwrap_or_default()
    } else if let Some(digits) = body.strip_prefix("0o").or_else(|| body.strip_prefix("0O")) {
        i64::from_str_radix(digits, 8).unwrap_or_default()
    } else {
        body.parse().unwrap_or_default()
    }
}

fn parse_float_literal(value: &str) -> f64 {
    numeric_body(value)
        .replace('_', "")
        .parse()
        .unwrap_or_default()
}

fn lex(source: &str) -> Result<Vec<Token>, ParseError> {
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    'lex: while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        let start = i;
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let s = &source[start..i];
            if s == "rust" {
                let mut brace = i;
                while brace < bytes.len() && bytes[brace].is_ascii_whitespace() {
                    brace += 1;
                }
                if bytes.get(brace) == Some(&b'{') {
                    let content_start = brace + 1;
                    let mut depth = 1;
                    let mut end = content_start;
                    while end < bytes.len() && depth > 0 {
                        if bytes[end] == b'{' {
                            depth += 1;
                        } else if bytes[end] == b'}' {
                            depth -= 1;
                        }
                        end += 1;
                    }
                    if depth != 0 {
                        return Err(ParseError {
                            message: "unterminated rust block".into(),
                            span: Span {
                                start,
                                end: bytes.len(),
                            },
                        });
                    }
                    out.push(Token {
                        kind: TokenKind::RawRust(source[content_start..end - 1].into()),
                        span: Span { start, end },
                    });
                    i = end;
                    continue;
                }
                let mut expression_start = i;
                while expression_start < bytes.len()
                    && bytes[expression_start].is_ascii_whitespace()
                {
                    expression_start += 1;
                }
                let is_rust_use =
                    source[expression_start..]
                        .strip_prefix("use")
                        .is_some_and(|rest| {
                            rest.as_bytes()
                                .first()
                                .is_none_or(|byte| byte.is_ascii_whitespace() || *byte == b';')
                        });
                let is_rust_let =
                    source[expression_start..]
                        .strip_prefix("let")
                        .is_some_and(|rest| {
                            rest.as_bytes()
                                .first()
                                .is_none_or(|byte| byte.is_ascii_whitespace())
                        });
                if expression_start < bytes.len() && is_rust_let {
                    let content_start = expression_start;
                    let mut cursor = content_start;
                    let mut brace_depth = 0usize;
                    let mut paren_depth = 0usize;
                    let mut bracket_depth = 0usize;
                    let mut quote = None;
                    let mut escaped = false;
                    while cursor < bytes.len() {
                        let byte = bytes[cursor];
                        if let Some(active_quote) = quote {
                            if escaped {
                                escaped = false;
                            } else if byte == b'\\' {
                                escaped = true;
                            } else if byte == active_quote {
                                quote = None;
                            }
                        } else if byte == b'"' || byte == b'\'' {
                            quote = Some(byte);
                        } else if byte == b'{' {
                            brace_depth += 1;
                        } else if byte == b'}' {
                            brace_depth = brace_depth.saturating_sub(1);
                        } else if byte == b'(' {
                            paren_depth += 1;
                        } else if byte == b')' {
                            paren_depth = paren_depth.saturating_sub(1);
                        } else if byte == b'[' {
                            bracket_depth += 1;
                        } else if byte == b']' {
                            bracket_depth = bracket_depth.saturating_sub(1);
                        } else if byte == b';'
                            && brace_depth == 0
                            && paren_depth == 0
                            && bracket_depth == 0
                        {
                            out.push(Token {
                                kind: TokenKind::RawRust(
                                    source[content_start..=cursor].trim().into(),
                                ),
                                span: Span {
                                    start,
                                    end: cursor + 1,
                                },
                            });
                            i = cursor + 1;
                            continue 'lex;
                        }
                        cursor += 1;
                    }
                    return Err(ParseError {
                        message: "unterminated rust let declaration".into(),
                        span: Span {
                            start,
                            end: bytes.len(),
                        },
                    });
                }
                if expression_start < bytes.len() && !is_rust_use {
                    let content_start = expression_start;
                    let mut cursor = content_start;
                    let mut brace_depth = 0usize;
                    let mut paren_depth = 0usize;
                    let mut bracket_depth = 0usize;
                    let mut quote = None;
                    let mut escaped = false;
                    while cursor < bytes.len() {
                        let byte = bytes[cursor];
                        if let Some(active_quote) = quote {
                            if escaped {
                                escaped = false;
                            } else if byte == b'\\' {
                                escaped = true;
                            } else if byte == active_quote {
                                quote = None;
                            }
                            cursor += 1;
                            continue;
                        }
                        if byte == b'"' || byte == b'\'' {
                            quote = Some(byte);
                        } else if byte == b'{' {
                            brace_depth += 1;
                        } else if byte == b'}' {
                            brace_depth = brace_depth.saturating_sub(1);
                        } else if byte == b'(' {
                            paren_depth += 1;
                        } else if byte == b')' {
                            paren_depth = paren_depth.saturating_sub(1);
                        } else if byte == b'[' {
                            bracket_depth += 1;
                        } else if byte == b']' {
                            bracket_depth = bracket_depth.saturating_sub(1);
                        } else if byte == b';'
                            && brace_depth == 0
                            && paren_depth == 0
                            && bracket_depth == 0
                        {
                            out.push(Token {
                                kind: TokenKind::RawRustExpr(
                                    source[content_start..cursor].trim().into(),
                                ),
                                span: Span { start, end: cursor },
                            });
                            i = cursor;
                            continue 'lex;
                        }
                        cursor += 1;
                    }
                    if cursor >= bytes.len() {
                        return Err(ParseError {
                            message: "unterminated rust expression".into(),
                            span: Span {
                                start,
                                end: bytes.len(),
                            },
                        });
                    }
                }
            }
            let kind = match s {
                "class" | "void" | "int" | "double" | "bool" | "String" | "var" | "print"
                | "return" | "new" | "true" | "false" | "if" | "else" | "while" | "for"
                | "lend" | "mut" | "copy" | "this" | "interface" | "implements" | "null"
                | "shared" | "async" | "await" | "Future" | "spawn" | "throws" | "try"
                | "catch" | "extends" | "enum" | "error" | "throw" | "test" | "factory"
                | "switch" | "case" | "default" | "extension" | "on" | "drop" | "import"
                | "export" | "as" | "show" | "hide" | "private" | "internal" | "typedef"
                | "type" | "when" => TokenKind::Keyword(s.into()),
                _ => TokenKind::Ident(s.into()),
            };
            out.push(Token {
                kind,
                span: Span { start, end: i },
            });
            continue;
        }
        if bytes[i].is_ascii_digit() {
            if bytes[i] == b'0'
                && matches!(
                    bytes.get(i + 1),
                    Some(b'x' | b'X' | b'b' | b'B' | b'o' | b'O')
                )
            {
                i += 2;
                while i < bytes.len() && (bytes[i].is_ascii_hexdigit() || bytes[i] == b'_') {
                    i += 1;
                }
            } else {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                    i += 1;
                }
                if bytes.get(i) == Some(&b'.') && bytes.get(i + 1).is_some_and(u8::is_ascii_digit) {
                    i += 1;
                    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                        i += 1;
                    }
                }
            }
            while i < bytes.len() && bytes[i].is_ascii_alphanumeric() {
                i += 1;
            }
            out.push(Token {
                kind: TokenKind::Number(source[start..i].into()),
                span: Span { start, end: i },
            });
            continue;
        }
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            i += 1;
            let content_start = i;
            let mut escaped = false;
            while i < bytes.len() {
                if escaped {
                    escaped = false;
                    i += 1;
                    continue;
                }
                if bytes[i] == b'\\' {
                    escaped = true;
                    i += 1;
                    continue;
                }
                if bytes[i] == quote {
                    break;
                }
                i += 1;
            }
            if i == bytes.len() {
                return Err(ParseError {
                    message: "unterminated string".into(),
                    span: Span { start, end: i },
                });
            }
            let s = decode_string_literal(&source[content_start..i]);
            i += 1;
            out.push(Token {
                kind: TokenKind::String(s),
                span: Span { start, end: i },
            });
            continue;
        }
        let c = bytes[i] as char;
        i += 1;
        if c == '?' && bytes.get(i) == Some(&b'?') {
            i += 1;
            out.push(Token {
                kind: TokenKind::Operator("??".into()),
                span: Span { start, end: i },
            });
            continue;
        }
        if c == '?' && bytes.get(i) == Some(&b'.') {
            i += 1;
            out.push(Token {
                kind: TokenKind::Operator("?.".into()),
                span: Span { start, end: i },
            });
            continue;
        }
        if c == '.' && bytes.get(i) == Some(&b'.') {
            i += 1;
            let inclusive = bytes.get(i) == Some(&b'=');
            if inclusive {
                i += 1;
            }
            out.push(Token {
                kind: TokenKind::Operator(if inclusive { "..=" } else { ".." }.into()),
                span: Span { start, end: i },
            });
            continue;
        }
        if "=+-*/<>!".contains(c) {
            let mut op = c.to_string();
            if bytes.get(i) == Some(&b'=') {
                op.push('=');
                i += 1;
            } else if c == '=' && bytes.get(i) == Some(&b'>') {
                op.push('>');
                i += 1;
            }
            out.push(Token {
                kind: TokenKind::Operator(op),
                span: Span { start, end: i },
            });
        } else {
            out.push(Token {
                kind: TokenKind::Symbol(c),
                span: Span { start, end: i },
            });
        }
    }
    out.push(Token {
        kind: TokenKind::Eof,
        span: Span {
            start: source.len(),
            end: source.len(),
        },
    });
    Ok(out)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}
impl Parser {
    fn current(&self) -> &Token {
        &self.tokens[self.pos]
    }
    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }
    fn is(&self, kind: &TokenKind) -> bool {
        &self.current().kind == kind
    }
    fn starts_typed_local(&self) -> bool {
        if matches!(
            self.tokens.get(self.pos).map(|token| &token.kind),
            Some(TokenKind::Symbol('('))
        ) && matches!(
            self.tokens.get(self.pos + 1).map(|token| &token.kind),
            Some(TokenKind::Symbol('{'))
        ) {
            let mut index = self.pos + 2;
            while index < self.tokens.len()
                && !matches!(
                    self.tokens.get(index).map(|token| &token.kind),
                    Some(TokenKind::Symbol(')'))
                )
            {
                index += 1;
            }
            let next_is_name = matches!(
                self.tokens.get(index + 1).map(|token| &token.kind),
                Some(TokenKind::Ident(_))
            );
            let next_is_equals = matches!(
                self.tokens.get(index + 2).map(|token| &token.kind),
                Some(TokenKind::Operator(op)) if op == "="
            );
            return next_is_name && next_is_equals;
        }
        let Some(first) = self.tokens.get(self.pos) else {
            return false;
        };
        let Some(second) = self.tokens.get(self.pos + 1) else {
            return false;
        };
        let Some(third) = self.tokens.get(self.pos + 2) else {
            return false;
        };
        if matches!(first.kind, TokenKind::Ident(_) | TokenKind::Keyword(_))
            && matches!(second.kind, TokenKind::Ident(_))
            && matches!(third.kind, TokenKind::Operator(ref op) if op == "=")
        {
            return true;
        }
        matches!(second.kind, TokenKind::Symbol('?'))
            && matches!(third.kind, TokenKind::Ident(_))
            && matches!(
                self.tokens.get(self.pos + 3).map(|token| &token.kind),
                Some(TokenKind::Operator(op)) if op == "="
            )
    }
    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        if self.is(&kind) {
            Ok(self.bump())
        } else {
            Err(ParseError {
                message: format!("expected {:?}, found {:?}", kind, self.current().kind),
                span: self.current().span,
            })
        }
    }
    fn ident(&mut self) -> Result<(String, Span), ParseError> {
        match self.bump() {
            Token {
                kind: TokenKind::Ident(s),
                span,
            } => Ok((s, span)),
            Token {
                kind: TokenKind::Keyword(s),
                span,
            } if matches!(
                s.as_str(),
                "show" | "hide" | "as" | "private" | "internal" | "error"
            ) =>
            {
                Ok((s, span))
            }
            t => Err(ParseError {
                message: "expected identifier".into(),
                span: t.span,
            }),
        }
    }
    fn rust_ident(&mut self) -> Result<(String, Span), ParseError> {
        match self.bump() {
            Token {
                kind: TokenKind::Ident(s) | TokenKind::Keyword(s),
                span,
            } => Ok((s, span)),
            t => Err(ParseError {
                message: "expected Rust path segment".into(),
                span: t.span,
            }),
        }
    }
    fn rust_use_segment(&mut self) -> Result<(String, Span), ParseError> {
        if self.is(&TokenKind::Operator("*".into())) {
            let token = self.bump();
            return Ok(("*".into(), token.span));
        }
        self.rust_ident()
    }
    fn rust_use_tree(&mut self) -> Result<String, ParseError> {
        let (first, _) = self.rust_use_segment()?;
        let mut path = first;
        while self.is(&TokenKind::Symbol(':')) {
            self.bump();
            self.expect(TokenKind::Symbol(':'))?;
            if self.is(&TokenKind::Symbol('{')) {
                self.bump();
                let mut entries = Vec::new();
                while !self.is(&TokenKind::Symbol('}')) {
                    entries.push(self.rust_use_tree()?);
                    if self.is(&TokenKind::Symbol(',')) {
                        self.bump();
                    } else {
                        break;
                    }
                }
                self.expect(TokenKind::Symbol('}'))?;
                path.push_str("::{");
                path.push_str(&entries.join(", "));
                path.push('}');
                break;
            }
            let (segment, _) = self.rust_use_segment()?;
            path.push_str("::");
            path.push_str(&segment);
        }
        if matches!(
            self.tokens.get(self.pos).map(|token| &token.kind),
            Some(TokenKind::Ident(value) | TokenKind::Keyword(value)) if value == "as"
        ) {
            self.bump();
            let (alias, _) = self.rust_use_segment()?;
            path.push_str(" as ");
            path.push_str(&alias);
        }
        Ok(path)
    }
    fn generics(&mut self) -> Result<(Vec<String>, Vec<String>), ParseError> {
        if !self.is(&TokenKind::Operator("<".into())) {
            return Ok((Vec::new(), Vec::new()));
        }
        self.bump();
        let mut names = Vec::new();
        let mut bounds = Vec::new();
        while !self.is(&TokenKind::Operator(">".into())) {
            let (name, _) = self.ident()?;
            names.push(name.clone());
            if self.is(&TokenKind::Keyword("extends".into())) {
                self.bump();
                let (bound, _) = self.ident()?;
                bounds.push(format!("{name}: {bound}"));
            }
            if !self.is(&TokenKind::Operator(">".into())) {
                self.expect(TokenKind::Symbol(','))?;
            }
        }
        self.bump();
        Ok((names, bounds))
    }
    fn ty(&mut self) -> Result<String, ParseError> {
        if self.is(&TokenKind::Symbol('(')) {
            self.bump();
            if self.is(&TokenKind::Symbol('{')) {
                self.bump();
                let mut fields = Vec::new();
                while !self.is(&TokenKind::Symbol('}')) {
                    let field_ty = self.ty()?;
                    let (field_name, _) = self.ident()?;
                    fields.push(format!("{field_name}:{field_ty}"));
                    if !self.is(&TokenKind::Symbol('}')) {
                        self.expect(TokenKind::Symbol(','))?;
                    }
                }
                self.bump();
                self.expect(TokenKind::Symbol(')'))?;
                return Ok(format!("record{{{}}}", fields.join(",")));
            }
            let mut parts = Vec::new();
            while !self.is(&TokenKind::Symbol(')')) {
                parts.push(self.ty()?);
                if !self.is(&TokenKind::Symbol(')')) {
                    self.expect(TokenKind::Symbol(','))?;
                }
            }
            self.bump();
            return Ok(format!("({})", parts.join(", ")));
        }
        if self.is(&TokenKind::Keyword("lend".into())) {
            self.bump();
            let mutable = if self.is(&TokenKind::Keyword("mut".into())) {
                self.bump();
                true
            } else {
                false
            };
            return Ok(format!(
                "&{}{}",
                if mutable { "mut " } else { "" },
                self.ty()?
            ));
        }
        if self.is(&TokenKind::Keyword("void".into()))
            && matches!(
                self.tokens.get(self.pos + 1).map(|token| &token.kind),
                Some(TokenKind::Ident(name)) if name == "Function"
            )
        {
            self.bump();
            self.bump();
            self.expect(TokenKind::Symbol('('))?;
            let mut args = Vec::new();
            while !self.is(&TokenKind::Symbol(')')) {
                args.push(self.ty()?);
                if !self.is(&TokenKind::Symbol(')')) {
                    self.expect(TokenKind::Symbol(','))?;
                }
            }
            self.bump();
            return Ok(format!("function({})", args.join(",")));
        }
        match self.bump().kind {
            TokenKind::Ident(s) | TokenKind::Keyword(s) => {
                let mut ty = s;
                while self.is(&TokenKind::Symbol(':')) {
                    self.bump();
                    self.expect(TokenKind::Symbol(':'))?;
                    let (segment, _) = self.rust_ident()?;
                    ty.push_str("::");
                    ty.push_str(&segment);
                }
                if self.is(&TokenKind::Operator("<".into())) {
                    self.bump();
                    let mut args = Vec::new();
                    while !self.is(&TokenKind::Operator(">".into())) {
                        args.push(self.ty()?);
                        if !self.is(&TokenKind::Operator(">".into())) {
                            self.expect(TokenKind::Symbol(','))?;
                        }
                    }
                    self.bump();
                    ty = format!("{}<{}>", ty, args.join(", "));
                }
                if self.is(&TokenKind::Symbol('?')) {
                    self.bump();
                    ty.push('?');
                }
                Ok(ty)
            }
            _ => Err(ParseError {
                message: "expected type".into(),
                span: self.current().span,
            }),
        }
    }
    fn program(mut self) -> Result<Program, ParseError> {
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut rust_uses = Vec::new();
        let mut attributes = Vec::new();
        let mut items = Vec::new();
        while !self.is(&TokenKind::Eof) {
            let start = self.current().span.start;
            if matches!(
                self.current().kind,
                TokenKind::Ident(ref value) if value == "rust"
            ) && matches!(
                self.tokens.get(self.pos + 1).map(|token| &token.kind),
                Some(TokenKind::Ident(value) | TokenKind::Keyword(value)) if value == "use"
            ) {
                self.bump();
                self.bump();
                let path = self.rust_use_tree()?;
                self.expect(TokenKind::Symbol(';'))?;
                rust_uses.push(path);
                continue;
            }
            if self.is(&TokenKind::Keyword("import".into())) {
                self.bump();
                let token = self.bump();
                let path = match token.kind {
                    TokenKind::String(path) => path,
                    _ => {
                        return Err(ParseError {
                            message: "expected import path string".into(),
                            span: token.span,
                        })
                    }
                };
                let alias = if self.is(&TokenKind::Keyword("as".into())) {
                    self.bump();
                    Some(self.ident()?.0)
                } else {
                    None
                };
                let mut show = Vec::new();
                let mut hide = Vec::new();
                loop {
                    let target = if self.is(&TokenKind::Keyword("show".into())) {
                        Some(&mut show)
                    } else if self.is(&TokenKind::Keyword("hide".into())) {
                        Some(&mut hide)
                    } else {
                        None
                    };
                    let Some(target) = target else { break };
                    self.bump();
                    target.push(self.ident()?.0);
                    while self.is(&TokenKind::Symbol(',')) {
                        self.bump();
                        target.push(self.ident()?.0);
                    }
                }
                self.expect(TokenKind::Symbol(';'))?;
                imports.push(Import {
                    path,
                    alias,
                    show,
                    hide,
                    span: Span {
                        start,
                        end: self.tokens[self.pos - 1].span.end,
                    },
                });
                continue;
            }
            if self.is(&TokenKind::Keyword("export".into()))
                && matches!(
                    self.tokens.get(self.pos + 1).map(|t| &t.kind),
                    Some(TokenKind::String(_))
                )
            {
                self.bump();
                let token = self.bump();
                let TokenKind::String(path) = token.kind else {
                    unreachable!()
                };
                let mut show = Vec::new();
                let mut hide = Vec::new();
                loop {
                    let target = if self.is(&TokenKind::Keyword("show".into())) {
                        Some(&mut show)
                    } else if self.is(&TokenKind::Keyword("hide".into())) {
                        Some(&mut hide)
                    } else {
                        None
                    };
                    let Some(target) = target else { break };
                    self.bump();
                    target.push(self.ident()?.0);
                    while self.is(&TokenKind::Symbol(',')) {
                        self.bump();
                        target.push(self.ident()?.0);
                    }
                }
                self.expect(TokenKind::Symbol(';'))?;
                exports.push(Export {
                    path,
                    show,
                    hide,
                    span: Span {
                        start,
                        end: self.tokens[self.pos - 1].span.end,
                    },
                });
                continue;
            }
            let mut exported = if self.is(&TokenKind::Keyword("export".into())) {
                self.bump();
                true
            } else {
                false
            };
            let mut data = false;
            let mut rust_name = None;
            let mut rust_type = None;
            let mut rust_import = None;
            let mut item_attributes = ItemAttributes::default();
            while self.is(&TokenKind::Symbol('@')) {
                self.bump();
                let (annotation, _) = self.ident()?;
                if annotation == "data" {
                    data = true;
                } else if annotation == "rustName" {
                    self.expect(TokenKind::Symbol('('))?;
                    let token = self.bump();
                    rust_name = match token.kind {
                        TokenKind::String(value) => Some(value),
                        _ => {
                            return Err(ParseError {
                                message: "expected string in @rustName".into(),
                                span: token.span,
                            })
                        }
                    };
                    self.expect(TokenKind::Symbol(')'))?;
                } else if annotation == "rustType" {
                    self.expect(TokenKind::Symbol('('))?;
                    let token = self.bump();
                    rust_type = match token.kind {
                        TokenKind::String(value) => Some(value),
                        _ => {
                            return Err(ParseError {
                                message: "expected string in @rustType".into(),
                                span: token.span,
                            })
                        }
                    };
                    self.expect(TokenKind::Symbol(')'))?;
                } else if annotation == "derive" {
                    self.expect(TokenKind::Symbol('('))?;
                    while !self.is(&TokenKind::Symbol(')')) {
                        let token = self.bump();
                        let TokenKind::String(value) = token.kind else {
                            return Err(ParseError {
                                message: "expected string in @derive".into(),
                                span: token.span,
                            });
                        };
                        item_attributes.derives.push(value);
                        if !self.is(&TokenKind::Symbol(')')) {
                            self.expect(TokenKind::Symbol(','))?;
                        }
                    }
                    self.bump();
                } else if annotation == "repr" || annotation == "cfg" {
                    self.expect(TokenKind::Symbol('('))?;
                    let token = self.bump();
                    let value = match token.kind {
                        TokenKind::String(value) => value,
                        _ => {
                            return Err(ParseError {
                                message: format!("expected string in @{annotation}"),
                                span: token.span,
                            })
                        }
                    };
                    self.expect(TokenKind::Symbol(')'))?;
                    if annotation == "repr" {
                        if !valid_rust_repr(&value) {
                            return Err(ParseError {
                                message: format!("unsupported Rust representation `{value}`"),
                                span: token.span,
                            });
                        }
                        item_attributes.repr = Some(value);
                    } else {
                        item_attributes.cfg = Some(value);
                    }
                } else if annotation == "unsafeRust" {
                    item_attributes.unsafe_rust = true;
                } else if annotation == "orustExport" {
                    item_attributes.orust_export = true;
                    exported = true;
                } else if annotation == "borrowed" {
                    let lifetime = if self.is(&TokenKind::Symbol('(')) {
                        self.bump();
                        let token = self.bump();
                        let value = match token.kind {
                            TokenKind::String(value) => value,
                            _ => {
                                return Err(ParseError {
                                    message: "expected lifetime string in @borrowed".into(),
                                    span: token.span,
                                })
                            }
                        };
                        self.expect(TokenKind::Symbol(')'))?;
                        value
                    } else {
                        "'a".into()
                    };
                    item_attributes.borrowed_lifetime = Some(lifetime);
                } else if annotation == "owned" {
                    item_attributes.owned = true;
                } else if annotation == "rustImport" || annotation == "externRust" {
                    self.expect(TokenKind::Symbol('('))?;
                    let token = self.bump();
                    rust_import = match token.kind {
                        TokenKind::String(value) => Some(value),
                        _ => {
                            return Err(ParseError {
                                message: format!("expected string in @{annotation}"),
                                span: token.span,
                            })
                        }
                    };
                    self.expect(TokenKind::Symbol(')'))?;
                } else {
                    return Err(ParseError {
                        message: format!("unknown annotation @{annotation}"),
                        span: self.current().span,
                    });
                }
            }
            let mut item = if self.is(&TokenKind::Keyword("class".into())) {
                Item::Class(self.class(false, data, exported)?)
            } else if self.is(&TokenKind::Keyword("shared".into())) {
                self.bump();
                self.expect(TokenKind::Keyword("class".into()))?;
                Item::Class(self.class_after(true, data, exported)?)
            } else if self.is(&TokenKind::Keyword("interface".into())) {
                Item::Interface(self.interface(exported)?)
            } else if self.is(&TokenKind::Keyword("enum".into())) {
                Item::Enum(self.enum_decl(exported)?)
            } else if self.is(&TokenKind::Keyword("error".into())) {
                Item::Error(self.error_decl(exported)?)
            } else if self.is(&TokenKind::Keyword("test".into())) {
                Item::Function(self.test_decl()?)
            } else if self.is(&TokenKind::Keyword("extension".into())) {
                Item::Extension(self.extension(exported)?)
            } else if self.is(&TokenKind::Keyword("typedef".into())) {
                Item::TypeAlias(self.type_alias(exported)?)
            } else if self.is(&TokenKind::Keyword("type".into())) {
                if let Some(ty) = rust_type {
                    self.bump();
                    let (name, _) = self.ident()?;
                    self.expect(TokenKind::Symbol(';'))?;
                    Item::TypeAlias(TypeAlias {
                        exported,
                        name,
                        rust_name: None,
                        rust_import: None,
                        generics: Vec::new(),
                        ty,
                    })
                } else if let Some(path) = rust_import.clone() {
                    self.bump();
                    let (name, _) = self.ident()?;
                    self.expect(TokenKind::Symbol(';'))?;
                    Item::TypeAlias(TypeAlias {
                        exported,
                        name,
                        rust_name: None,
                        rust_import: Some(path.clone()),
                        generics: Vec::new(),
                        ty: path,
                    })
                } else {
                    Item::Newtype(self.newtype(exported)?)
                }
            } else {
                Item::Function(self.function(exported)?)
            };
            match &mut item {
                Item::Class(value) => value.rust_name = rust_name,
                Item::Function(value) => {
                    value.rust_name = rust_name;
                    value.rust_import = rust_import;
                }
                Item::Interface(value) => value.rust_name = rust_name,
                Item::Enum(value) => value.rust_name = rust_name,
                Item::Error(value) => value.rust_name = rust_name,
                Item::Extension(_) => {}
                Item::TypeAlias(value) => value.rust_name = rust_name,
                Item::Newtype(value) => value.rust_name = rust_name,
            }
            if let Some(lifetime) = item_attributes.borrowed_lifetime.as_deref() {
                if !lifetime.starts_with('\'') {
                    return Err(ParseError {
                        message: "@borrowed lifetime must start with `'`".into(),
                        span: Span {
                            start,
                            end: self.tokens[self.pos.saturating_sub(1)].span.end,
                        },
                    });
                }
                if let Item::Function(function) = &mut item {
                    if !function.generics.iter().any(|generic| generic == lifetime) {
                        function.generics.push(lifetime.into());
                    }
                    for parameter in &mut function.params {
                        if let Some(inner) = parameter.ty.strip_prefix("&mut ") {
                            parameter.ty = format!("&{lifetime} mut {inner}");
                        } else if let Some(inner) = parameter.ty.strip_prefix('&') {
                            if !inner.starts_with('\'') {
                                parameter.ty = format!("&{lifetime} {inner}");
                            }
                        }
                    }
                    if let Some(inner) = function.return_type.strip_prefix("&mut ") {
                        function.return_type = format!("&{lifetime} mut {inner}");
                    } else if let Some(inner) = function.return_type.strip_prefix('&') {
                        if !inner.starts_with('\'') {
                            function.return_type = format!("&{lifetime} {inner}");
                        }
                    }
                }
            }
            if item_attributes.borrowed_lifetime.is_some() || item_attributes.owned {
                if let Item::Function(function) = &item {
                    let has_borrowed_input = function
                        .params
                        .iter()
                        .any(|parameter| parameter.ty.starts_with('&'));
                    let has_borrowed_output = function.return_type.starts_with('&');
                    if item_attributes.borrowed_lifetime.is_some()
                        && !has_borrowed_input
                        && !has_borrowed_output
                    {
                        return Err(ParseError {
                            message: "@borrowed requires a borrowed input or return type".into(),
                            span: Span {
                                start,
                                end: self.tokens[self.pos.saturating_sub(1)].span.end,
                            },
                        });
                    }
                    if item_attributes.owned && (has_borrowed_input || has_borrowed_output) {
                        return Err(ParseError {
                            message: "@owned cannot be combined with borrowed inputs or returns"
                                .into(),
                            span: Span {
                                start,
                                end: self.tokens[self.pos.saturating_sub(1)].span.end,
                            },
                        });
                    }
                }
            }
            let end = self.tokens[self.pos.saturating_sub(1)].span.end;
            items.push(Spanned {
                node: item,
                span: Span { start, end },
            });
            attributes.push(item_attributes);
        }
        Ok(Program {
            imports,
            exports,
            rust_uses,
            attributes,
            items,
        })
    }
    fn class(&mut self, shared: bool, data: bool, exported: bool) -> Result<Class, ParseError> {
        self.expect(TokenKind::Keyword("class".into()))?;
        self.class_after(shared, data, exported)
    }

    fn type_alias(&mut self, exported: bool) -> Result<TypeAlias, ParseError> {
        self.expect(TokenKind::Keyword("typedef".into()))?;
        let (name, _) = self.ident()?;
        let (generics, _) = self.generics()?;
        self.expect(TokenKind::Operator("=".into()))?;
        let ty = self.ty()?;
        self.expect(TokenKind::Symbol(';'))?;
        Ok(TypeAlias {
            exported,
            name,
            rust_name: None,
            rust_import: None,
            generics,
            ty,
        })
    }

    fn newtype(&mut self, exported: bool) -> Result<Newtype, ParseError> {
        self.expect(TokenKind::Keyword("type".into()))?;
        let (name, _) = self.ident()?;
        self.expect(TokenKind::Symbol('('))?;
        let inner = self.ty()?;
        self.expect(TokenKind::Symbol(')'))?;
        self.expect(TokenKind::Symbol(';'))?;
        Ok(Newtype {
            exported,
            name,
            rust_name: None,
            inner,
        })
    }
    fn enum_decl(&mut self, exported: bool) -> Result<Enum, ParseError> {
        self.expect(TokenKind::Keyword("enum".into()))?;
        let (name, _) = self.ident()?;
        let (generics, bounds) = self.generics()?;
        self.expect(TokenKind::Symbol('{'))?;
        let mut variants = Vec::new();
        while !self.is(&TokenKind::Symbol('}')) {
            let (variant, span) = self.ident()?;
            let mut fields = Vec::new();
            if self.is(&TokenKind::Symbol('(')) {
                self.bump();
                while !self.is(&TokenKind::Symbol(')')) {
                    fields.push(self.ty()?);
                    if matches!(self.current().kind, TokenKind::Ident(_)) {
                        self.bump();
                    }
                    if !self.is(&TokenKind::Symbol(')')) {
                        self.expect(TokenKind::Symbol(','))?;
                    }
                }
                self.bump();
            }
            variants.push(Variant {
                name: variant,
                fields,
                span,
            });
            if self.is(&TokenKind::Symbol(',')) {
                self.bump();
            } else if !self.is(&TokenKind::Symbol('}')) {
                self.expect(TokenKind::Symbol(';'))?;
            }
        }
        self.bump();
        Ok(Enum {
            exported,
            name,
            rust_name: None,
            generics,
            bounds,
            variants,
        })
    }
    fn extension(&mut self, exported: bool) -> Result<Extension, ParseError> {
        self.expect(TokenKind::Keyword("extension".into()))?;
        self.expect(TokenKind::Keyword("on".into()))?;
        let target = match self.bump().kind {
            TokenKind::Ident(name) | TokenKind::Keyword(name) => name,
            _ => {
                return Err(ParseError {
                    message: "expected extension target type".into(),
                    span: self.current().span,
                })
            }
        };
        self.expect(TokenKind::Symbol('{'))?;
        let mut methods = Vec::new();
        while !self.is(&TokenKind::Symbol('}')) {
            let return_type = self.ty()?;
            let (name, _) = self.ident()?;
            methods.push(self.function_after(return_type, name, Vec::new(), Vec::new())?);
        }
        self.bump();
        Ok(Extension {
            exported,
            target,
            methods,
        })
    }

    fn class_after(
        &mut self,
        shared: bool,
        data: bool,
        exported: bool,
    ) -> Result<Class, ParseError> {
        let (name, _) = self.ident()?;
        let (generics, bounds) = self.generics()?;
        let mut implements = Vec::new();
        if self.is(&TokenKind::Keyword("implements".into())) {
            self.bump();
            implements.push(self.ty()?);
            while self.is(&TokenKind::Symbol(',')) {
                self.bump();
                implements.push(self.ty()?);
            }
        }
        self.expect(TokenKind::Symbol('{'))?;
        let mut associated_types = Vec::new();
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut constructors = Vec::new();
        let mut drop_body = None;
        while !self.is(&TokenKind::Symbol('}')) {
            if matches!(
                self.tokens.get(self.pos).map(|token| &token.kind),
                Some(TokenKind::Ident(value) | TokenKind::Keyword(value)) if value == "type"
            ) {
                self.bump();
                let (type_name, _) = self.ident()?;
                self.expect(TokenKind::Operator("=".into()))?;
                let type_value = self.ty()?;
                self.expect(TokenKind::Symbol(';'))?;
                associated_types.push((type_name, type_value));
                continue;
            }
            if self.is(&TokenKind::Keyword("on".into())) {
                self.bump();
                self.expect(TokenKind::Keyword("drop".into()))?;
                if drop_body.is_some() {
                    return Err(ParseError {
                        message: "a class can only have one `on drop` block".into(),
                        span: self.current().span,
                    });
                }
                drop_body = Some(self.block()?);
                continue;
            }
            let factory = if self.is(&TokenKind::Keyword("factory".into())) {
                self.bump();
                true
            } else {
                false
            };
            let is_constructor = matches!(
                (
                    self.tokens.get(self.pos).map(|token| &token.kind),
                    self.tokens.get(self.pos + 1).map(|token| &token.kind),
                ),
                (
                    Some(TokenKind::Ident(value)),
                    Some(TokenKind::Symbol('(' | '.')))
                    if value == &name
            );
            if is_constructor {
                self.bump();
                let constructor_name = if self.is(&TokenKind::Symbol('.')) {
                    self.bump();
                    self.ident()?.0
                } else {
                    "new".into()
                };
                constructors.push(self.constructor_after(
                    constructor_name,
                    &fields,
                    exported,
                    factory,
                )?);
                continue;
            }
            let visibility = if self.is(&TokenKind::Keyword("private".into())) {
                self.bump();
                Visibility::Private
            } else if self.is(&TokenKind::Keyword("internal".into())) {
                self.bump();
                Visibility::Internal
            } else {
                Visibility::Public
            };
            let return_type = self.ty()?;
            let (member, _) = self.ident()?;
            let (method_generics, method_bounds) = self.generics()?;
            if self.is(&TokenKind::Symbol('(')) {
                let mut method =
                    self.function_after(return_type, member, method_generics, method_bounds)?;
                method.visibility = visibility;
                methods.push(method);
            } else {
                let initializer = if self.is(&TokenKind::Operator("=".into())) {
                    self.bump();
                    Some(self.expr()?)
                } else {
                    None
                };
                self.expect(TokenKind::Symbol(';'))?;
                fields.push(Field {
                    ty: return_type,
                    name: member,
                    initializer,
                    span: self.current().span,
                    visibility,
                });
            }
        }
        self.expect(TokenKind::Symbol('}'))?;
        if shared && fields.iter().any(|field| field.ty.starts_with('&')) {
            return Err(ParseError {
                message: "a shared class cannot hold borrowed fields".into(),
                span: self.current().span,
            });
        }
        let mut constructor_names = std::collections::HashSet::new();
        for constructor in &constructors {
            if !constructor_names.insert(constructor.name.clone()) {
                return Err(ParseError {
                    message: format!(
                        "class `{name}` declares constructor `{}` more than once",
                        constructor.name
                    ),
                    span: constructor.span,
                });
            }
            if constructor.factory && constructor.name == "new" {
                return Err(ParseError {
                    message:
                        "a factory constructor must have a named form such as `Class.from(...)`"
                            .into(),
                    span: constructor.span,
                });
            }
            let mut seen_parameters = std::collections::HashSet::new();
            let mut saw_default = false;
            for parameter in &constructor.params {
                if !seen_parameters.insert(parameter.name.clone()) {
                    return Err(ParseError {
                        message: format!(
                            "constructor `{}` repeats parameter `{}`",
                            constructor.name, parameter.name
                        ),
                        span: parameter.span,
                    });
                }
                if parameter.default.is_some() {
                    saw_default = true;
                } else if saw_default {
                    return Err(ParseError {
                        message: format!(
                            "constructor `{}` puts a required parameter after a default",
                            constructor.name
                        ),
                        span: parameter.span,
                    });
                }
            }
            if let Some(field) = fields.iter().find(|field| {
                field.ty.starts_with('&')
                    && field.initializer.is_none()
                    && !constructor.initializing_fields.contains(&field.name)
            }) {
                return Err(ParseError {
                    message: format!(
                        "constructor `{}` must initialize borrowed field `{}`",
                        constructor.name, field.name
                    ),
                    span: constructor.span,
                });
            }
        }
        Ok(Class {
            exported,
            name,
            rust_name: None,
            generics,
            bounds,
            data,
            shared,
            implements,
            associated_types,
            fields,
            methods,
            constructors,
            drop_body,
        })
    }

    fn constructor_after(
        &mut self,
        name: String,
        fields: &[Field],
        exported: bool,
        factory: bool,
    ) -> Result<Constructor, ParseError> {
        let start = self.current().span.start;
        self.expect(TokenKind::Symbol('('))?;
        let mut params = Vec::new();
        let mut initializing_fields = Vec::new();
        while !self.is(&TokenKind::Symbol(')')) {
            if self.is(&TokenKind::Keyword("this".into())) {
                self.bump();
                self.expect(TokenKind::Symbol('.'))?;
                let (field, span) = self.ident()?;
                let Some(declaration) = fields.iter().find(|value| value.name == field) else {
                    return Err(ParseError {
                        message: format!("constructor parameter refers to unknown field `{field}`"),
                        span,
                    });
                };
                if initializing_fields.contains(&field) {
                    return Err(ParseError {
                        message: format!("field `{field}` is initialized more than once"),
                        span,
                    });
                }
                initializing_fields.push(field.clone());
                let default = if self.is(&TokenKind::Operator("=".into())) {
                    self.bump();
                    Some(self.expr()?)
                } else {
                    None
                };
                params.push(Param {
                    ty: declaration.ty.clone(),
                    name: field,
                    default,
                    span,
                });
            } else {
                let ty = self.ty()?;
                let (parameter, span) = self.ident()?;
                let default = if self.is(&TokenKind::Operator("=".into())) {
                    self.bump();
                    Some(self.expr()?)
                } else {
                    None
                };
                params.push(Param {
                    ty,
                    name: parameter,
                    default,
                    span,
                });
            }
            if !self.is(&TokenKind::Symbol(')')) {
                self.expect(TokenKind::Symbol(','))?;
            }
        }
        self.bump();
        let body = if self.is(&TokenKind::Symbol('{')) {
            self.block()?
        } else {
            self.expect(TokenKind::Symbol(';'))?;
            Vec::new()
        };
        Ok(Constructor {
            exported,
            factory,
            name,
            params,
            initializing_fields,
            body,
            span: Span {
                start,
                end: self.current().span.end,
            },
        })
    }

    fn error_decl(&mut self, exported: bool) -> Result<Error, ParseError> {
        let start = self.bump().span.start;
        let (name, _) = self.ident()?;
        self.expect(TokenKind::Symbol('{'))?;
        let mut fields = Vec::new();
        let mut cases = Vec::new();
        if !self.is(&TokenKind::Symbol('}')) {
            let is_cases = matches!(
                self.tokens.get(self.pos + 1).map(|token| &token.kind),
                Some(TokenKind::Symbol('('))
            );
            if is_cases {
                while !self.is(&TokenKind::Symbol('}')) {
                    let (case_name, case_span) = self.ident()?;
                    self.expect(TokenKind::Symbol('('))?;
                    let mut case_fields = Vec::new();
                    while !self.is(&TokenKind::Symbol(')')) {
                        let ty = self.ty()?;
                        let (field_name, field_span) = self.ident()?;
                        case_fields.push(Field {
                            ty,
                            name: field_name,
                            initializer: None,
                            span: field_span,
                            visibility: Visibility::Public,
                        });
                        if !self.is(&TokenKind::Symbol(')')) {
                            self.expect(TokenKind::Symbol(','))?;
                        }
                    }
                    self.bump();
                    let message = if self.is(&TokenKind::Operator("=>".into())) {
                        self.bump();
                        match self.bump().kind {
                            TokenKind::String(value) => Some(value),
                            other => {
                                return Err(ParseError {
                                    message: format!(
                                        "expected error message string, found {other:?}"
                                    ),
                                    span: self.current().span,
                                })
                            }
                        }
                    } else {
                        None
                    };
                    let end =
                        if self.is(&TokenKind::Symbol(';')) || self.is(&TokenKind::Symbol(',')) {
                            self.bump().span.end
                        } else {
                            self.current().span.start
                        };
                    cases.push(ErrorCase {
                        name: case_name,
                        fields: case_fields,
                        message,
                        span: Span {
                            start: case_span.start,
                            end,
                        },
                    });
                }
            } else {
                while !self.is(&TokenKind::Symbol('}')) {
                    let ty = self.ty()?;
                    let (field_name, span) = self.ident()?;
                    fields.push(Field {
                        ty,
                        name: field_name,
                        initializer: None,
                        span,
                        visibility: Visibility::Public,
                    });
                    if self.is(&TokenKind::Symbol(';')) || self.is(&TokenKind::Symbol(',')) {
                        self.bump();
                    }
                }
            }
        }
        let end = self.bump().span.end;
        Ok(Error {
            exported,
            name,
            rust_name: None,
            fields,
            cases,
            span: Span { start, end },
        })
    }

    fn test_decl(&mut self) -> Result<Function, ParseError> {
        let start = self.bump().span.start;
        let is_async = if self.is(&TokenKind::Keyword("async".into())) {
            self.bump();
            true
        } else {
            false
        };
        let token = self.bump();
        let test_name = match token.kind {
            TokenKind::String(name) => name,
            _ => {
                return Err(ParseError {
                    message: "expected a string test name".into(),
                    span: token.span,
                })
            }
        };
        let body = self.block()?;
        let name = format!(
            "orust_test_{}",
            test_name
                .chars()
                .map(|character| if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '_'
                })
                .collect::<String>()
        );
        Ok(Function {
            exported: false,
            visibility: Visibility::Private,
            rust_name: None,
            rust_import: None,
            return_type: "void".into(),
            name,
            generics: Vec::new(),
            bounds: Vec::new(),
            is_async,
            throws: false,
            throws_type: None,
            test_name: Some(test_name),
            params: Vec::new(),
            body,
            span: Span {
                start,
                end: self.current().span.end,
            },
        })
    }

    fn interface(&mut self, exported: bool) -> Result<Interface, ParseError> {
        self.expect(TokenKind::Keyword("interface".into()))?;
        let (name, _) = self.ident()?;
        let mut extends = Vec::new();
        if self.is(&TokenKind::Keyword("extends".into())) {
            self.bump();
            extends.push(self.ident()?.0);
            while self.is(&TokenKind::Symbol(',')) {
                self.bump();
                extends.push(self.ident()?.0);
            }
        }
        self.expect(TokenKind::Symbol('{'))?;
        let mut methods = Vec::new();
        let mut associated_types = Vec::new();
        while !self.is(&TokenKind::Symbol('}')) {
            if matches!(
                self.tokens.get(self.pos).map(|token| &token.kind),
                Some(TokenKind::Ident(value) | TokenKind::Keyword(value)) if value == "type"
            ) {
                self.bump();
                associated_types.push(self.ident()?.0);
                self.expect(TokenKind::Symbol(';'))?;
                continue;
            }
            let ty = self.ty()?;
            let (method, _) = self.ident()?;
            self.expect(TokenKind::Symbol('('))?;
            let mut params = Vec::new();
            while !self.is(&TokenKind::Symbol(')')) {
                let param_ty = self.ty()?;
                let (param_name, span) = self.ident()?;
                params.push(Param {
                    ty: param_ty,
                    name: param_name,
                    default: None,
                    span,
                });
                if !self.is(&TokenKind::Symbol(')')) {
                    self.expect(TokenKind::Symbol(','))?;
                }
            }
            self.bump();
            let is_async = if self.is(&TokenKind::Keyword("async".into())) {
                self.bump();
                true
            } else {
                false
            };
            let (body, end) = if self.is(&TokenKind::Symbol(';')) {
                (Vec::new(), self.bump().span.end)
            } else {
                let body = self.block()?;
                let end = self.tokens[self.pos.saturating_sub(1)].span.end;
                (body, end)
            };
            methods.push(Function {
                exported: false,
                visibility: Visibility::Public,
                rust_name: None,
                rust_import: None,
                return_type: ty,
                name: method,
                generics: Vec::new(),
                bounds: Vec::new(),
                is_async,
                throws: false,
                throws_type: None,
                test_name: None,
                params,
                body,
                span: Span { start: end, end },
            });
        }
        self.expect(TokenKind::Symbol('}'))?;
        Ok(Interface {
            exported,
            name,
            rust_name: None,
            extends,
            associated_types,
            methods,
        })
    }
    fn function(&mut self, exported: bool) -> Result<Function, ParseError> {
        let is_async = if self.is(&TokenKind::Keyword("async".into())) {
            self.bump();
            true
        } else {
            false
        };
        if self.is(&TokenKind::Keyword("lend".into()))
            && matches!(
                self.tokens.get(self.pos + 1).map(|token| &token.kind),
                Some(TokenKind::Symbol('('))
            )
        {
            self.bump();
            self.expect(TokenKind::Symbol('('))?;
            self.ident()?;
            self.expect(TokenKind::Symbol(')'))?;
        }
        let ty = self.ty()?;
        let (name, _) = self.ident()?;
        let (generics, bounds) = self.generics()?;
        let mut function = self.function_after(ty.clone(), name, generics, bounds)?;
        function.exported = exported;
        function.is_async = is_async || function.is_async;
        if ty.starts_with('&')
            && !function
                .params
                .iter()
                .any(|parameter| parameter.ty.starts_with('&'))
        {
            return Err(ParseError {
                message: "borrowed return has no input lifetime; use lend(a) syntax".into(),
                span: function.span,
            });
        }
        Ok(function)
    }
    fn function_after(
        &mut self,
        return_type: String,
        name: String,
        generics: Vec<String>,
        bounds: Vec<String>,
    ) -> Result<Function, ParseError> {
        let start = self.current().span.start;
        self.expect(TokenKind::Symbol('('))?;
        let mut params = Vec::new();
        while !self.is(&TokenKind::Symbol(')')) {
            let ty = self.ty()?;
            let (name, span) = self.ident()?;
            let default = if self.is(&TokenKind::Operator("=".into())) {
                self.bump();
                Some(self.expr()?)
            } else {
                None
            };
            params.push(Param {
                ty,
                name,
                default,
                span,
            });
            if !self.is(&TokenKind::Symbol(')')) {
                self.expect(TokenKind::Symbol(','))?;
            }
        }
        self.expect(TokenKind::Symbol(')'))?;
        let is_async = if self.is(&TokenKind::Keyword("async".into())) {
            self.bump();
            true
        } else {
            false
        };
        let (throws, throws_type) = if self.is(&TokenKind::Keyword("throws".into())) {
            self.bump();
            let error = if self.is(&TokenKind::Symbol('{')) || self.is(&TokenKind::Symbol(';')) {
                None
            } else {
                Some(self.ident()?.0)
            };
            (true, error)
        } else {
            (false, None)
        };
        let (body, end) = if self.is(&TokenKind::Symbol(';')) {
            (Vec::new(), self.bump().span.end)
        } else {
            self.expect(TokenKind::Symbol('{'))?;
            let mut body = Vec::new();
            while !self.is(&TokenKind::Symbol('}')) {
                let span = self.current().span;
                body.push(Spanned {
                    node: self.stmt()?,
                    span,
                });
            }
            let end = self.expect(TokenKind::Symbol('}'))?.span.end;
            (body, end)
        };
        Ok(Function {
            exported: false,
            visibility: Visibility::Public,
            rust_name: None,
            rust_import: None,
            return_type,
            name,
            generics,
            bounds,
            is_async,
            throws,
            throws_type,
            test_name: None,
            params,
            body,
            span: Span { start, end },
        })
    }
    fn stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.is(&TokenKind::Symbol('@')) {
            self.bump();
            let (annotation, _) = self.ident()?;
            if annotation != "unsafeRust" && annotation != "rustBlock" {
                return Err(ParseError {
                    message: format!("unknown statement annotation @{annotation}"),
                    span: self.current().span,
                });
            }
            let Token {
                kind: TokenKind::RawRust(raw),
                ..
            } = self.bump()
            else {
                return Err(ParseError {
                    message: format!("@{annotation} must annotate a rust block"),
                    span: self.current().span,
                });
            };
            if annotation == "unsafeRust" {
                Ok(Stmt::Rust(format!("unsafe {{ {raw} }}")))
            } else {
                Ok(Stmt::Rust(raw))
            }
        } else if self.is(&TokenKind::Keyword("throw".into())) {
            self.bump();
            let value = self.expr()?;
            self.expect(TokenKind::Symbol(';'))?;
            Ok(Stmt::Throw(value))
        } else if self.is(&TokenKind::Keyword("drop".into())) {
            self.bump();
            let value = self.primary()?;
            self.expect(TokenKind::Symbol(';'))?;
            Ok(Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Name("drop".into())),
                args: vec![value],
            }))
        } else if matches!(self.current().kind, TokenKind::RawRust(_)) {
            let Token {
                kind: TokenKind::RawRust(raw),
                ..
            } = self.bump()
            else {
                unreachable!()
            };
            Ok(Stmt::Rust(raw))
        } else if self.is(&TokenKind::Keyword("try".into())) {
            self.bump();
            let body = self.block()?;
            self.expect(TokenKind::Keyword("catch".into()))?;
            self.expect(TokenKind::Symbol('('))?;
            let (error, _) = self.ident()?;
            self.expect(TokenKind::Symbol(')'))?;
            let catch_body = self.block()?;
            Ok(Stmt::TryCatch {
                body,
                error,
                catch_body,
            })
        } else if self.is(&TokenKind::Keyword("if".into())) {
            self.bump();
            self.expect(TokenKind::Symbol('('))?;
            let value = self.expr()?;
            if self.is(&TokenKind::Keyword("case".into())) {
                self.bump();
                let (pattern, bindings) = self.pattern()?;
                self.expect(TokenKind::Symbol(')'))?;
                let then_body = self.block()?;
                let else_body = if self.is(&TokenKind::Keyword("else".into())) {
                    self.bump();
                    self.block()?
                } else {
                    Vec::new()
                };
                return Ok(Stmt::IfCase {
                    value,
                    pattern,
                    bindings,
                    then_body,
                    else_body,
                });
            }
            let condition = value;
            self.expect(TokenKind::Symbol(')'))?;
            let then_body = self.block()?;
            let else_body = if self.is(&TokenKind::Keyword("else".into())) {
                self.bump();
                self.block()?
            } else {
                Vec::new()
            };
            Ok(Stmt::If {
                condition,
                then_body,
                else_body,
            })
        } else if self.is(&TokenKind::Keyword("await".into()))
            && self.tokens.get(self.pos + 1).map(|token| &token.kind)
                == Some(&TokenKind::Keyword("for".into()))
        {
            self.bump();
            self.bump();
            self.expect(TokenKind::Symbol('('))?;
            self.expect(TokenKind::Keyword("var".into()))?;
            let (name, _) = self.ident()?;
            let (in_keyword, _) = self.ident()?;
            if in_keyword != "in" {
                return Err(ParseError {
                    message: "expected `in` in await for loop".into(),
                    span: self.current().span,
                });
            }
            let stream = self.expr()?;
            self.expect(TokenKind::Symbol(')'))?;
            Ok(Stmt::AwaitFor {
                name,
                stream,
                body: self.block()?,
            })
        } else if self.is(&TokenKind::Keyword("while".into())) {
            self.bump();
            self.expect(TokenKind::Symbol('('))?;
            let condition = self.expr()?;
            if self.is(&TokenKind::Keyword("case".into())) {
                self.bump();
                let (pattern, _) = self.pattern()?;
                self.expect(TokenKind::Symbol(')'))?;
                return Ok(Stmt::WhileCase {
                    value: condition,
                    pattern,
                    body: self.block()?,
                });
            }
            self.expect(TokenKind::Symbol(')'))?;
            Ok(Stmt::While {
                condition,
                body: self.block()?,
            })
        } else if self.is(&TokenKind::Keyword("for".into())) {
            self.bump();
            self.expect(TokenKind::Symbol('('))?;
            if self.is(&TokenKind::Keyword("var".into()))
                && matches!(
                    self.tokens.get(self.pos + 2).map(|token| &token.kind),
                    Some(TokenKind::Ident(value)) if value == "in"
                )
            {
                self.bump();
                let (name, _) = self.ident()?;
                self.ident()?;
                let iterable = self.expr()?;
                self.expect(TokenKind::Symbol(')'))?;
                return Ok(Stmt::ForIn {
                    name,
                    iterable,
                    body: self.block()?,
                });
            }
            let initializer = if self.is(&TokenKind::Symbol(';')) {
                self.bump();
                None
            } else {
                Some(Box::new(self.stmt()?))
            };
            let condition = if self.is(&TokenKind::Symbol(';')) {
                self.bump();
                None
            } else {
                let e = self.expr()?;
                self.expect(TokenKind::Symbol(';'))?;
                Some(e)
            };
            let step = if self.is(&TokenKind::Symbol(')')) {
                None
            } else {
                Some(self.expr()?)
            };
            self.expect(TokenKind::Symbol(')'))?;
            Ok(Stmt::For {
                initializer,
                condition,
                step,
                body: self.block()?,
            })
        } else if self.is(&TokenKind::Keyword("switch".into())) {
            self.bump();
            self.expect(TokenKind::Symbol('('))?;
            let value = self.expr()?;
            self.expect(TokenKind::Symbol(')'))?;
            self.expect(TokenKind::Symbol('{'))?;
            let mut cases = Vec::new();
            while !self.is(&TokenKind::Symbol('}')) {
                let start = self.current().span.start;
                let (pattern, bindings) = if self.is(&TokenKind::Keyword("default".into())) {
                    self.bump();
                    ("_".into(), Vec::new())
                } else {
                    self.expect(TokenKind::Keyword("case".into()))?;
                    self.pattern()?
                };
                let guard = if self.is(&TokenKind::Keyword("when".into())) {
                    self.bump();
                    Some(self.pattern_guard()?)
                } else {
                    None
                };
                self.expect(TokenKind::Symbol(':'))?;
                let body = self.block()?;
                let end = self.tokens[self.pos.saturating_sub(1)].span.end;
                cases.push(SwitchCase {
                    pattern,
                    bindings,
                    guard,
                    body,
                    span: Span { start, end },
                });
            }
            self.bump();
            Ok(Stmt::Switch { value, cases })
        } else if self.is(&TokenKind::Keyword("var".into())) {
            self.bump();
            if self.is(&TokenKind::Symbol('(')) {
                self.bump();
                let mut bindings = Vec::new();
                while !self.is(&TokenKind::Symbol(')')) {
                    bindings.push(self.ident()?.0);
                    if !self.is(&TokenKind::Symbol(')')) {
                        self.expect(TokenKind::Symbol(','))?;
                    }
                }
                self.bump();
                self.expect(TokenKind::Operator("=".into()))?;
                let initializer = self.expr()?;
                self.expect(TokenKind::Symbol(';'))?;
                return Ok(Stmt::Var {
                    name: format!("__orust_tuple_pattern__{}", bindings.join(",")),
                    initializer,
                    declared_type: None,
                });
            }
            if self.is(&TokenKind::Symbol('[')) {
                let (pattern, _) = self.pattern()?;
                self.expect(TokenKind::Operator("=".into()))?;
                let initializer = self.expr()?;
                self.expect(TokenKind::Keyword("else".into()))?;
                return Ok(Stmt::PatternVar {
                    pattern,
                    initializer,
                    else_body: self.block()?,
                });
            }
            let (name, _) = self.ident()?;
            let is_pattern = self.is(&TokenKind::Symbol('(')) || self.is(&TokenKind::Symbol('@'));
            if is_pattern {
                let mut pattern = name;
                if self.is(&TokenKind::Symbol('(')) {
                    self.bump();
                    pattern.push('(');
                    while !self.is(&TokenKind::Symbol(')')) {
                        pattern.push_str(&token_text(&self.bump().kind));
                        if !self.is(&TokenKind::Symbol(')')) {
                            self.expect(TokenKind::Symbol(','))?;
                            pattern.push_str(", ");
                        }
                    }
                    self.bump();
                    pattern.push(')');
                } else {
                    self.bump();
                    pattern.push_str(" @ ");
                    pattern.push_str(&self.pattern_atom()?);
                }
                self.expect(TokenKind::Operator("=".into()))?;
                let initializer = self.expr()?;
                self.expect(TokenKind::Keyword("else".into()))?;
                return Ok(Stmt::PatternVar {
                    pattern,
                    initializer,
                    else_body: self.block()?,
                });
            }
            self.expect(TokenKind::Operator("=".into()))?;
            let initializer = self.expr()?;
            self.expect(TokenKind::Symbol(';'))?;
            Ok(Stmt::Var {
                name,
                initializer,
                declared_type: None,
            })
        } else if self.starts_typed_local() {
            let declared_type = self.ty()?;
            let (name, _) = self.ident()?;
            self.expect(TokenKind::Operator("=".into()))?;
            let initializer = self.expr()?;
            self.expect(TokenKind::Symbol(';'))?;
            Ok(Stmt::Var {
                name,
                initializer,
                declared_type: Some(declared_type),
            })
        } else if self.is(&TokenKind::Keyword("print".into())) {
            self.bump();
            self.expect(TokenKind::Symbol('('))?;
            let e = self.expr()?;
            self.expect(TokenKind::Symbol(')'))?;
            self.expect(TokenKind::Symbol(';'))?;
            Ok(Stmt::Print(e))
        } else if self.is(&TokenKind::Keyword("return".into())) {
            self.bump();
            let e = if self.is(&TokenKind::Symbol(';')) {
                None
            } else {
                Some(self.expr()?)
            };
            self.expect(TokenKind::Symbol(';'))?;
            Ok(Stmt::Return(e))
        } else {
            let e = self.expr()?;
            self.expect(TokenKind::Symbol(';'))?;
            Ok(Stmt::Expr(e))
        }
    }
    fn block(&mut self) -> Result<Vec<Spanned<Stmt>>, ParseError> {
        self.expect(TokenKind::Symbol('{'))?;
        let mut body = Vec::new();
        while !self.is(&TokenKind::Symbol('}')) {
            let span = self.current().span;
            body.push(Spanned {
                node: self.stmt()?,
                span,
            });
        }
        self.expect(TokenKind::Symbol('}'))?;
        Ok(body)
    }
    fn pattern(&mut self) -> Result<(String, Vec<String>), ParseError> {
        let mut pattern = self.pattern_atom()?;
        let mut bindings = Vec::new();
        if let Some(open) = pattern.find('(') {
            let inner = pattern[open + 1..].strip_suffix(')').unwrap_or("");
            bindings.extend(
                inner
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty() && *value != "_")
                    .map(str::to_string),
            );
        }
        while self.is(&TokenKind::Symbol('|')) {
            self.bump();
            pattern.push_str(" | ");
            pattern.push_str(&self.pattern_atom()?);
        }
        Ok((pattern, bindings))
    }

    fn pattern_atom(&mut self) -> Result<String, ParseError> {
        let first = self.bump();
        let mut value = token_text(&first.kind);
        if value == "var" {
            let binding = self.ident()?.0;
            value = format!("Some({binding})");
            if self.is(&TokenKind::Symbol('?')) {
                self.bump();
            }
        } else if value == "null" {
            value = "None".into();
        } else if first.kind == TokenKind::Symbol('[') {
            while !self.is(&TokenKind::Symbol(']')) {
                if self.is(&TokenKind::Symbol(',')) {
                    self.bump();
                    value.push_str(", ");
                    continue;
                }
                if self.is(&TokenKind::Operator("..".into())) {
                    self.bump();
                    let binding = self.ident()?.0;
                    value.push_str(&format!("{binding} @ .."));
                } else {
                    value.push_str(&token_text(&self.bump().kind));
                }
            }
            self.bump();
            value.push(']');
            return Ok(value);
        }
        if self.is(&TokenKind::Symbol('(')) {
            self.bump();
            value.push('(');
            while !self.is(&TokenKind::Symbol(')')) {
                value.push_str(&token_text(&self.bump().kind));
                if !self.is(&TokenKind::Symbol(')')) {
                    self.expect(TokenKind::Symbol(','))?;
                    value.push_str(", ");
                }
            }
            self.bump();
            value.push(')');
        } else if self.is(&TokenKind::Symbol('@')) {
            self.bump();
            value.push_str(" @ ");
            value.push_str(&self.pattern_atom()?);
        } else if self.is(&TokenKind::Operator("..".into()))
            || self.is(&TokenKind::Operator("..=".into()))
        {
            let operator = token_text(&self.bump().kind);
            let end = self.bump();
            value.push_str(&operator);
            value.push_str(&token_text(&end.kind));
        }
        Ok(value)
    }

    fn pattern_guard(&mut self) -> Result<String, ParseError> {
        let mut guard = String::new();
        while !self.is(&TokenKind::Symbol(':')) {
            if !guard.is_empty() {
                guard.push(' ');
            }
            guard.push_str(&token_text(&self.bump().kind));
        }
        Ok(guard)
    }
    fn expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.primary()?;
        while let TokenKind::Operator(op) = &self.current().kind {
            let op = op.clone();
            if op == ".." || op == "..=" {
                break;
            }
            self.bump();
            let right = self.primary()?;
            left = match op.as_str() {
                "??" => Expr::Coalesce {
                    left: Box::new(left),
                    right: Box::new(right),
                },
                "?." => match right {
                    Expr::Name(name) => Expr::OptionalMember {
                        object: Box::new(left),
                        name,
                    },
                    _ => {
                        return Err(ParseError {
                            message: "expected member after ?.".into(),
                            span: self.current().span,
                        })
                    }
                },
                _ => Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
            };
        }
        Ok(left)
    }
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let mut e = match self.bump() {
            Token {
                kind: TokenKind::Number(n),
                ..
            } if n.contains('.') => Expr::Float(parse_float_literal(&n)),
            Token {
                kind: TokenKind::Number(n),
                ..
            } => Expr::Int(parse_integer_literal(&n)),
            Token {
                kind: TokenKind::String(s),
                ..
            } => Expr::String(s),
            Token {
                kind: TokenKind::Keyword(k),
                ..
            } if k == "true" || k == "false" => Expr::Bool(k == "true"),
            Token {
                kind: TokenKind::Keyword(k),
                ..
            } if k == "null" => Expr::Null,
            Token {
                kind: TokenKind::Keyword(k),
                ..
            } if k == "await" => Expr::Await(Box::new(self.primary()?)),
            Token {
                kind: TokenKind::Keyword(k),
                ..
            } if k == "spawn" => Expr::Spawn(Box::new(self.primary()?)),
            Token {
                kind: TokenKind::Keyword(k),
                ..
            } if k == "Future" => Expr::Name(k),
            Token {
                kind: TokenKind::Symbol('['),
                ..
            } => {
                let mut values = Vec::new();
                while !self.is(&TokenKind::Symbol(']')) {
                    values.push(self.expr()?);
                    if !self.is(&TokenKind::Symbol(']')) {
                        self.expect(TokenKind::Symbol(','))?;
                    }
                }
                self.expect(TokenKind::Symbol(']'))?;
                Expr::List(values)
            }
            Token {
                kind: TokenKind::Symbol('('),
                ..
            } => {
                let is_record = matches!(
                    (self.tokens.get(self.pos), self.tokens.get(self.pos + 1)),
                    (
                        Some(Token {
                            kind: TokenKind::Ident(_),
                            ..
                        }),
                        Some(Token {
                            kind: TokenKind::Symbol(':'),
                            ..
                        })
                    )
                );
                if is_record {
                    let mut fields = Vec::new();
                    while !self.is(&TokenKind::Symbol(')')) {
                        let (name, _) = self.ident()?;
                        self.expect(TokenKind::Symbol(':'))?;
                        fields.push(Expr::NamedArg {
                            name,
                            value: Box::new(self.expr()?),
                        });
                        if !self.is(&TokenKind::Symbol(')')) {
                            self.expect(TokenKind::Symbol(','))?;
                        }
                    }
                    self.bump();
                    Expr::NewArgs {
                        name: "__orust_record__".into(),
                        args: fields,
                    }
                } else {
                    let mut values = Vec::new();
                    while !self.is(&TokenKind::Symbol(')')) {
                        values.push(self.expr()?);
                        if !self.is(&TokenKind::Symbol(')')) {
                            self.expect(TokenKind::Symbol(','))?;
                        }
                    }
                    self.bump();
                    if values.len() == 1 {
                        values.pop().unwrap()
                    } else {
                        Expr::NewArgs {
                            name: "__orust_tuple__".into(),
                            args: values,
                        }
                    }
                }
            }
            Token {
                kind: TokenKind::Symbol('|'),
                ..
            } => {
                let mut params = Vec::new();
                while !self.is(&TokenKind::Symbol('|')) {
                    params.push(self.ident()?.0);
                    if !self.is(&TokenKind::Symbol('|')) {
                        self.expect(TokenKind::Symbol(','))?;
                    }
                }
                self.bump();
                let body = if self.is(&TokenKind::Symbol('{')) {
                    ClosureBody::Block(self.block()?)
                } else {
                    ClosureBody::Expr(Box::new(self.expr()?))
                };
                Expr::Closure { params, body }
            }
            Token {
                kind: TokenKind::Keyword(k),
                ..
            } if k == "new" => {
                let (n, _) = self.ident()?;
                let constructor = if self.is(&TokenKind::Symbol('.')) {
                    self.bump();
                    Some(self.ident()?.0)
                } else {
                    None
                };
                self.expect(TokenKind::Symbol('('))?;
                let mut args = Vec::new();
                while !self.is(&TokenKind::Symbol(')')) {
                    let named = matches!(
                        (self.tokens.get(self.pos), self.tokens.get(self.pos + 1)),
                        (
                            Some(Token {
                                kind: TokenKind::Ident(_),
                                ..
                            }),
                            Some(Token {
                                kind: TokenKind::Symbol(':'),
                                ..
                            })
                        )
                    );
                    if named {
                        let (name, _) = self.ident()?;
                        self.expect(TokenKind::Symbol(':'))?;
                        args.push(Expr::NamedArg {
                            name,
                            value: Box::new(self.expr()?),
                        });
                    } else {
                        args.push(self.expr()?);
                    }
                    if !self.is(&TokenKind::Symbol(')')) {
                        self.expect(TokenKind::Symbol(','))?;
                    }
                }
                self.bump();
                let target = constructor.map(|name| format!("{n}.{name}")).unwrap_or(n);
                if args.is_empty() {
                    Expr::New(target)
                } else {
                    Expr::NewArgs { name: target, args }
                }
            }
            Token {
                kind: TokenKind::Keyword(k),
                ..
            } if k == "this" => Expr::This,
            Token {
                kind: TokenKind::RawRustExpr(value),
                ..
            } => Expr::Name(format!("__orust_rust_expr__{value}")),
            Token {
                kind: TokenKind::Keyword(k),
                ..
            } if k == "copy" => Expr::Copy(Box::new(self.primary()?)),
            Token {
                kind: TokenKind::Keyword(k),
                ..
            } if k == "lend" => {
                let mutable = if self.is(&TokenKind::Keyword("mut".into())) {
                    self.bump();
                    true
                } else {
                    false
                };
                Expr::Borrow {
                    mutable,
                    value: Box::new(self.primary()?),
                }
            }
            Token {
                kind: TokenKind::Ident(s),
                ..
            } => Expr::Name(s),
            Token {
                kind: TokenKind::Keyword(s),
                ..
            } if matches!(s.as_str(), "show" | "hide" | "as" | "private" | "internal") => {
                Expr::Name(s)
            }
            t => {
                return Err(ParseError {
                    message: "expected expression".into(),
                    span: t.span,
                })
            }
        };
        loop {
            if self.is(&TokenKind::Symbol('.')) {
                self.bump();
                let name = if self.is(&TokenKind::Symbol('$')) {
                    self.bump();
                    let token = self.bump();
                    let TokenKind::Number(index) = token.kind else {
                        return Err(ParseError {
                            message: "expected tuple position after `$`".into(),
                            span: token.span,
                        });
                    };
                    format!("${index}")
                } else {
                    let mut name = self.ident()?.0;
                    if self.is(&TokenKind::Operator("<".into())) {
                        self.bump();
                        let mut type_args = Vec::new();
                        while !self.is(&TokenKind::Operator(">".into())) {
                            type_args.push(self.ty()?);
                            if !self.is(&TokenKind::Operator(">".into())) {
                                self.expect(TokenKind::Symbol(','))?;
                            }
                        }
                        self.bump();
                        name.push('<');
                        name.push_str(&type_args.join(","));
                        name.push('>');
                    }
                    name
                };
                e = Expr::Member {
                    object: Box::new(e),
                    name,
                };
            } else if self.is(&TokenKind::Symbol('$')) {
                self.bump();
                let token = self.bump();
                let TokenKind::Number(index) = token.kind else {
                    return Err(ParseError {
                        message: "expected tuple position after `$`".into(),
                        span: token.span,
                    });
                };
                e = Expr::Member {
                    object: Box::new(e),
                    name: format!("${index}"),
                };
            } else if self.is(&TokenKind::Symbol('(')) {
                self.bump();
                let mut args = Vec::new();
                while !self.is(&TokenKind::Symbol(')')) {
                    let named = matches!(
                        (self.tokens.get(self.pos), self.tokens.get(self.pos + 1)),
                        (
                            Some(Token {
                                kind: TokenKind::Ident(_),
                                ..
                            }),
                            Some(Token {
                                kind: TokenKind::Symbol(':'),
                                ..
                            })
                        )
                    );
                    if named {
                        let (name, _) = self.ident()?;
                        self.expect(TokenKind::Symbol(':'))?;
                        args.push(Expr::NamedArg {
                            name,
                            value: Box::new(self.expr()?),
                        });
                    } else {
                        args.push(self.expr()?);
                    }
                    if !self.is(&TokenKind::Symbol(')')) {
                        self.expect(TokenKind::Symbol(','))?;
                    }
                }
                self.bump();
                e = Expr::Call {
                    callee: Box::new(e),
                    args,
                };
            } else if self.is(&TokenKind::Symbol('[')) {
                self.bump();
                let start = if self.is(&TokenKind::Operator("..".into()))
                    || self.is(&TokenKind::Operator("..=".into()))
                {
                    None
                } else {
                    Some(Box::new(self.expr()?))
                };
                if self.is(&TokenKind::Operator("..".into()))
                    || self.is(&TokenKind::Operator("..=".into()))
                {
                    let inclusive = self.is(&TokenKind::Operator("..=".into()));
                    self.bump();
                    let end = if self.is(&TokenKind::Symbol(']')) {
                        None
                    } else {
                        Some(Box::new(self.expr()?))
                    };
                    self.expect(TokenKind::Symbol(']'))?;
                    e = Expr::Slice {
                        object: Box::new(e),
                        start,
                        end,
                        inclusive,
                    };
                    continue;
                }
                let Some(index) = start else {
                    return Err(ParseError {
                        message: "expected slice range or index".into(),
                        span: self.current().span,
                    });
                };
                self.expect(TokenKind::Symbol(']'))?;
                e = Expr::Index {
                    object: Box::new(e),
                    index,
                };
            } else {
                break;
            }
        }
        Ok(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_class_and_preserves_spans() {
        let source = "class Box { int size = 1; }";
        let program = parse(source).unwrap();
        assert_eq!(program.items.len(), 1);
        assert_eq!(
            program.items[0].span,
            Span {
                start: 0,
                end: source.len()
            }
        );
    }

    #[test]
    fn reports_unterminated_strings_with_a_span() {
        let error = parse("void main() { print(\"oops); }").unwrap_err();
        assert_eq!(error.message, "unterminated string");
        assert_eq!(error.span.start, 20);
    }

    #[test]
    fn reports_missing_semicolons() {
        let error = parse("void main() { print(1) }").unwrap_err();
        assert!(error.message.contains("expected Symbol(';')"));
    }

    #[test]
    fn parses_borrowing_vocabulary() {
        let program = parse("void show(lend mut Box b) { print(copy b); print(lend b); }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        assert_eq!(function.params[0].ty, "&mut Box");
        assert!(matches!(function.body[0].node, Stmt::Print(Expr::Copy(_))));
        assert!(matches!(
            function.body[1].node,
            Stmt::Print(Expr::Borrow { mutable: false, .. })
        ));
    }

    #[test]
    fn parses_interfaces_nullable_types_and_option_operators() {
        let program = parse("interface Speak { String speak(); } class Dog implements Speak { String? name; } void main() { print(null ?? \"fallback\"); }").unwrap();
        assert!(matches!(program.items[0].node, Item::Interface(_)));
        let Item::Class(class) = &program.items[1].node else {
            panic!("expected class")
        };
        assert_eq!(class.implements, vec!["Speak"]);
        assert_eq!(class.fields[0].ty, "String?");
        assert!(matches!(program.items[2].node, Item::Function(_)));
    }

    #[test]
    fn parses_lifetime_escape_hatch_and_rust_passthrough() {
        let program =
            parse("lend(a) int first(lend int a) { rust { let x = 1; } return a; }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        assert_eq!(function.return_type, "int");
        assert!(matches!(function.body[0].node, Stmt::Rust(_)));
    }

    #[test]
    fn parses_async_try_catch() {
        let program = parse(
            "Future<int> risky() async throws { return 1; } async void main() { try { await risky(); } catch (error) { print(\"failed\"); } }",
        )
        .unwrap();
        let Item::Function(function) = &program.items[1].node else {
            panic!("expected function")
        };
        assert!(matches!(function.body[0].node, Stmt::TryCatch { .. }));
    }

    #[test]
    fn parses_interface_parameters_async_methods_and_defaults() {
        let program = parse(
            "interface Speak { String speak(lend String prefix); Future<int> count() async { return 1; } }",
        )
        .unwrap();
        let Item::Interface(interface) = &program.items[0].node else {
            panic!("expected interface")
        };
        assert_eq!(interface.methods[0].params[0].ty, "&String");
        assert!(interface.methods[1].is_async);
        assert!(!interface.methods[1].body.is_empty());
    }

    #[test]
    fn parses_generic_enum_and_switch_patterns() {
        let program = parse(
            "enum Maybe<T> { Some(T), None } void inspect(Maybe<int> value) { switch (value) { case Some(item): { print(item); } default: { print(0); } } }",
        )
        .unwrap();
        assert!(matches!(program.items[0].node, Item::Enum(_)));
        let Item::Function(function) = &program.items[1].node else {
            panic!("expected function")
        };
        assert!(matches!(function.body[0].node, Stmt::Switch { .. }));
    }

    #[test]
    fn parses_slice_ranges() {
        let program = parse(
            "void main() { var a = xs[1..4]; var b = xs[..3]; var c = xs[2..]; var d = xs[1..=3]; }",
        )
        .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function");
        };
        assert!(matches!(
            function.body[0].node,
            Stmt::Var {
                initializer: Expr::Slice {
                    inclusive: false,
                    start: Some(_),
                    end: Some(_),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            function.body[3].node,
            Stmt::Var {
                initializer: Expr::Slice {
                    inclusive: true,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn parses_for_in_loops() {
        let program = parse("void main() { for (var item in items) { print(item); } }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function");
        };
        assert!(matches!(&function.body[0].node, Stmt::ForIn { name, .. } if name == "item"));
    }

    #[test]
    fn parses_interface_extensions() {
        let program = parse("interface Pet extends Animal, Named { void play(); }").unwrap();
        let Item::Interface(interface) = &program.items[0].node else {
            panic!("expected interface");
        };
        assert_eq!(interface.extends, vec!["Animal", "Named"]);
    }

    #[test]
    fn parses_interface_associated_types() {
        let program = parse("interface Container { type Item; Item get(int index); }").unwrap();
        let Item::Interface(interface) = &program.items[0].node else {
            panic!("expected interface");
        };
        assert_eq!(interface.associated_types, vec!["Item"]);
        assert_eq!(interface.methods[0].return_type, "Item");
    }

    #[test]
    fn parses_aliases_and_newtypes() {
        let program = parse("typedef UserId = int; type EmailId(String);").unwrap();
        assert!(
            matches!(&program.items[0].node, Item::TypeAlias(alias) if alias.name == "UserId" && alias.ty == "int")
        );
        assert!(
            matches!(&program.items[1].node, Item::Newtype(newtype) if newtype.name == "EmailId" && newtype.inner == "String")
        );
    }

    #[test]
    fn parses_generic_function_type_aliases() {
        let program = parse("typedef Callback<T> = void Function(T); ").unwrap();
        let Item::TypeAlias(alias) = &program.items[0].node else {
            panic!("expected type alias");
        };
        assert_eq!(alias.generics, vec!["T"]);
        assert_eq!(alias.ty, "function(T)");
    }

    #[test]
    fn accepts_named_enum_payload_fields() {
        let program = parse("enum Expr { Num(int n), Add(Expr left, Expr right) }").unwrap();
        let Item::Enum(enum_decl) = &program.items[0].node else {
            panic!("expected enum");
        };
        assert_eq!(enum_decl.variants[0].fields, vec!["int"]);
        assert_eq!(enum_decl.variants[1].fields, vec!["Expr", "Expr"]);
    }

    #[test]
    fn parses_named_record_types_and_literals() {
        let program =
            parse("void main() { ({int x, String label}) point = (x: 1, label: \"ok\"); }")
                .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        let Stmt::Var {
            declared_type: Some(ty),
            initializer: Expr::NewArgs { name, .. },
            ..
        } = &function.body[0].node
        else {
            panic!("expected record local")
        };
        assert_eq!(ty, "record{x:int,label:String}");
        assert_eq!(name, "__orust_record__");
    }

    #[test]
    fn parses_tuple_destructuring_declarations() {
        let program = parse("void main() { var (left, right) = (1, 2); }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        let Stmt::Var { name, .. } = &function.body[0].node else {
            panic!("expected variable")
        };
        assert_eq!(name, "__orust_tuple_pattern__left,right");
    }

    #[test]
    fn parses_radix_literals_separators_and_numeric_suffixes() {
        let program = parse("void main() { u8 a = 250u8; int b = 0xff; int c = 0b1010_0001; double d = 5.0f32; print(a + b + c); print(d); }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        let values = function
            .body
            .iter()
            .filter_map(|statement| match &statement.node {
                Stmt::Var { initializer, .. } => Some(initializer),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(matches!(values[0], Expr::Int(250)));
        assert!(matches!(values[1], Expr::Int(255)));
        assert!(matches!(values[2], Expr::Int(161)));
        assert!(matches!(values[3], Expr::Float(value) if (*value - 5.0).abs() < f64::EPSILON));
    }

    #[test]
    fn parses_range_binding_patterns() {
        let program = parse("void inspect(int value) { switch (value) { case n @ 1..=5: { print(n); } default: { print(0); } } }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        let Stmt::Switch { cases, .. } = &function.body[0].node else {
            panic!("expected switch")
        };
        assert_eq!(cases[0].pattern, "n @ 1..=5");
    }

    #[test]
    fn parses_list_rest_patterns() {
        let program = parse("void inspect(List<int> values) { switch (values) { case [first, ..rest]: { print(first); } default: { print(0); } } }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        let Stmt::Switch { cases, .. } = &function.body[0].node else {
            panic!("expected switch")
        };
        assert_eq!(cases[0].pattern, "[first, rest @ ..]");
    }

    #[test]
    fn parses_pattern_guards_or_patterns_and_ranges() {
        let program = parse(
            "void inspect(int value) { switch (value) { case 1..=5: { print(value); } case 6 | 7 when value > 0: { print(value); } default: { print(0); } } }",
        )
        .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function");
        };
        let Stmt::Switch { cases, .. } = &function.body[0].node else {
            panic!("expected switch");
        };
        assert_eq!(cases[0].pattern, "1..=5");
        assert_eq!(cases[1].pattern, "6 | 7");
        assert_eq!(cases[1].guard.as_deref(), Some("value > 0"));
    }

    #[test]
    fn parses_nullable_patterns() {
        let program = parse(
            "void inspect(int? value) { switch (value) { case null: { print(0); } case var item: { print(item); } } }",
        )
        .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function");
        };
        let Stmt::Switch { cases, .. } = &function.body[0].node else {
            panic!("expected switch");
        };
        assert_eq!(cases[0].pattern, "None");
        assert_eq!(cases[1].pattern, "Some(item)");
    }

    #[test]
    fn parses_while_case_patterns() {
        let program = parse(
            "void consume(Iterator<int> it) { while (it.next() case var item?) { print(item); } }",
        )
        .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function");
        };
        let Stmt::WhileCase { pattern, .. } = &function.body[0].node else {
            panic!("expected while case");
        };
        assert_eq!(pattern, "Some(item)");
    }

    #[test]
    fn parses_refutable_pattern_bindings_with_else() {
        let program = parse(
            "void consume(Shape shape) { var Circle(radius) = shape else { return; } print(radius); }",
        )
        .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function");
        };
        let Stmt::PatternVar {
            pattern, else_body, ..
        } = &function.body[0].node
        else {
            panic!("expected pattern binding");
        };
        assert_eq!(pattern, "Circle(radius)");
        assert!(matches!(else_body[0].node, Stmt::Return(None)));
    }

    #[test]
    fn parses_list_pattern_bindings_with_else() {
        let program = parse(
            "void consume(List<int> values) { var [first, ..rest] = values else { return; } print(first); }",
        )
        .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function");
        };
        let Stmt::PatternVar { pattern, .. } = &function.body[0].node else {
            panic!("expected pattern binding");
        };
        assert_eq!(pattern, "[first, rest @ ..]");
    }

    #[test]
    fn parses_data_annotation() {
        let program = parse("@data class User { String name; }").unwrap();
        let Item::Class(class) = &program.items[0].node else {
            panic!("expected class")
        };
        assert!(class.data);
    }

    #[test]
    fn parses_await_for() {
        let program =
            parse("async void main() { await for (var value in values()) { print(value); } }")
                .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        assert!(matches!(function.body[0].node, Stmt::AwaitFor { .. }));
    }

    #[test]
    fn parses_extensions() {
        let program = parse("extension on String { int length() { return 1; } }").unwrap();
        assert!(matches!(program.items[0].node, Item::Extension(_)));
    }

    #[test]
    fn parses_expression_and_block_closures() {
        let program = parse(
            "async void main() { var inc = |value| value + 1; var task = spawn |value| { print(value); }; }",
        )
        .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        assert!(matches!(
            function.body[0].node,
            Stmt::Var {
                initializer: Expr::Closure { .. },
                ..
            }
        ));
        assert!(matches!(
            function.body[1].node,
            Stmt::Var {
                initializer: Expr::Spawn(_),
                ..
            }
        ));
    }

    #[test]
    fn parses_module_directives_and_exported_items() {
        let program = parse("import 'models/user.or' as users; export class User {}\n").unwrap();
        assert_eq!(program.imports[0].path, "models/user.or");
        assert_eq!(program.imports[0].alias.as_deref(), Some("users"));
        let Item::Class(class) = &program.items[0].node else {
            panic!("expected class")
        };
        assert!(class.exported);
    }

    #[test]
    fn parses_top_level_rust_use_declarations() {
        let program = parse(
            "rust use serde_json::Value; rust use tokio::time::sleep as pause; rust use std::fmt::{self, Display}; void main() {}",
        )
        .unwrap();
        assert_eq!(
            program.rust_uses,
            vec![
                "serde_json::Value".to_string(),
                "tokio::time::sleep as pause".to_string(),
                "std::fmt::{self, Display}".to_string()
            ]
        );
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn parses_rust_let_as_a_shared_scope_statement() {
        let program = parse(
            "void main() { rust let mut value: serde_json::Value = serde_json::json!({ \"ok\": true }); print(value); }",
        )
        .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        assert!(
            matches!(function.body[0].node, Stmt::Rust(ref raw) if raw.starts_with("let mut value: serde_json::Value") && raw.ends_with(';'))
        );
    }

    #[test]
    fn parses_unsafe_rust_blocks_as_explicit_unsafe_scopes() {
        let program =
            parse("void main() { @unsafeRust rust { native_handle.release(); } }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        assert!(
            matches!(function.body[0].node, Stmt::Rust(ref raw) if raw == "unsafe {  native_handle.release();  }")
        );
    }

    #[test]
    fn parses_explicit_shared_rust_blocks() {
        let program = parse("void main() { @rustBlock rust { value += 1; } }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        assert!(matches!(function.body[0].node, Stmt::Rust(ref raw) if raw == " value += 1; "));
    }

    #[test]
    fn parses_filtered_imports() {
        let program = parse(
            "import 'models/user.or' show User, Admin hide Secret; export 'models/user.or' show User;\n",
        )
        .unwrap();
        assert_eq!(program.imports[0].show, ["User", "Admin"]);
        assert_eq!(program.imports[0].hide, ["Secret"]);
        assert_eq!(program.exports[0].show, ["User"]);
    }

    #[test]
    fn parses_member_visibility() {
        let program = parse(
            "export class User { private String secret; internal int count = 0; String greet() {} }",
        )
        .unwrap();
        let Item::Class(class) = &program.items[0].node else {
            panic!("expected class")
        };
        assert_eq!(class.fields[0].visibility, Visibility::Private);
        assert_eq!(class.fields[1].visibility, Visibility::Internal);
        assert_eq!(class.methods[0].visibility, Visibility::Public);
    }

    #[test]
    fn parses_rust_name_annotation() {
        let program = parse("export @rustName(\"ExternalUser\") class User {}").unwrap();
        let Item::Class(class) = &program.items[0].node else {
            panic!("expected class")
        };
        assert_eq!(class.rust_name.as_deref(), Some("ExternalUser"));
    }

    #[test]
    fn parses_rust_type_alias_annotation() {
        let program = parse("@rustType(\"serde_json::Value\") type JsonValue;").unwrap();
        let Item::TypeAlias(alias) = &program.items[0].node else {
            panic!("expected Rust type alias")
        };
        assert_eq!(alias.name, "JsonValue");
        assert_eq!(alias.ty, "serde_json::Value");
    }

    #[test]
    fn parses_rust_import_type_annotation() {
        let program = parse("export @rustImport(\"serde_json::Value\") type JsonValue;").unwrap();
        let Item::TypeAlias(alias) = &program.items[0].node else {
            panic!("expected Rust import")
        };
        assert_eq!(alias.rust_import.as_deref(), Some("serde_json::Value"));
    }

    #[test]
    fn parses_rust_item_attributes() {
        let program = parse(
            "@derive(\"Serialize\", \"Deserialize\") @repr(\"C\") @cfg(\"unix\") @unsafeRust @orustExport class Config {}",
        )
        .unwrap();
        assert_eq!(program.attributes[0].derives, ["Serialize", "Deserialize"]);
        assert_eq!(program.attributes[0].repr.as_deref(), Some("C"));
        assert_eq!(program.attributes[0].cfg.as_deref(), Some("unix"));
        assert!(program.attributes[0].unsafe_rust);
        assert!(program.attributes[0].orust_export);
        let Item::Class(class) = &program.items[0].node else {
            panic!("expected class")
        };
        assert!(class.exported);
    }

    #[test]
    fn rejects_unsupported_rust_representations() {
        let error = parse("@repr(\"not-a-rust-repr\") class Config {}").unwrap_err();
        assert!(error
            .message
            .contains("unsupported Rust representation `not-a-rust-repr`"));
    }

    #[test]
    fn parses_and_validates_ownership_boundary_attributes() {
        let program = parse(
            "@borrowed(\"'a\") lend String first(lend String value) { return value; } @owned String normalize(String value) { return value; }",
        )
        .unwrap();
        assert_eq!(
            program.attributes[0].borrowed_lifetime.as_deref(),
            Some("'a")
        );
        assert!(!program.attributes[0].owned);
        assert!(program.attributes[1].owned);
        assert!(program.attributes[1].borrowed_lifetime.is_none());
    }

    #[test]
    fn rejects_contradictory_ownership_boundary_attributes() {
        let error =
            parse("@owned lend String first(lend String value) { return value; }").unwrap_err();
        assert!(error.message.contains("@owned cannot be combined"));
    }

    #[test]
    fn parses_rust_expressions_without_parsing_rust_tokens() {
        let program =
            parse("void main() { var value = rust serde_json::json!({ \"ok\": true }); }").unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        let Stmt::Var { initializer, .. } = &function.body[0].node else {
            panic!("expected local")
        };
        assert!(
            matches!(initializer, Expr::Name(name) if name == "__orust_rust_expr__serde_json::json!({ \"ok\": true })")
        );
    }

    #[test]
    fn parses_qualified_rust_types_in_signatures() {
        let program =
            parse("void inspect(serde_json::Value value, tokio::sync::Mutex<String> lock) {}")
                .unwrap();
        let Item::Function(function) = &program.items[0].node else {
            panic!("expected function")
        };
        assert_eq!(function.params[0].ty, "serde_json::Value");
        assert_eq!(function.params[1].ty, "tokio::sync::Mutex<String>");
    }

    #[test]
    fn parses_explicit_drop_statement() {
        let program = parse("void main() { var value = new User(); drop value; }").unwrap();
        assert!(matches!(program.items[0].node, Item::Function(_)));
        let Item::Function(function) = &program.items[0].node else {
            unreachable!()
        };
        assert!(matches!(
            function.body[1].node,
            Stmt::Expr(Expr::Call { .. })
        ));
    }

    #[test]
    fn parses_initializing_and_named_constructors() {
        let program = parse(
            "class Reader { lend String text; Reader(this.text); Reader.empty(this.text) { return; } }",
        )
        .unwrap();
        let Item::Class(class) = &program.items[0].node else {
            panic!("expected class");
        };
        assert_eq!(class.constructors.len(), 2);
        assert_eq!(class.constructors[0].name, "new");
        assert_eq!(class.constructors[0].params[0].ty, "&String");
        assert_eq!(class.constructors[0].initializing_fields, vec!["text"]);
        assert_eq!(class.constructors[1].name, "empty");
    }

    #[test]
    fn rejects_shared_borrowed_classes() {
        let error = parse("shared class Reader { lend String text; }").unwrap_err();
        assert!(error
            .message
            .contains("shared class cannot hold borrowed fields"));
    }

    #[test]
    fn parses_single_case_errors_and_typed_throws() {
        let program = parse(
            "error NotFound { String path } String load(String path) throws NotFound { return path; }",
        )
        .unwrap();
        assert!(matches!(program.items[0].node, Item::Error(_)));
        let Item::Function(function) = &program.items[1].node else {
            panic!("expected function");
        };
        assert_eq!(function.throws_type.as_deref(), Some("NotFound"));
    }

    #[test]
    fn parses_multi_case_errors_with_messages() {
        let program = parse(
            "error LoadError { Missing(String path) => \"missing\"; Permission(String user); }",
        )
        .unwrap();
        let Item::Error(error) = &program.items[0].node else {
            panic!("expected error")
        };
        assert!(error.fields.is_empty());
        assert_eq!(error.cases.len(), 2);
        assert_eq!(error.cases[0].name, "Missing");
        assert_eq!(error.cases[0].message.as_deref(), Some("missing"));
        assert_eq!(error.cases[1].fields[0].name, "user");
    }

    #[test]
    fn parses_named_and_async_tests() {
        let program =
            parse("test \"adds numbers\" { expect(true); } test async \"waits\" { expect(true); }")
                .unwrap();
        let Item::Function(first) = &program.items[0].node else {
            panic!("expected test")
        };
        assert_eq!(first.test_name.as_deref(), Some("adds numbers"));
        assert!(!first.is_async);
        let Item::Function(second) = &program.items[1].node else {
            panic!("expected test")
        };
        assert_eq!(second.test_name.as_deref(), Some("waits"));
        assert!(second.is_async);
    }

    #[test]
    fn parses_factory_constructors_and_validates_borrowed_fields() {
        let program =
            parse("class Reader { lend String text; factory Reader.from(this.text); }").unwrap();
        let Item::Class(class) = &program.items[0].node else {
            panic!("expected class")
        };
        assert!(class.constructors[0].factory);
        assert!(parse("class Reader { lend String text; Reader.empty(); }").is_err());
    }

    #[test]
    fn rejects_invalid_constructor_shapes() {
        let error = parse("class User { User(String name = \"x\", int id); }").unwrap_err();
        assert!(error.message.contains("required parameter after a default"));
        let error = parse("class User { factory User(String name); }").unwrap_err();
        assert!(error
            .message
            .contains("factory constructor must have a named form"));
        let error = parse("class User { User(String name); User(int id); }").unwrap_err();
        assert!(error
            .message
            .contains("declares constructor `new` more than once"));
    }
}
