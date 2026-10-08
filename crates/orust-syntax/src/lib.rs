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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommentKind {
    Line,
    Block,
    OuterDocLine,
    OuterDocBlock,
    InnerDocLine,
    InnerDocBlock,
    Shebang,
}

impl CommentKind {
    pub fn is_doc(self) -> bool {
        matches!(
            self,
            Self::OuterDocLine | Self::OuterDocBlock | Self::InnerDocLine | Self::InnerDocBlock
        )
    }

    pub fn is_outer_doc(self) -> bool {
        matches!(self, Self::OuterDocLine | Self::OuterDocBlock)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    pub kind: CommentKind,
    /// The exact source spelling, including its comment delimiters.
    pub text: String,
    pub span: Span,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocBlock {
    pub raw: String,
    pub summary: String,
    pub body: String,
    pub tags: Vec<DocTag>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocTag {
    Author {
        name: String,
        email: Option<String>,
        url: Option<String>,
    },
    Param {
        name: String,
        desc: String,
    },
    TypeParam {
        name: String,
        desc: String,
    },
    Returns(String),
    Throws {
        ty: String,
        desc: String,
    },
    Panics(String),
    Example {
        title: Option<String>,
        code: String,
    },
    See(String),
    Since(String),
    Deprecated {
        since: Option<String>,
        note: String,
    },
    Version(String),
    Todo(String),
    Note(String),
    Warning(String),
    Internal,
    Category(String),
    InheritDoc,
    License(String),
    Copyright(String),
    Unknown {
        name: String,
        text: String,
    },
}

impl Comment {
    pub fn doc_block(&self) -> Option<DocBlock> {
        self.kind
            .is_doc()
            .then(|| DocBlock::parse(&self.text, self.span))
    }
}

impl DocBlock {
    pub fn parse(text: &str, span: Span) -> Self {
        let content = normalize_doc_text(text);
        let mut body_lines = Vec::new();
        let mut tags: Vec<(String, Vec<String>)> = Vec::new();
        let mut current = None;
        let mut fenced = false;
        for line in content.lines() {
            let line = line.trim_end();
            let trimmed = line.trim_start();
            if !fenced && trimmed.starts_with('@') {
                let value = trimmed[1..].trim_start();
                let (name, rest) = value.split_once(char::is_whitespace).unwrap_or((value, ""));
                tags.push((name.to_owned(), vec![rest.trim_start().to_owned()]));
                current = Some(tags.len() - 1);
            } else if let Some(index) = current {
                tags[index].1.push(line.to_owned());
            } else {
                body_lines.push(line.to_owned());
            }
            if trimmed.starts_with("```") {
                fenced = !fenced;
            }
        }
        let body = body_lines.join("\n").trim().to_owned();
        let summary = body
            .split("\n\n")
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned();
        let tags = tags
            .into_iter()
            .map(|(name, lines)| parse_doc_tag(&name, &lines))
            .collect();
        Self {
            raw: text.to_owned(),
            summary,
            body,
            tags,
            span,
        }
    }
}

fn normalize_doc_text(text: &str) -> String {
    if text.starts_with("///") || text.starts_with("//!") {
        return text
            .lines()
            .map(|line| {
                line.get(3..)
                    .unwrap_or_default()
                    .strip_prefix(' ')
                    .unwrap_or_else(|| line.get(3..).unwrap_or_default())
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    if text.starts_with("/*") {
        let inner = text
            .strip_prefix("/*!")
            .or_else(|| text.strip_prefix("/**"))
            .unwrap_or(text)
            .strip_suffix("*/")
            .unwrap_or(text);
        return inner
            .lines()
            .map(|line| {
                let line = line.trim_start();
                line.strip_prefix('*')
                    .unwrap_or(line)
                    .strip_prefix(' ')
                    .unwrap_or_else(|| line.strip_prefix('*').unwrap_or(line))
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    text.to_owned()
}

fn parse_doc_tag(name: &str, lines: &[String]) -> DocTag {
    let text = lines.join("\n").trim().to_owned();
    match name {
        "author" => parse_author(&text),
        "param" => parse_named_tag(&text, false),
        "typeParam" => parse_named_tag(&text, true),
        "returns" => DocTag::Returns(text),
        "throws" => {
            let (ty, desc) = text.split_once(char::is_whitespace).unwrap_or((&text, ""));
            DocTag::Throws {
                ty: ty.to_owned(),
                desc: desc.trim().to_owned(),
            }
        }
        "panics" => DocTag::Panics(text),
        "example" => {
            let mut lines = text.lines();
            let title = lines.next().filter(|line| !line.trim().starts_with("```"));
            let mut code = Vec::new();
            let mut fenced = false;
            for line in lines {
                if line.trim_start().starts_with("```") {
                    fenced = !fenced;
                    continue;
                }
                if fenced {
                    code.push(line);
                }
            }
            DocTag::Example {
                title: title
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned),
                code: code.join("\n"),
            }
        }
        "see" => DocTag::See(text),
        "since" => DocTag::Since(text),
        "deprecated" => parse_deprecated(&text),
        "version" => DocTag::Version(text),
        "todo" => DocTag::Todo(text),
        "note" => DocTag::Note(text),
        "warning" => DocTag::Warning(text),
        "internal" => DocTag::Internal,
        "category" => DocTag::Category(text),
        "inheritDoc" => DocTag::InheritDoc,
        "license" => DocTag::License(text),
        "copyright" => DocTag::Copyright(text),
        _ => DocTag::Unknown {
            name: name.to_owned(),
            text,
        },
    }
}

fn parse_named_tag(text: &str, type_param: bool) -> DocTag {
    let (name, desc) = text.split_once(char::is_whitespace).unwrap_or((text, ""));
    if type_param {
        DocTag::TypeParam {
            name: name.to_owned(),
            desc: desc.trim().to_owned(),
        }
    } else {
        DocTag::Param {
            name: name.to_owned(),
            desc: desc.trim().to_owned(),
        }
    }
}

fn parse_author(text: &str) -> DocTag {
    let mut name = text.to_owned();
    let mut email = None;
    let mut url = None;
    if let Some(start) = name.find('<') {
        if let Some(end_offset) = name[start + 1..].find('>') {
            let end = start + 1 + end_offset;
            email = Some(name[start + 1..end].to_owned());
            name.replace_range(start..=end, "");
        }
    }
    if let Some(start) = name.rfind('(') {
        if name.ends_with(')') {
            url = Some(name[start + 1..name.len() - 1].to_owned());
            name.truncate(start);
        }
    }
    DocTag::Author {
        name: name.trim().to_owned(),
        email,
        url,
    }
}

fn parse_deprecated(text: &str) -> DocTag {
    let mut rest = text;
    let mut since = None;
    if let Some(value) = rest.strip_prefix("since ") {
        let (version, note) = value.split_once(char::is_whitespace).unwrap_or((value, ""));
        since = Some(version.to_owned());
        rest = note.trim();
    }
    DocTag::Deprecated {
        since,
        note: rest.to_owned(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LintSeverity {
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lint {
    pub code: &'static str,
    pub message: String,
    pub severity: LintSeverity,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    /// The original source, retained so tools can implement lossless modes.
    pub source: String,
    pub bom: bool,
    pub shebang: Option<Comment>,
    pub comments: Vec<Comment>,
    pub lints: Vec<Lint>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    /// Crate-level Rust `use` declarations that should be emitted alongside
    /// the generated ORust items.
    pub rust_uses: Vec<String>,
    pub attributes: Vec<ItemAttributes>,
    pub items: Vec<Spanned<Item>>,
}

impl Program {
    pub fn lossless_source(&self) -> &str {
        &self.source
    }

    /// Recompute documentation lints with project metadata that is not part of
    /// a standalone source file, such as the package version.
    pub fn lint_documentation(
        &mut self,
        package_version: Option<&str>,
        first_release: Option<&str>,
    ) {
        self.lints = collect_comment_lints_with_context(
            self,
            package_version.and_then(parse_version),
            first_release.and_then(parse_version),
        );
    }

    pub fn doc_blocks(&self) -> Vec<DocBlock> {
        let comments = self
            .comments
            .iter()
            .filter(|comment| comment.kind.is_doc())
            .collect::<Vec<_>>();
        let mut blocks = Vec::new();
        let mut start = 0;
        while start < comments.len() {
            let mut end = start;
            while end + 1 < comments.len()
                && comments[end].kind == comments[end + 1].kind
                && comments[end].line_end + 1 >= comments[end + 1].line_start
            {
                end += 1;
            }
            let text = comments[start..=end]
                .iter()
                .map(|comment| comment.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            blocks.push(DocBlock::parse(
                &text,
                Span {
                    start: comments[start].span.start,
                    end: comments[end].span.end,
                },
            ));
            start = end + 1;
        }
        blocks
    }

    pub fn docs_for_span(&self, span: Span) -> Option<DocBlock> {
        let comments = self
            .comments
            .iter()
            .filter(|comment| comment.kind.is_doc() && comment.span.end <= span.start)
            .collect::<Vec<_>>();
        let item_line = line_at(&self.source, span.start);
        let last = comments
            .iter()
            .rposition(|comment| comment.line_end + 1 >= item_line)?;
        let mut first = last;
        while first > 0
            && comments[first - 1].kind == comments[first].kind
            && comments[first - 1].line_end + 1 >= comments[first].line_start
        {
            first -= 1;
        }
        let text = comments[first..=last]
            .iter()
            .map(|comment| comment.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        Some(DocBlock::parse(
            &text,
            Span {
                start: comments[first].span.start,
                end: comments[last].span.end,
            },
        ))
    }

    /// Resolve `@inheritDoc` for a method implemented by a class.  The
    /// interface method is the source of truth, so the returned block can be
    /// consumed by both diagnostics and the Rust emitter.
    pub fn inherited_docs_for_span(&self, span: Span) -> Option<DocBlock> {
        for item in &self.items {
            let Item::Class(class) = &item.node else {
                continue;
            };
            let Some(method) = class.methods.iter().find(|method| method.span == span) else {
                continue;
            };
            for implemented in &class.implements {
                let interface_name = implemented.split('<').next().unwrap_or(implemented).trim();
                let Some(interface) = self.items.iter().find_map(|item| match &item.node {
                    Item::Interface(interface) if interface.name == interface_name => {
                        Some(interface)
                    }
                    _ => None,
                }) else {
                    continue;
                };
                let Some(parent) = interface
                    .methods
                    .iter()
                    .find(|candidate| candidate.name == method.name)
                else {
                    continue;
                };
                let Some(doc) = self.docs_for_span(parent.span) else {
                    continue;
                };
                if !doc.tags.iter().any(|tag| matches!(tag, DocTag::InheritDoc)) {
                    return Some(doc);
                }
            }
        }
        None
    }

    /// Return the item's own documentation, replacing a resolvable
    /// `@inheritDoc` marker with the interface documentation it names.
    pub fn resolved_docs_for_span(&self, span: Span) -> Option<DocBlock> {
        let own = self.docs_for_span(span)?;
        if own.tags.iter().any(|tag| matches!(tag, DocTag::InheritDoc)) {
            self.inherited_docs_for_span(span).or(Some(own))
        } else {
            Some(own)
        }
    }

    pub fn docs(&self) -> impl Iterator<Item = &Comment> {
        self.comments.iter().filter(|comment| comment.kind.is_doc())
    }

    pub fn leading_trivia(&self, span: Span) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|comment| {
                comment.span.end <= span.start && same_or_adjacent_line(comment, span, &self.source)
            })
            .collect()
    }

    pub fn trailing_trivia(&self, span: Span) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|comment| {
                comment.span.start >= span.end
                    && same_line(span.end, comment.span.start, &self.source)
            })
            .collect()
    }
}

/// Apply the deterministic source formatter used by the CLI. It only removes
/// trailing horizontal whitespace, preserving comments, line endings, and
/// blank-line structure.
pub fn format_source(source: &str) -> String {
    source
        .split_inclusive('\n')
        .map(|line| {
            if let Some(content) = line.strip_suffix('\n') {
                let (body, carriage_return) = content
                    .strip_suffix('\r')
                    .map_or((content, ""), |body| (body, "\r"));
                format!(
                    "{}{}\n",
                    body.trim_end_matches([' ', '\t']),
                    carriage_return
                )
            } else {
                line.trim_end_matches([' ', '\t']).to_owned()
            }
        })
        .collect()
}

impl<T> Spanned<T> {
    pub fn leading_trivia<'a>(&self, program: &'a Program) -> Vec<&'a Comment> {
        program.leading_trivia(self.span)
    }

    pub fn trailing_trivia<'a>(&self, program: &'a Program) -> Vec<&'a Comment> {
        program.trailing_trivia(self.span)
    }

    pub fn docs<'a>(&self, program: &'a Program) -> Vec<&'a Comment> {
        self.leading_trivia(program)
            .into_iter()
            .filter(|comment| comment.kind.is_doc())
            .collect()
    }
}

fn line_at(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn same_line(left: usize, right: usize, source: &str) -> bool {
    line_at(source, left) == line_at(source, right)
}

fn same_or_adjacent_line(comment: &Comment, span: Span, source: &str) -> bool {
    comment.line_end + 1 >= line_at(source, span.start)
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
    let lexed = lex(source)?;
    let mut program = Parser {
        tokens: lexed.tokens,
        pos: 0,
    }
    .program()?;
    program.source = source.to_owned();
    program.bom = lexed.bom;
    program.shebang = lexed.shebang;
    program.comments = lexed.comments;
    program.lints = collect_comment_lints(&program);
    Ok(program)
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

struct Lexed {
    tokens: Vec<Token>,
    comments: Vec<Comment>,
    shebang: Option<Comment>,
    bom: bool,
}

fn lex(source: &str) -> Result<Lexed, ParseError> {
    let bytes = source.as_bytes();
    let bom = source.starts_with('\u{feff}');
    let mut i = if bom { '\u{feff}'.len_utf8() } else { 0 };
    let mut out = Vec::new();
    let comments = scan_comments(source, i)?;
    let shebang = comments
        .iter()
        .find(|comment| comment.kind == CommentKind::Shebang)
        .cloned();
    'lex: while i < bytes.len() {
        if i == if bom { '\u{feff}'.len_utf8() } else { 0 }
            && bytes.get(i) == Some(&b'#')
            && bytes.get(i + 1) == Some(&b'!')
        {
            i = line_end(bytes, i);
            continue;
        }
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
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            i = skip_block_comment(source, i)?;
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
        if c == '|' && bytes.get(i) == Some(&b'|') {
            i += 1;
            out.push(Token {
                kind: TokenKind::Operator("||".into()),
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
    Ok(Lexed {
        tokens: out,
        comments,
        shebang,
        bom,
    })
}

fn skip_block_comment(source: &str, start: usize) -> Result<usize, ParseError> {
    let bytes = source.as_bytes();
    let mut depth = 1usize;
    let mut i = start + 2;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return Ok(i);
            }
        } else {
            i += 1;
        }
    }
    Err(ParseError {
        message: "unterminated block comment".into(),
        span: Span {
            start,
            end: source.len(),
        },
    })
}

fn scan_comments(source: &str, start: usize) -> Result<Vec<Comment>, ParseError> {
    let bytes = source.as_bytes();
    let mut comments = Vec::new();
    let mut i = start;
    let mut quote = None;
    let mut escaped = false;
    while i < bytes.len() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if bytes[i] == b'\\' {
                escaped = true;
            } else if bytes[i] == active_quote {
                quote = None;
            }
            i += 1;
            continue;
        }
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            quote = Some(bytes[i]);
            i += 1;
            continue;
        }
        if i == start && bytes.get(i) == Some(&b'#') && bytes.get(i + 1) == Some(&b'!') {
            let end = line_end(bytes, i);
            comments.push(make_comment(source, CommentKind::Shebang, i, end));
            i = end;
            continue;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'/') {
            let end = line_end(bytes, i);
            let kind = if bytes.get(i + 2) == Some(&b'!') {
                CommentKind::InnerDocLine
            } else if bytes.get(i + 2) == Some(&b'/') && bytes.get(i + 3) != Some(&b'/') {
                CommentKind::OuterDocLine
            } else {
                CommentKind::Line
            };
            comments.push(make_comment(source, kind, i, end));
            i = end;
            continue;
        }
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            let end = skip_block_comment(source, i)?;
            let raw = &source[i..end];
            let kind = if raw == "/**/" || raw == "/***/" {
                CommentKind::Block
            } else if raw.starts_with("/*!") {
                CommentKind::InnerDocBlock
            } else if raw.starts_with("/**") {
                CommentKind::OuterDocBlock
            } else {
                CommentKind::Block
            };
            comments.push(make_comment(source, kind, i, end));
            i = end;
            continue;
        }
        i += 1;
    }
    Ok(comments)
}

fn line_end(bytes: &[u8], start: usize) -> usize {
    bytes[start..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(bytes.len(), |offset| start + offset)
}

fn make_comment(source: &str, kind: CommentKind, start: usize, end: usize) -> Comment {
    Comment {
        kind,
        text: source[start..end].to_owned(),
        span: Span { start, end },
        line_start: line_at(source, start),
        line_end: line_at(source, end),
    }
}

fn documentation_target_spans(item: &Spanned<Item>, spans: &mut Vec<Span>) {
    spans.push(item.span);
    match &item.node {
        Item::Class(value) => {
            spans.extend(value.fields.iter().map(|field| field.span));
            spans.extend(value.methods.iter().map(|method| method.span));
            spans.extend(
                value
                    .constructors
                    .iter()
                    .map(|constructor| constructor.span),
            );
        }
        Item::Interface(value) => spans.extend(value.methods.iter().map(|method| method.span)),
        Item::Enum(value) => spans.extend(value.variants.iter().map(|variant| variant.span)),
        Item::Error(value) => spans.extend(value.cases.iter().map(|case| case.span)),
        Item::Extension(value) => spans.extend(value.methods.iter().map(|method| method.span)),
        Item::Function(_) | Item::TypeAlias(_) | Item::Newtype(_) => {}
    }
}

fn collect_comment_lints(program: &Program) -> Vec<Lint> {
    collect_comment_lints_with_context(program, None, None)
}

fn collect_comment_lints_with_context(
    program: &Program,
    package_version: Option<Vec<u64>>,
    first_release: Option<Vec<u64>>,
) -> Vec<Lint> {
    let mut target_spans = Vec::new();
    for item in &program.items {
        documentation_target_spans(item, &mut target_spans);
    }
    let mut lints = Vec::new();
    if !program.source.contains("orust:allow(OR0616)") {
        let marker = "orust:allow(";
        for (start, _) in program.source.match_indices(marker) {
            let rest = &program.source[start + marker.len()..];
            let Some(end) = rest.find(')') else {
                continue;
            };
            let code = &rest[..end];
            if code.starts_with("OR") && !is_known_doc_lint(code) {
                lints.push(Lint {
                    code: "OR0616",
                    message: format!("unknown suppression code {code}"),
                    severity: LintSeverity::Warning,
                    span: Span {
                        start,
                        end: start + marker.len() + end + 1,
                    },
                });
            }
        }
    }
    lints.extend(
        program
            .comments
            .iter()
            .filter(|comment| comment.kind.is_outer_doc())
            .filter(|comment| {
                !program.comments.iter().any(|next| {
                    next.kind == comment.kind
                        && next.span.start > comment.span.end
                        && next.line_start <= comment.line_end + 1
                })
            })
            .filter(|comment| {
                !target_spans.iter().any(|span| {
                    span.start > comment.span.end
                        && line_at(&program.source, span.start) <= comment.line_end + 1
                })
            })
            .map(|comment| Lint {
                code: "OR0601",
                message: "outer documentation comment has no declaration target".into(),
                severity: LintSeverity::Warning,
                span: comment.span,
            })
            .collect::<Vec<_>>(),
    );
    let unknown_suppressed = program.source.contains("orust:allow(OR0603)");
    if !unknown_suppressed {
        for comment in &program.comments {
            let Some(doc) = comment.doc_block() else {
                continue;
            };
            for tag in doc.tags {
                if let DocTag::Unknown { name, .. } = tag {
                    let suggestion = suggest_doc_tag(&name)
                        .map(|value| format!(", did you mean @{value}?"))
                        .unwrap_or_default();
                    lints.push(Lint {
                        code: "OR0603",
                        message: format!("unknown doc tag @{name}{suggestion}"),
                        severity: LintSeverity::Warning,
                        span: comment.span,
                    });
                }
            }
        }
    }
    if program.source.contains("orust:allow(OR0601)") {
        lints.retain(|lint| lint.code != "OR0601");
    }
    for item in &program.items {
        let doc = program.docs_for_span(item.span);
        if item_is_exported(&item.node)
            && doc.is_none()
            && !program.source.contains("orust:allow(OR0606)")
        {
            lints.push(Lint {
                code: "OR0606",
                message: "exported item is missing documentation".into(),
                severity: LintSeverity::Warning,
                span: item.span,
            });
        }
        let Some(doc) = doc else {
            continue;
        };
        let suppressed = |code: &str| program.source.contains(&format!("orust:allow({code})"));
        let Item::Function(function) = &item.node else {
            continue;
        };
        let params = doc
            .tags
            .iter()
            .filter_map(|tag| match tag {
                DocTag::Param { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !suppressed("OR0607")
            && (!params.is_empty()
                && (params
                    .iter()
                    .any(|name| !function.params.iter().any(|param| &param.name == name))
                    || function
                        .params
                        .iter()
                        .any(|param| !params.contains(&param.name.as_str()))))
        {
            lints.push(Lint {
                code: "OR0607",
                message: "documentation parameters do not match the function parameters".into(),
                severity: LintSeverity::Warning,
                span: doc.span,
            });
        }
        let throws = doc
            .tags
            .iter()
            .filter_map(|tag| match tag {
                DocTag::Throws { ty, .. } => Some(ty.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !suppressed("OR0608")
            && ((function.throws_type.is_some() && throws.is_empty())
                || throws
                    .iter()
                    .any(|ty| Some(*ty) != function.throws_type.as_deref()))
        {
            lints.push(Lint {
                code: "OR0608",
                message: "documentation errors do not match the function throws type".into(),
                severity: LintSeverity::Warning,
                span: doc.span,
            });
        }
        if !suppressed("OR0609")
            && matches!(function.return_type.as_str(), "void" | "()")
            && doc.tags.iter().any(|tag| matches!(tag, DocTag::Returns(_)))
        {
            lints.push(Lint {
                code: "OR0609",
                message: "@returns documents a function with no return value".into(),
                severity: LintSeverity::Warning,
                span: doc.span,
            });
        }
    }
    let item_names = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Class(value) => Some(value.name.as_str()),
            Item::Function(value) => Some(value.name.as_str()),
            Item::Interface(value) => Some(value.name.as_str()),
            Item::Enum(value) => Some(value.name.as_str()),
            Item::Error(value) => Some(value.name.as_str()),
            Item::TypeAlias(value) => Some(value.name.as_str()),
            Item::Newtype(value) => Some(value.name.as_str()),
            Item::Extension(_) => None,
        })
        .collect::<Vec<_>>();
    for doc in program.doc_blocks() {
        let suppressed = |code: &str| program.source.contains(&format!("orust:allow({code})"));
        let count =
            |predicate: fn(&DocTag) -> bool| doc.tags.iter().filter(|tag| predicate(tag)).count();
        let has_resolved_inheritance = target_spans.iter().any(|target| {
            program
                .docs_for_span(*target)
                .is_some_and(|candidate| candidate.span == doc.span)
                && program.inherited_docs_for_span(*target).is_some()
        });
        if !suppressed("OR0602")
            && doc.tags.iter().any(|tag| matches!(tag, DocTag::InheritDoc))
            && !has_resolved_inheritance
        {
            lints.push(Lint {
                code: "OR0602",
                message: "@inheritDoc has no resolvable parent documentation".into(),
                severity: LintSeverity::Warning,
                span: doc.span,
            });
        }
        let links = doc.tags.iter().filter_map(|tag| match tag {
            DocTag::See(value) => Some(value.as_str()),
            _ => None,
        });
        if !suppressed("OR0604")
            && links.clone().any(|link| {
                link.trim_matches(&['[', ']', '`'][..])
                    .split_once('.')
                    .is_some_and(|(name, _)| !item_names.contains(&name))
            })
        {
            lints.push(Lint {
                code: "OR0604",
                message: "documentation link does not resolve to an ORust item".into(),
                severity: LintSeverity::Warning,
                span: doc.span,
            });
        }
        if !suppressed("OR0605")
            && doc
                .tags
                .iter()
                .any(|tag| matches!(tag, DocTag::Example { code, .. } if code.contains("std:http")))
        {
            lints.push(Lint {
                code: "OR0605",
                message: "doctest uses a network feature without `no_run` or `ignore`".into(),
                severity: LintSeverity::Warning,
                span: doc.span,
            });
        }
        if !suppressed("OR0612")
            && [
                count(|tag| matches!(tag, DocTag::Since(_))) > 1,
                count(|tag| matches!(tag, DocTag::Version(_))) > 1,
                count(|tag| matches!(tag, DocTag::Returns(_))) > 1,
            ]
            .into_iter()
            .any(|value| value)
        {
            lints.push(Lint {
                code: "OR0612",
                message: "documentation tag may appear only once".into(),
                severity: LintSeverity::Warning,
                span: doc.span,
            });
        }
        if !suppressed("OR0613") && !doc.tags.is_empty() && doc.summary.is_empty() {
            lints.push(Lint {
                code: "OR0613",
                message: "documentation is empty after removing tags".into(),
                severity: LintSeverity::Warning,
                span: doc.span,
            });
        }
        if !suppressed("OR0610") {
            for tag in &doc.tags {
                let DocTag::Since(value) = tag else {
                    continue;
                };
                let Some(since) = parse_version(value) else {
                    continue;
                };
                let too_new = package_version
                    .as_ref()
                    .is_some_and(|package| since > *package);
                let too_old = first_release
                    .as_ref()
                    .is_some_and(|release| since < *release);
                if too_new || too_old {
                    lints.push(Lint {
                        code: "OR0610",
                        message: format!("@since {value} is outside the package release range"),
                        severity: LintSeverity::Warning,
                        span: doc.span,
                    });
                }
            }
        }
        if !suppressed("OR0614")
            && (program.source.contains("orust:deny(OR0614)")
                || program.source.contains("orust:warn(OR0614)"))
            && (doc.summary.chars().count() > 120
                || (!doc.summary.is_empty() && !doc.summary.trim_end().ends_with(['.', '!', '?'])))
        {
            lints.push(Lint {
                code: "OR0614",
                message: "summary is too long or does not end as a sentence".into(),
                severity: LintSeverity::Warning,
                span: doc.span,
            });
        }
    }
    if !program.source.contains("orust:allow(OR0611)") {
        for (name, span) in deprecated_item_names(program) {
            if source_uses_deprecated_name(program, &name) {
                lints.push(Lint {
                    code: "OR0611",
                    message: format!("`{name}` is deprecated and is still used in this package"),
                    severity: LintSeverity::Warning,
                    span,
                });
            }
        }
    }
    if !program.source.contains("orust:allow(OR0615)") {
        for comment in &program.comments {
            if comment.kind == CommentKind::Line
                && ["TODO", "FIXME", "HACK", "XXX"]
                    .iter()
                    .any(|marker| comment.text.contains(marker))
            {
                lints.push(Lint {
                    code: "OR0615",
                    message: "work marker comment is present".into(),
                    severity: LintSeverity::Warning,
                    span: comment.span,
                });
            }
        }
    }
    lints.sort_by_key(|lint| (lint.span.start, lint.code));
    lints
}

fn parse_version(value: &str) -> Option<Vec<u64>> {
    let value = value.trim().trim_start_matches('v');
    let value = value.split_once('-').map_or(value, |(base, _)| base);
    let parts = value.split('.').collect::<Vec<_>>();
    (!parts.is_empty() && parts.iter().all(|part| part.parse::<u64>().is_ok())).then(|| {
        parts
            .into_iter()
            .map(|part| part.parse().unwrap())
            .collect()
    })
}

fn suggest_doc_tag(name: &str) -> Option<&'static str> {
    const TAGS: &[&str] = &[
        "author",
        "param",
        "typeParam",
        "returns",
        "throws",
        "panics",
        "example",
        "see",
        "since",
        "deprecated",
        "version",
        "todo",
        "note",
        "warning",
        "internal",
        "category",
        "inheritDoc",
        "license",
        "copyright",
    ];
    TAGS.iter()
        .copied()
        .map(|candidate| (edit_distance(name, candidate), candidate))
        .filter(|(distance, _)| *distance <= 3)
        .min_by_key(|(distance, candidate)| (*distance, *candidate))
        .map(|(_, candidate)| candidate)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut row = (0..=right.chars().count()).collect::<Vec<_>>();
    for (left_index, left_char) in left.chars().enumerate() {
        let mut next = vec![left_index + 1];
        for (right_index, right_char) in right.chars().enumerate() {
            let substitution = row[right_index] + usize::from(left_char != right_char);
            next.push(
                (substitution)
                    .min(row[right_index + 1] + 1)
                    .min(next[right_index] + 1),
            );
        }
        row = next;
    }
    *row.last().unwrap_or(&0)
}

fn deprecated_item_names(program: &Program) -> Vec<(String, Span)> {
    let mut result = Vec::new();
    for item in &program.items {
        if program.docs_for_span(item.span).is_some_and(|doc| {
            doc.tags
                .iter()
                .any(|tag| matches!(tag, DocTag::Deprecated { .. }))
        }) {
            if let Some(name) = item_name(&item.node) {
                result.push((name.to_owned(), item.span));
            }
        }
        if let Item::Class(class) = &item.node {
            for method in &class.methods {
                if program.docs_for_span(method.span).is_some_and(|doc| {
                    doc.tags
                        .iter()
                        .any(|tag| matches!(tag, DocTag::Deprecated { .. }))
                }) {
                    result.push((method.name.clone(), method.span));
                }
            }
        }
    }
    result
}

fn item_name(item: &Item) -> Option<&str> {
    match item {
        Item::Class(value) => Some(&value.name),
        Item::Function(value) => Some(&value.name),
        Item::Interface(value) => Some(&value.name),
        Item::Enum(value) => Some(&value.name),
        Item::Error(value) => Some(&value.name),
        Item::TypeAlias(value) => Some(&value.name),
        Item::Newtype(value) => Some(&value.name),
        Item::Extension(_) => None,
    }
}

fn source_uses_deprecated_name(program: &Program, name: &str) -> bool {
    let needle = format!("{name}(");
    program.source.lines().any(|line| {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            return false;
        }
        line.contains(&needle)
            && !trimmed.starts_with(&format!("void {name}"))
            && !trimmed.starts_with(&format!("async void {name}"))
    })
}

fn is_known_doc_lint(code: &str) -> bool {
    matches!(
        code,
        "OR0601"
            | "OR0602"
            | "OR0603"
            | "OR0604"
            | "OR0605"
            | "OR0606"
            | "OR0607"
            | "OR0608"
            | "OR0609"
            | "OR0610"
            | "OR0611"
            | "OR0612"
            | "OR0613"
            | "OR0614"
            | "OR0615"
            | "OR0616"
    )
}

fn item_is_exported(item: &Item) -> bool {
    match item {
        Item::Class(value) => value.exported,
        Item::Function(value) => value.exported,
        Item::Interface(value) => value.exported,
        Item::Enum(value) => value.exported,
        Item::Error(value) => value.exported,
        Item::Extension(value) => value.exported,
        Item::TypeAlias(value) => value.exported,
        Item::Newtype(value) => value.exported,
    }
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
            if matches!(kind, TokenKind::Symbol(';'))
                && self.is(&TokenKind::Symbol('}'))
                && self.pos > 0
            {
                let previous_end = self.tokens[self.pos - 1].span.end;
                return Err(ParseError {
                    message: "expected Symbol(';') before closing `}`; add `;` to finish the previous statement".into(),
                    span: Span {
                        start: previous_end,
                        end: previous_end,
                    },
                });
            }
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
            source: String::new(),
            bom: false,
            shebang: None,
            comments: Vec::new(),
            lints: Vec::new(),
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
                let declaration_start = self.current().span.start;
                let initializer = if self.is(&TokenKind::Operator("=".into())) {
                    self.bump();
                    Some(self.expr()?)
                } else {
                    None
                };
                let declaration_end = self.expect(TokenKind::Symbol(';'))?.span.end;
                fields.push(Field {
                    ty: return_type,
                    name: member,
                    initializer,
                    span: Span {
                        start: declaration_start,
                        end: declaration_end,
                    },
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
    use proptest::prelude::*;

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
        assert!(error.message.contains("before closing"));
        assert_eq!(error.span.start, "void main() { print(1)".len());
        assert_eq!(error.span.end, error.span.start);
    }

    #[test]
    fn comments_preserve_error_line_mapping() {
        let plain = "void main() { print(1) }\n";
        let with_prefix = "// explanation\nvoid main() { print(1) }\n";
        let with_inline = "void main() { print(1 /* explanation */) }\n";
        let line = |source: &str, offset: usize| {
            source[..offset]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1
        };
        let plain_error = parse(plain).unwrap_err();
        let prefix_error = parse(with_prefix).unwrap_err();
        let inline_error = parse(with_inline).unwrap_err();
        assert_eq!(line(plain, plain_error.span.start), 1);
        assert_eq!(line(with_prefix, prefix_error.span.start), 2);
        assert_eq!(line(with_inline, inline_error.span.start), 1);
    }

    #[test]
    fn one_hundred_inserted_comment_lines_shift_errors_exactly() {
        let source = (0..100)
            .map(|index| format!("// generated-{index}\n"))
            .collect::<String>()
            + "void main() {\n  print(1)\n}\n";
        let error = parse(&source).unwrap_err();
        let line = source[..error.span.start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        assert_eq!(line, 102);
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

    #[test]
    fn preserves_comment_kinds_and_exact_source() {
        let source = "\u{feff}#!/usr/bin/env orust\r\n/// outer\r\nclass User {} // trailing\r\n/** block */\r\n//! inner\r\n/*! inner block */\r\n//// plain\r\n/**/ /***/\r\n";
        let program = parse(source).unwrap();
        assert_eq!(program.source, source);
        assert!(program.bom);
        assert_eq!(
            program.shebang.as_ref().unwrap().text,
            "#!/usr/bin/env orust\r"
        );
        assert_eq!(program.comments.len(), 9);
        assert_eq!(
            program
                .comments
                .iter()
                .map(|comment| comment.kind)
                .collect::<Vec<_>>(),
            vec![
                CommentKind::Shebang,
                CommentKind::OuterDocLine,
                CommentKind::Line,
                CommentKind::OuterDocBlock,
                CommentKind::InnerDocLine,
                CommentKind::InnerDocBlock,
                CommentKind::Line,
                CommentKind::Block,
                CommentKind::Block,
            ]
        );
        assert_eq!(program.docs().count(), 4);
        assert_eq!(program.lints.len(), 1);
        assert_eq!(program.lints[0].code, "OR0601");
    }

    #[test]
    fn supports_nested_block_comments_and_non_ascii_text() {
        let program = parse("/* наружный /* вложенный */ комментарий */ class User {}").unwrap();
        assert_eq!(program.comments.len(), 1);
        assert_eq!(program.comments[0].kind, CommentKind::Block);
        assert!(program.comments[0].text.contains("наружный"));
        assert!(parse("class User {} /* unterminated /* nested */").is_err());
    }

    #[test]
    fn attaches_adjacent_comments_as_trivia_and_warns_for_detached_docs() {
        let program = parse("/// User docs\nclass User {} // after\n\n/// orphan\n\n").unwrap();
        let item = &program.items[0];
        assert_eq!(item.docs(&program).len(), 1);
        assert_eq!(item.trailing_trivia(&program).len(), 1);
        assert_eq!(program.lints.len(), 1);
        assert_eq!(program.lints[0].code, "OR0601");
    }

    #[test]
    fn comments_inside_expressions_do_not_change_parsing() {
        let plain = parse("void main() { print(1); }").unwrap();
        let commented = parse("void main() { print(/* first */ 1 /* second */); }").unwrap();
        assert!(matches!(plain.items[0].node, Item::Function(_)));
        assert!(matches!(commented.items[0].node, Item::Function(_)));
        assert_eq!(commented.comments.len(), 2);
    }

    #[test]
    fn parses_doc_tags_without_losing_unknown_text() {
        let doc = Comment {
            kind: CommentKind::OuterDocLine,
            text: "/// Summary\n///\n/// Details.\n/// @author Ada Lovelace <ada@example.com> (https://ada.dev)\n/// @param value the value\n/// @returns the result\n/// @throws Error when it fails\n/// @deprecated since 0.3.0 Use the new API.\n/// @unknown preserved text".into(),
            span: Span { start: 0, end: 300 },
            line_start: 1,
            line_end: 9,
        };
        let parsed = doc.doc_block().unwrap();
        assert_eq!(parsed.raw, doc.text);
        assert_eq!(parsed.summary, "Summary");
        assert!(
            matches!(parsed.tags[0], DocTag::Author { ref name, ref email, ref url } if name == "Ada Lovelace" && email.as_deref() == Some("ada@example.com") && url.as_deref() == Some("https://ada.dev"))
        );
        assert!(matches!(parsed.tags[1], DocTag::Param { ref name, .. } if name == "value"));
        assert!(matches!(parsed.tags[2], DocTag::Returns(ref value) if value == "the result"));
        assert!(matches!(parsed.tags[3], DocTag::Throws { ref ty, .. } if ty == "Error"));
        assert!(
            matches!(parsed.tags[4], DocTag::Deprecated { ref since, .. } if since.as_deref() == Some("0.3.0"))
        );
        assert!(
            matches!(parsed.tags[5], DocTag::Unknown { ref name, ref text } if name == "unknown" && text == "preserved text")
        );
    }

    #[test]
    fn covers_every_documentation_tag_row() {
        let source = concat!(
            "/// Summary.\n",
            "/// @author Ada <ada@example.com> (https://ada.dev)\n",
            "/// @param value the value\n",
            "/// @typeParam T the type\n",
            "/// @returns the result\n",
            "/// @throws Error when invalid\n",
            "/// @panics when impossible\n",
            "/// @example demo\n",
            "/// ```orust\n",
            "/// print(1);\n",
            "/// ```\n",
            "/// @see Other\n",
            "/// @since 1.0\n",
            "/// @deprecated 2.0 use Other\n",
            "/// @version 1.2\n",
            "/// @todo finish\n",
            "/// @note note\n",
            "/// @warning warning\n",
            "/// @internal\n",
            "/// @category api\n",
            "/// @inheritDoc\n",
            "/// @license MIT\n",
            "/// @copyright 2026\n",
            "/// @future preserved\n",
        );
        let parsed = DocBlock::parse(
            source,
            Span {
                start: 0,
                end: source.len(),
            },
        );
        assert_eq!(parsed.tags.len(), 20);
        assert!(matches!(parsed.tags[0], DocTag::Author { .. }));
        assert!(matches!(parsed.tags[1], DocTag::Param { .. }));
        assert!(matches!(parsed.tags[2], DocTag::TypeParam { .. }));
        assert!(matches!(parsed.tags[3], DocTag::Returns(_)));
        assert!(matches!(parsed.tags[4], DocTag::Throws { .. }));
        assert!(matches!(parsed.tags[5], DocTag::Panics(_)));
        assert!(matches!(parsed.tags[6], DocTag::Example { .. }));
        assert!(matches!(parsed.tags[7], DocTag::See(_)));
        assert!(matches!(parsed.tags[8], DocTag::Since(_)));
        assert!(matches!(parsed.tags[9], DocTag::Deprecated { .. }));
        assert!(matches!(parsed.tags[10], DocTag::Version(_)));
        assert!(matches!(parsed.tags[11], DocTag::Todo(_)));
        assert!(matches!(parsed.tags[12], DocTag::Note(_)));
        assert!(matches!(parsed.tags[13], DocTag::Warning(_)));
        assert!(matches!(parsed.tags[14], DocTag::Internal));
        assert!(matches!(parsed.tags[15], DocTag::Category(_)));
        assert!(matches!(parsed.tags[16], DocTag::InheritDoc));
        assert!(matches!(parsed.tags[17], DocTag::License(_)));
        assert!(matches!(parsed.tags[18], DocTag::Copyright(_)));
        assert!(matches!(parsed.tags[19], DocTag::Unknown { .. }));
        assert_eq!(parsed.raw, source);
    }

    #[test]
    fn doc_parser_handles_fences_and_arbitrary_delimiters() {
        let parsed = DocBlock::parse(
            "/// text with @not-a-tag\n/// ```orust\n/// @inside-code\n/// ```\n/// @example demo\n/// ```orust\n/// print(1);\n/// ```",
            Span { start: 4, end: 80 },
        );
        assert_eq!(parsed.tags.len(), 1);
        assert!(
            matches!(parsed.tags[0], DocTag::Example { ref title, .. } if title.as_deref() == Some("demo"))
        );
        for value in ["", "@", "@ ", "```", "/*", "😀 @unknown text", "\n\n\n"] {
            let _ = DocBlock::parse(
                value,
                Span {
                    start: 0,
                    end: value.len(),
                },
            );
        }
    }

    #[test]
    fn lossless_source_survives_deterministic_comment_insertions() {
        let base = "void main() { print(1); }\n";
        for index in 0..100 {
            let source = format!("// generated comment {index}\n{base}");
            let program = parse(&source).unwrap();
            assert_eq!(program.lossless_source(), source);
            assert_eq!(program.items.len(), 1);
        }
    }

    #[test]
    fn unknown_doc_tags_have_clean_and_suppressed_cases() {
        let failing = parse("/// @authro Ada\nclass User {}").unwrap();
        assert!(failing
            .lints
            .iter()
            .any(|lint| lint.code == "OR0603" && lint.message.contains("did you mean @author")));
        let clean = parse("/// @author Ada\nclass User {}").unwrap();
        assert!(!clean.lints.iter().any(|lint| lint.code == "OR0603"));
        let suppressed = parse("// orust:allow(OR0603)\n/// @authro Ada\nclass User {}").unwrap();
        assert!(!suppressed.lints.iter().any(|lint| lint.code == "OR0603"));
    }

    #[test]
    fn orphan_doc_lint_has_a_suppression_case() {
        let failing = parse("/// orphan\n\n").unwrap();
        assert!(failing.lints.iter().any(|lint| lint.code == "OR0601"));
        let clean = parse("/// attached\nclass User {}").unwrap();
        assert!(!clean.lints.iter().any(|lint| lint.code == "OR0601"));
        let suppressed = parse("// orust:allow(OR0601)\n/// orphan\n\n").unwrap();
        assert!(!suppressed.lints.iter().any(|lint| lint.code == "OR0601"));
    }

    #[test]
    fn groups_line_doc_comments_into_one_example_block() {
        let program = parse(
            "/// Demo\n/// @example smoke\n/// ```orust\n/// print(1);\n/// ```\nvoid main() {}",
        )
        .unwrap();
        let blocks = program.doc_blocks();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(
            &blocks[0].tags[0],
            DocTag::Example { code, .. } if code == "print(1);"
        ));
    }

    #[test]
    fn signature_doc_lints_have_failing_clean_and_suppressed_cases() {
        let mismatch = parse("/// docs\n/// @param missing value\nvoid run(int value) {}").unwrap();
        assert!(mismatch.lints.iter().any(|lint| lint.code == "OR0607"));
        let clean = parse("/// docs\n/// @param value input\nvoid run(int value) {}").unwrap();
        assert!(!clean.lints.iter().any(|lint| lint.code == "OR0607"));
        let suppressed = parse(
            "// orust:allow(OR0607)\n/// docs\n/// @param missing value\nvoid run(int value) {}",
        )
        .unwrap();
        assert!(!suppressed.lints.iter().any(|lint| lint.code == "OR0607"));

        let throws_mismatch = parse(
            "/// docs\n/// @throws Other fails\nString run() throws Error { return \"ok\"; }",
        )
        .unwrap();
        assert!(throws_mismatch
            .lints
            .iter()
            .any(|lint| lint.code == "OR0608"));
        let throws_clean = parse(
            "/// docs\n/// @throws Error fails\nString run() throws Error { return \"ok\"; }",
        )
        .unwrap();
        assert!(!throws_clean.lints.iter().any(|lint| lint.code == "OR0608"));
        let throws_suppressed =
            parse("// orust:allow(OR0608)\n/// docs\nString run() throws Error { return \"ok\"; }")
                .unwrap();
        assert!(!throws_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0608"));

        let returns_mismatch = parse("/// docs\n/// @returns value\nvoid run() {}").unwrap();
        assert!(returns_mismatch
            .lints
            .iter()
            .any(|lint| lint.code == "OR0609"));
        let returns_clean = parse("/// docs\nvoid run() {}").unwrap();
        assert!(!returns_clean.lints.iter().any(|lint| lint.code == "OR0609"));
        let returns_suppressed =
            parse("// orust:allow(OR0609)\n/// docs\n/// @returns value\nvoid run() {}").unwrap();
        assert!(!returns_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0609"));
    }

    #[test]
    fn remaining_documentation_lints_have_failing_clean_and_suppressed_cases() {
        let unknown_suppression = parse("// orust:allow(OR0999)\nclass User {}").unwrap();
        assert!(unknown_suppression
            .lints
            .iter()
            .any(|lint| lint.code == "OR0616"));
        let known_suppression = parse("// orust:allow(OR0601)\nclass User {}").unwrap();
        assert!(!known_suppression
            .lints
            .iter()
            .any(|lint| lint.code == "OR0616"));
        let suppression_suppressed =
            parse("// orust:allow(OR0616)\n// orust:allow(OR0999)\nclass User {}").unwrap();
        assert!(!suppression_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0616"));

        let missing = parse("export void public_api() {}").unwrap();
        assert!(missing.lints.iter().any(|lint| lint.code == "OR0606"));
        let documented = parse("/// Public API.\nexport void public_api() {}").unwrap();
        assert!(!documented.lints.iter().any(|lint| lint.code == "OR0606"));
        let missing_suppressed =
            parse("// orust:allow(OR0606)\nexport void public_api() {}").unwrap();
        assert!(!missing_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0606"));

        let inherit = parse("/// @inheritDoc\nclass User {}").unwrap();
        assert!(inherit.lints.iter().any(|lint| lint.code == "OR0602"));
        let inherit_clean = parse("/// User docs.\nclass User {}").unwrap();
        assert!(!inherit_clean.lints.iter().any(|lint| lint.code == "OR0602"));
        let inherit_suppressed =
            parse("// orust:allow(OR0602)\n/// @inheritDoc\nclass User {}").unwrap();
        assert!(!inherit_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0602"));

        let broken = parse("/// Link.\n/// @see [Missing.method]\nclass User {}").unwrap();
        assert!(broken.lints.iter().any(|lint| lint.code == "OR0604"));
        let link_clean = parse("/// Link.\n/// @see [User]\nclass User {}").unwrap();
        assert!(!link_clean.lints.iter().any(|lint| lint.code == "OR0604"));
        let link_suppressed =
            parse("// orust:allow(OR0604)\n/// Link.\n/// @see [Missing.method]\nclass User {}")
                .unwrap();
        assert!(!link_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0604"));

        let network = parse(
            "/// Net.\n/// @example net\n/// ```orust\n/// std:http\n/// ```\nvoid main() {}",
        )
        .unwrap();
        assert!(network.lints.iter().any(|lint| lint.code == "OR0605"));
        let network_clean = parse(
            "/// Net.\n/// @example net\n/// ```orust\n/// print(1);\n/// ```\nvoid main() {}",
        )
        .unwrap();
        assert!(!network_clean.lints.iter().any(|lint| lint.code == "OR0605"));
        let network_suppressed = parse(
            "// orust:allow(OR0605)\n/// Net.\n/// @example net\n/// ```orust\n/// std:http\n/// ```\nvoid main() {}",
        )
        .unwrap();
        assert!(!network_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0605"));

        let duplicate = parse("/// Docs.\n/// @since 1.0\n/// @since 2.0\nclass User {}").unwrap();
        assert!(duplicate.lints.iter().any(|lint| lint.code == "OR0612"));
        let duplicate_clean = parse("/// Docs.\n/// @since 1.0\nclass User {}").unwrap();
        assert!(!duplicate_clean
            .lints
            .iter()
            .any(|lint| lint.code == "OR0612"));
        let duplicate_suppressed = parse(
            "// orust:allow(OR0612)\n/// Docs.\n/// @since 1.0\n/// @since 2.0\nclass User {}",
        )
        .unwrap();
        assert!(!duplicate_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0612"));

        let empty = parse("/// @since 1.0\nclass User {}").unwrap();
        assert!(empty.lints.iter().any(|lint| lint.code == "OR0613"));
        let empty_clean = parse("/// User docs.\nclass User {}").unwrap();
        assert!(!empty_clean.lints.iter().any(|lint| lint.code == "OR0613"));
        let empty_suppressed =
            parse("// orust:allow(OR0613)\n/// @since 1.0\nclass User {}").unwrap();
        assert!(!empty_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0613"));

        let summary = parse("/// summary without punctuation\nclass User {}").unwrap();
        assert!(!summary.lints.iter().any(|lint| lint.code == "OR0614"));
        let summary_clean = parse("/// Summary.\nclass User {}").unwrap();
        assert!(!summary_clean.lints.iter().any(|lint| lint.code == "OR0614"));
        let summary_suppressed =
            parse("// orust:allow(OR0614)\n/// summary without punctuation\nclass User {}")
                .unwrap();
        assert!(!summary_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0614"));
        let summary_enabled =
            parse("// orust:deny(OR0614)\n/// summary without punctuation\nclass User {}").unwrap();
        assert!(summary_enabled
            .lints
            .iter()
            .any(|lint| lint.code == "OR0614"));

        let marker = parse("// TODO: finish this\nclass User {}").unwrap();
        assert!(marker.lints.iter().any(|lint| lint.code == "OR0615"));
        let marker_clean = parse("// note: finish this\nclass User {}").unwrap();
        assert!(!marker_clean.lints.iter().any(|lint| lint.code == "OR0615"));
        let marker_suppressed =
            parse("// orust:allow(OR0615)\n// TODO: finish this\nclass User {}").unwrap();
        assert!(!marker_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0615"));

        let mut since_new = parse("/// Docs.\n/// @since 2.0.0\nclass User {}").unwrap();
        since_new.lint_documentation(Some("1.0.0"), None);
        assert!(since_new.lints.iter().any(|lint| lint.code == "OR0610"));
        let mut since_clean = parse("/// Docs.\n/// @since 1.0.0\nclass User {}").unwrap();
        since_clean.lint_documentation(Some("1.0.0"), None);
        assert!(!since_clean.lints.iter().any(|lint| lint.code == "OR0610"));
        let mut since_suppressed =
            parse("// orust:allow(OR0610)\n/// Docs.\n/// @since 2.0.0\nclass User {}").unwrap();
        since_suppressed.lint_documentation(Some("1.0.0"), None);
        assert!(!since_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0610"));

        let mut deprecated_use = parse(
            "/// Old API.\n/// @deprecated since 0.3.0 use new_api\nvoid old_api() {}\nvoid run() { old_api(); }",
        )
        .unwrap();
        deprecated_use.lint_documentation(Some("1.0.0"), None);
        assert!(deprecated_use
            .lints
            .iter()
            .any(|lint| lint.code == "OR0611"));
        let mut deprecated_clean = parse(
            "/// Old API.\n/// @deprecated since 0.3.0 use new_api\nvoid old_api() {}\nvoid run() { new_api(); }",
        )
        .unwrap();
        deprecated_clean.lint_documentation(Some("1.0.0"), None);
        assert!(!deprecated_clean
            .lints
            .iter()
            .any(|lint| lint.code == "OR0611"));
        let mut deprecated_suppressed = parse(
            "// orust:allow(OR0611)\n/// Old API.\n/// @deprecated since 0.3.0 use new_api\nvoid old_api() {}\nvoid run() { old_api(); }",
        )
        .unwrap();
        deprecated_suppressed.lint_documentation(Some("1.0.0"), None);
        assert!(!deprecated_suppressed
            .lints
            .iter()
            .any(|lint| lint.code == "OR0611"));
    }

    #[test]
    fn inherit_doc_resolves_interface_method_documentation() {
        let program = parse(
            "interface Greeter { /// Greets a person.\n /// @param name the person\n String greet(String name); } class Friendly implements Greeter { /// @inheritDoc\n String greet(String name) { return name; } }",
        )
        .unwrap();
        let class_method = match &program.items[1].node {
            Item::Class(class) => class.methods[0].span,
            other => panic!("expected class, got {other:?}"),
        };
        let resolved = program.resolved_docs_for_span(class_method).unwrap();
        assert_eq!(resolved.body, "Greets a person.");
        assert!(resolved
            .tags
            .iter()
            .any(|tag| matches!(tag, DocTag::Param { name, .. } if name == "name")));
        assert!(!program.lints.iter().any(|lint| lint.code == "OR0602"));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            failure_persistence: None,
            .. ProptestConfig::default()
        })]

        #[test]
        fn proptest_cst_roundtrip_with_random_comment_insertions(
            offset in 0usize..=26,
            case in 0u32..10_000,
        ) {
            let base = "void main() { print(1); }\n";
            let left = offset.checked_sub(1).and_then(|index| base.as_bytes().get(index));
            let right = base.as_bytes().get(offset);
            prop_assume!(
                !matches!(left, Some(value) if value.is_ascii_alphanumeric() || *value == b'_')
                    || !matches!(right, Some(value) if value.is_ascii_alphanumeric() || *value == b'_')
            );
            let comment = format!("/* generated-{case} */");
            let source = format!("{}{}{}", &base[..offset], comment, &base[offset..]);
            let program = parse(&source).unwrap();
            prop_assert_eq!(program.lossless_source(), source);
            prop_assert!(program.comments.iter().any(|value| value.text.contains(&case.to_string())));
        }

        #[test]
        fn proptest_formatter_is_idempotent_with_comments(
            value in 0u8..=9,
            prefix in proptest::string::string_regex("[A-Za-z0-9 ]{0,24}").unwrap(),
        ) {
            let source = format!(
                "// {prefix}  \r\nvoid main() {{ /* left */ print({value}); // right  \r\n}}\r\n"
            );
            let first = format_source(&source);
            let second = format_source(&first);
            prop_assert_eq!(&second, &first);
            prop_assert!(first.contains("/* left */"));
            prop_assert!(first.contains("// right"));
        }

        #[test]
        fn proptest_doc_parser_never_panics_or_loses_text(
            payload in any::<String>(),
            case in 0u16..10_000,
        ) {
            let arbitrary = format!("/// arbitrary @tag{case} {payload}");
            let block = DocBlock::parse(
                &arbitrary,
                Span { start: 0, end: arbitrary.len() },
            );
            prop_assert_eq!(&block.raw, &arbitrary);
            let tag = format!("@tag{}", case);
            prop_assert!(block.raw.contains(&tag));
        }
    }
}
