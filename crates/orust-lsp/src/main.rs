#![allow(deprecated)]

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use orust_syntax::{parse, Class, Constructor, Function, Import, Item, Program, Span};
use tokio::time::sleep;
use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CompletionItem,
        CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        DocumentFormattingParams, DocumentSymbolParams, DocumentSymbolResponse, Documentation,
        GotoDefinitionParams, GotoDefinitionResponse, Hover, HoverContents, HoverParams,
        InitializeParams, InitializeResult, InlayHint, InlayHintLabel, InlayHintParams, Location,
        MarkupContent, MarkupKind, MessageType, OneOf, Position, ReferenceParams, RenameParams,
        ServerCapabilities, ServerInfo, SignatureHelp, SignatureHelpOptions, SignatureHelpParams,
        SignatureInformation, SymbolInformation, SymbolKind, TextDocumentContentChangeEvent,
        TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit, Url, WorkspaceEdit,
        WorkspaceSymbolParams,
    },
    Client, LanguageServer, LspService, Server,
};

#[derive(Clone, Debug)]
struct Symbol {
    name: String,
    kind: SymbolKind,
    span: Span,
    exported: bool,
    signature: String,
    docs: String,
    rust_name: Option<String>,
}

#[derive(Clone, Copy)]
struct BuiltinDoc {
    name: &'static str,
    signature: &'static str,
    kind: SymbolKind,
    documentation: &'static str,
}

fn builtin_docs() -> &'static [BuiltinDoc] {
    &[
        BuiltinDoc {
            name: "void",
            signature: "void",
            kind: SymbolKind::TYPE_PARAMETER,
            documentation: "The unit type for functions that return no value.\n\n**Example**\n```orust\nvoid greet() {\n  print(\"hello\");\n}\n```\n\n**Rust mapping:** `()`.",
        },
        BuiltinDoc {
            name: "int",
            signature: "int",
            kind: SymbolKind::TYPE_PARAMETER,
            documentation: "A signed 64-bit integer. Use it for counters, indexes, and whole-number calculations.\n\n**Example**\n```orust\nint count = 3;\nprint(count + 1);\n```\n\n**Rust mapping:** `i64`.",
        },
        BuiltinDoc {
            name: "double",
            signature: "double",
            kind: SymbolKind::TYPE_PARAMETER,
            documentation: "A 64-bit floating-point number.\n\n**Example**\n```orust\ndouble ratio = 1.5;\nprint(ratio * 2);\n```\n\n**Rust mapping:** `f64`.",
        },
        BuiltinDoc {
            name: "bool",
            signature: "bool",
            kind: SymbolKind::TYPE_PARAMETER,
            documentation: "A boolean value, either `true` or `false`.\n\n**Example**\n```orust\nbool ready = true;\nif (ready) { print(\"ready\"); }\n```\n\n**Rust mapping:** `bool`.",
        },
        BuiltinDoc {
            name: "String",
            signature: "String",
            kind: SymbolKind::STRUCT,
            documentation: "An owned UTF-8 string. Strings are owned by default; use `lend` when passing a shared borrow.\n\n**Example**\n```orust\nString name = \"ORust\";\nprint(name.toUpperCase());\nprint(name.length);\n```\n\n**Rust mapping:** `String`.",
        },
        BuiltinDoc {
            name: "List",
            signature: "List<T>",
            kind: SymbolKind::STRUCT,
            documentation: "An ordered, growable collection of values.\n\n**Example**\n```orust\nList<int> values = [1, 2, 3];\nvalues.add(4);\nprint(values.length);\n```\n\n**Rust mapping:** `Vec<T>`.",
        },
        BuiltinDoc {
            name: "Map",
            signature: "Map<K, V>",
            kind: SymbolKind::STRUCT,
            documentation: "A key/value collection.\n\n**Example**\n```orust\nMap<String, int> scores;\nscores.put(\"Ada\", 10);\nprint(scores[\"Ada\"]);\n```\n\n**Rust mapping:** `HashMap<K, V>`.",
        },
        BuiltinDoc {
            name: "Set",
            signature: "Set<T>",
            kind: SymbolKind::STRUCT,
            documentation: "A collection of unique values.\n\n**Example**\n```orust\nSet<String> names;\nnames.add(\"Ada\");\nprint(names.contains(\"Ada\"));\n```\n\n**Rust mapping:** `HashSet<T>`.",
        },
        BuiltinDoc {
            name: "Option",
            signature: "Option<T>",
            kind: SymbolKind::ENUM,
            documentation: "A value that may be present or `null`. Use `?.` and `??` for safe access and defaults.\n\n**Example**\n```orust\nString? nickname = null;\nprint(nickname ?? \"guest\");\n```\n\n**Rust mapping:** `Option<T>`.",
        },
        BuiltinDoc {
            name: "Future",
            signature: "Future<T>",
            kind: SymbolKind::INTERFACE,
            documentation: "An asynchronous value produced later. Await it inside an `async` function.\n\n**Example**\n```orust\nFuture<int> load() async {\n  return 42;\n}\n\nasync void main() {\n  print(await load());\n}\n```\n\n**Rust mapping:** a Tokio-compatible future.",
        },
        BuiltinDoc {
            name: "print",
            signature: "void print(Any value)",
            kind: SymbolKind::FUNCTION,
            documentation: "Writes a value to standard output without requiring a Rust block.\n\n**Example**\n```orust\nprint(\"Hello, ORust!\");\nprint(42);\n```\n\n**Rust mapping:** generated `println!(\"{}\", value)` output.",
        },
        BuiltinDoc {
            name: "println",
            signature: "void println(Any value)",
            kind: SymbolKind::FUNCTION,
            documentation: "Writes a value followed by a newline.\n\n**Example**\n```orust\nprintln(\"done\");\n```\n\n**Rust mapping:** `println!`.",
        },
        BuiltinDoc {
            name: "toString",
            signature: "String toString()",
            kind: SymbolKind::METHOD,
            documentation: "Converts a value to its display string.\n\n**Example**\n```orust\nvar count = 7;\nprint(count.toString());\n```\n\n**Rust mapping:** `ToString::to_string()`.",
        },
        BuiltinDoc {
            name: "toUpperCase",
            signature: "String toUpperCase()",
            kind: SymbolKind::METHOD,
            documentation: "Returns an uppercase copy of a string.\n\n**Example**\n```orust\nprint(\"hello\".toUpperCase());\n```\n\n**Rust mapping:** `to_uppercase()`.",
        },
        BuiltinDoc {
            name: "toLowerCase",
            signature: "String toLowerCase()",
            kind: SymbolKind::METHOD,
            documentation: "Returns a lowercase copy of a string.\n\n**Example**\n```orust\nprint(\"HELLO\".toLowerCase());\n```\n\n**Rust mapping:** `to_lowercase()`.",
        },
        BuiltinDoc {
            name: "trim",
            signature: "String trim()",
            kind: SymbolKind::METHOD,
            documentation: "Returns a string with leading and trailing whitespace removed.\n\n**Example**\n```orust\nprint(\"  hello  \".trim());\n```\n\n**Rust mapping:** `trim()`.",
        },
        BuiltinDoc {
            name: "contains",
            signature: "bool contains(String value)",
            kind: SymbolKind::METHOD,
            documentation: "Checks whether a string or collection contains a value.\n\n**Example**\n```orust\nif (\"orust\".contains(\"rust\")) {\n  print(\"found\");\n}\n```\n\n**Rust mapping:** `contains()`.",
        },
        BuiltinDoc {
            name: "add",
            signature: "void add(T value)",
            kind: SymbolKind::METHOD,
            documentation: "Appends a value to a `List<T>`.\n\n**Example**\n```orust\nvar values = [1, 2];\nvalues.add(3);\n```\n\n**Rust mapping:** `Vec::push()`.",
        },
        BuiltinDoc {
            name: "put",
            signature: "void put(K key, V value)",
            kind: SymbolKind::METHOD,
            documentation: "Inserts or replaces a value in a `Map<K, V>`.\n\n**Example**\n```orust\nMap<String, int> scores;\nscores.put(\"Ada\", 10);\n```\n\n**Rust mapping:** `HashMap::insert()`.",
        },
        BuiltinDoc {
            name: "remove",
            signature: "bool remove(T value)",
            kind: SymbolKind::METHOD,
            documentation: "Removes a value from a collection and reports whether it was present.\n\n**Example**\n```orust\nvar values = [1, 2, 3];\nvalues.remove(2);\n```\n\n**Rust mapping:** collection-specific `remove` operation.",
        },
        BuiltinDoc {
            name: "isEmpty",
            signature: "bool isEmpty",
            kind: SymbolKind::FIELD,
            documentation: "Reports whether a string or collection contains no values.\n\n**Example**\n```orust\nvar values = [1, 2];\nif (!values.isEmpty) { print(\"has values\"); }\n```\n\n**Rust mapping:** `.is_empty()`.",
        },
        BuiltinDoc {
            name: "substring",
            signature: "String substring(int start, int end?)",
            kind: SymbolKind::METHOD,
            documentation: "Returns a slice of a string between the requested indexes.\n\n**Example**\n```orust\nprint(\"ORust\".substring(0, 2));\n```\n\n**Rust mapping:** a checked string slice conversion.",
        },
        BuiltinDoc {
            name: "where",
            signature: "Iterable<T> where(bool predicate(T value))",
            kind: SymbolKind::METHOD,
            documentation: "Filters a collection using a predicate.\n\n**Example**\n```orust\nvar values = [1, 2, 3];\nvar even = values.where(|value| value % 2 == 0);\n```\n\n**Rust mapping:** iterator `filter()`.",
        },
        BuiltinDoc {
            name: "map",
            signature: "Iterable<U> map(U transform(T value))",
            kind: SymbolKind::METHOD,
            documentation: "Transforms each value in a collection.\n\n**Example**\n```orust\nvar values = [1, 2, 3];\nvar doubled = values.map(|value| value * 2);\n```\n\n**Rust mapping:** iterator `map()`.",
        },
        BuiltinDoc {
            name: "toList",
            signature: "List<T> toList()",
            kind: SymbolKind::METHOD,
            documentation: "Collects an iterable into a list.\n\n**Example**\n```orust\nvar values = [1, 2, 3];\nvar copy = values.map(|value| value).toList();\n```\n\n**Rust mapping:** iterator `collect::<Vec<_>>()`.",
        },
        BuiltinDoc {
            name: "length",
            signature: "int length",
            kind: SymbolKind::FIELD,
            documentation: "The number of characters in a string or elements in a collection.\n\n**Example**\n```orust\nprint(\"ORust\".length);\n```\n\n**Rust mapping:** `.len()`.",
        },
    ]
}

fn builtin_doc(name: &str) -> Option<&'static BuiltinDoc> {
    builtin_docs().iter().find(|builtin| builtin.name == name)
}

#[derive(Clone, Debug)]
struct Document {
    version: i32,
    source: String,
    open: bool,
    program: Option<Program>,
    symbols: Vec<Symbol>,
}

#[derive(Default)]
struct Workspace {
    root: Option<PathBuf>,
    documents: HashMap<Url, Document>,
}

#[derive(Clone)]
struct Backend {
    client: Client,
    workspace: Arc<RwLock<Workspace>>,
}

impl Backend {
    fn line_col(source: &str, offset: usize) -> Position {
        let offset = offset.min(source.len());
        let prefix = &source[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
        let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
        let character = source[line_start..offset].encode_utf16().count();
        Position::new(line as u32, character as u32)
    }

    fn offset_at(source: &str, position: Position) -> usize {
        let mut line_start = 0;
        for _ in 0..position.line {
            let Some(relative) = source[line_start..].find('\n') else {
                return source.len();
            };
            line_start += relative + 1;
        }
        let line = &source[line_start..];
        let mut utf16 = 0;
        for (index, character) in line.char_indices() {
            if utf16 >= position.character as usize {
                return line_start + index;
            }
            utf16 += character.len_utf16();
        }
        line_start + line.find('\n').unwrap_or(line.len())
    }

    fn range(source: &str, span: Span) -> tower_lsp::lsp_types::Range {
        tower_lsp::lsp_types::Range::new(
            Self::line_col(source, span.start),
            Self::line_col(source, span.end),
        )
    }

    fn word_at(source: &str, offset: usize) -> Option<(String, Span)> {
        let bytes = source.as_bytes();
        if offset > source.len() {
            return None;
        }
        let mut start = offset.min(source.len());
        while start > 0 && is_ident_continue(bytes[start - 1]) {
            start -= 1;
        }
        let mut end = offset.min(source.len());
        while end < bytes.len() && is_ident_continue(bytes[end]) {
            end += 1;
        }
        (start < end).then(|| (source[start..end].to_owned(), Span { start, end }))
    }

    fn declaration_span(source: &str, enclosing: Span, name: &str) -> Span {
        let slice = &source[enclosing.start.min(source.len())..enclosing.end.min(source.len())];
        slice.find(name).map_or(enclosing, |start| Span {
            start: enclosing.start + start,
            end: enclosing.start + start + name.len(),
        })
    }

    fn doc_text(program: &Program, span: Span) -> String {
        program
            .resolved_docs_for_span(span)
            .map(|doc| {
                if doc.body.is_empty() {
                    doc.summary
                } else {
                    format!("{}\n\n{}", doc.summary, doc.body)
                }
            })
            .unwrap_or_default()
    }

    fn generated_signature(program: &Program, symbol: &Symbol) -> Option<String> {
        let generated = orust_emit::emit(program);
        let names = [
            symbol.name.as_str(),
            symbol.rust_name.as_deref().unwrap_or(""),
        ];
        generated
            .lines()
            .map(str::trim)
            .find(|line| {
                names.iter().any(|name| {
                    !name.is_empty()
                        && (line.contains(&format!("fn {name}"))
                            || line.contains(&format!("struct {name}"))
                            || line.contains(&format!("enum {name}"))
                            || line.contains(&format!("trait {name}")))
                })
            })
            .map(str::to_owned)
    }

    fn function_signature(function: &Function) -> String {
        let params = function
            .params
            .iter()
            .map(|param| format!("{} {}", param.ty, param.name))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} {}({})", function.return_type, function.name, params)
    }

    fn constructor_signature(constructor: &Constructor, class_name: &str) -> String {
        let params = constructor
            .params
            .iter()
            .map(|param| format!("{} {}", param.ty, param.name))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} {}({})", class_name, constructor.name, params)
    }

    fn collect_symbols(source: &str, program: &Program) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        for item in &program.items {
            match &item.node {
                Item::Class(class) => {
                    Self::collect_class_symbols(source, program, item.span, class, &mut symbols)
                }
                Item::Interface(interface) => {
                    let span = Self::declaration_span(source, item.span, &interface.name);
                    symbols.push(Symbol {
                        name: interface.name.clone(),
                        kind: SymbolKind::INTERFACE,
                        span,
                        exported: interface.exported,
                        signature: format!("interface {}", interface.name),
                        docs: Self::doc_text(program, item.span),
                        rust_name: interface.rust_name.clone(),
                    });
                    for method in &interface.methods {
                        Self::push_function_symbol(source, program, method, &mut symbols, true);
                    }
                }
                Item::Enum(enumeration) => {
                    let span = Self::declaration_span(source, item.span, &enumeration.name);
                    symbols.push(Symbol {
                        name: enumeration.name.clone(),
                        kind: SymbolKind::ENUM,
                        span,
                        exported: enumeration.exported,
                        signature: format!("enum {}", enumeration.name),
                        docs: Self::doc_text(program, item.span),
                        rust_name: enumeration.rust_name.clone(),
                    });
                    for variant in &enumeration.variants {
                        symbols.push(Symbol {
                            name: variant.name.clone(),
                            kind: SymbolKind::ENUM_MEMBER,
                            span: Self::declaration_span(source, variant.span, &variant.name),
                            exported: enumeration.exported,
                            signature: format!("{}::{}", enumeration.name, variant.name),
                            docs: String::new(),
                            rust_name: None,
                        });
                    }
                }
                Item::Error(error) => {
                    let span = Self::declaration_span(source, item.span, &error.name);
                    symbols.push(Symbol {
                        name: error.name.clone(),
                        kind: SymbolKind::ENUM,
                        span,
                        exported: error.exported,
                        signature: format!("error {}", error.name),
                        docs: Self::doc_text(program, item.span),
                        rust_name: error.rust_name.clone(),
                    });
                }
                Item::Function(function) => Self::push_function_symbol(
                    source,
                    program,
                    function,
                    &mut symbols,
                    function.exported,
                ),
                Item::TypeAlias(alias) => {
                    symbols.push(Symbol {
                        name: alias.name.clone(),
                        kind: SymbolKind::TYPE_PARAMETER,
                        span: Self::declaration_span(source, item.span, &alias.name),
                        exported: alias.exported,
                        signature: format!("typedef {} = {}", alias.name, alias.ty),
                        docs: Self::doc_text(program, item.span),
                        rust_name: alias.rust_name.clone(),
                    });
                }
                Item::Newtype(newtype) => {
                    symbols.push(Symbol {
                        name: newtype.name.clone(),
                        kind: SymbolKind::STRUCT,
                        span: Self::declaration_span(source, item.span, &newtype.name),
                        exported: newtype.exported,
                        signature: format!("type {}({})", newtype.name, newtype.inner),
                        docs: Self::doc_text(program, item.span),
                        rust_name: newtype.rust_name.clone(),
                    });
                }
                Item::Extension(extension) => {
                    for method in &extension.methods {
                        Self::push_function_symbol(
                            source,
                            program,
                            method,
                            &mut symbols,
                            extension.exported,
                        );
                    }
                }
            }
        }
        symbols.extend(Self::local_symbols(source));
        symbols
    }

    fn collect_class_symbols(
        source: &str,
        program: &Program,
        item_span: Span,
        class: &Class,
        symbols: &mut Vec<Symbol>,
    ) {
        let span = Self::declaration_span(source, item_span, &class.name);
        symbols.push(Symbol {
            name: class.name.clone(),
            kind: SymbolKind::CLASS,
            span,
            exported: class.exported,
            signature: format!("class {}", class.name),
            docs: Self::doc_text(program, item_span),
            rust_name: class.rust_name.clone(),
        });
        for field in &class.fields {
            symbols.push(Symbol {
                name: field.name.clone(),
                kind: SymbolKind::FIELD,
                span: Self::declaration_span(source, field.span, &field.name),
                exported: class.exported,
                signature: format!("{} {}", field.ty, field.name),
                docs: Self::doc_text(program, field.span),
                rust_name: None,
            });
        }
        for constructor in &class.constructors {
            symbols.push(Symbol {
                name: constructor.name.clone(),
                kind: SymbolKind::CONSTRUCTOR,
                span: Self::declaration_span(source, constructor.span, &constructor.name),
                exported: class.exported,
                signature: Self::constructor_signature(constructor, &class.name),
                docs: Self::doc_text(program, constructor.span),
                rust_name: None,
            });
        }
        for method in &class.methods {
            Self::push_function_symbol(
                source,
                program,
                method,
                symbols,
                class.exported || method.exported,
            );
        }
    }

    fn push_function_symbol(
        source: &str,
        program: &Program,
        function: &Function,
        symbols: &mut Vec<Symbol>,
        exported: bool,
    ) {
        symbols.push(Symbol {
            name: function.name.clone(),
            kind: SymbolKind::FUNCTION,
            span: Self::declaration_span(source, function.span, &function.name),
            exported,
            signature: Self::function_signature(function),
            docs: Self::doc_text(program, function.span),
            rust_name: function.rust_name.clone(),
        });
        for param in &function.params {
            symbols.push(Symbol {
                name: param.name.clone(),
                kind: SymbolKind::VARIABLE,
                span: Self::declaration_span(source, param.span, &param.name),
                exported: false,
                signature: format!("{} {}", param.ty, param.name),
                docs: String::new(),
                rust_name: None,
            });
        }
    }

    fn local_symbols(source: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut cursor = 0;
        while let Some(relative) = source[cursor..].find("var ") {
            let start = cursor + relative + 4;
            let end = start
                + source[start..]
                    .bytes()
                    .take_while(|byte| is_ident_continue(*byte))
                    .count();
            if start < end {
                symbols.push(Symbol {
                    name: source[start..end].to_owned(),
                    kind: SymbolKind::VARIABLE,
                    span: Span { start, end },
                    exported: false,
                    signature: "inferred local".into(),
                    docs: String::new(),
                    rust_name: None,
                });
            }
            cursor = end.max(cursor + 4);
        }
        symbols
    }

    fn document(&self, uri: &Url) -> Option<Document> {
        self.workspace.read().ok()?.documents.get(uri).cloned()
    }

    fn all_symbols(&self) -> Vec<(Url, Symbol)> {
        self.workspace
            .read()
            .map(|workspace| {
                workspace
                    .documents
                    .iter()
                    .flat_map(|(uri, document)| {
                        document
                            .symbols
                            .iter()
                            .cloned()
                            .map(|symbol| (uri.clone(), symbol))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn visible_symbols(&self, uri: &Url) -> Vec<(Url, Symbol)> {
        self.all_symbols()
            .into_iter()
            .filter(|(candidate_uri, symbol)| candidate_uri == uri || symbol.exported)
            .collect()
    }

    fn import_at(program: &Program, offset: usize) -> Option<&Import> {
        program
            .imports
            .iter()
            .find(|import| offset >= import.span.start && offset <= import.span.end)
    }

    fn resolve_import_path(&self, current: &Url, path: &str) -> Option<PathBuf> {
        let current_path = current.to_file_path().ok()?;
        let raw = path.strip_prefix("rust:").unwrap_or(path);
        let base = current_path.parent()?;
        let candidate = if raw.starts_with("./") || raw.starts_with("../") {
            base.join(raw)
        } else if raw.starts_with("crate::") {
            let module = raw
                .trim_start_matches("crate::")
                .split("::")
                .next()
                .unwrap_or_default();
            let workspace_root = self.workspace.read().ok()?.root.clone()?;
            let module = Path::new(module);
            let candidates = [
                workspace_root.join("src").join(module).with_extension("rs"),
                workspace_root.join(module).with_extension("rs"),
            ];
            candidates.into_iter().find(|path| path.exists())?
        } else {
            base.join(raw)
        };
        if candidate.exists() {
            Some(candidate)
        } else if candidate.extension().is_none() {
            [
                candidate.with_extension("or"),
                candidate.with_extension("rs"),
            ]
            .into_iter()
            .find(|path| path.exists())
        } else {
            None
        }
    }

    fn rust_import_for_name(program: &Program, name: &str) -> Option<String> {
        program.items.iter().find_map(|item| match &item.node {
            Item::Function(function) if function.name == name => function.rust_import.clone(),
            Item::TypeAlias(alias) if alias.name == name => alias.rust_import.clone(),
            _ => None,
        })
    }

    fn identifier_location(source: &str, name: &str) -> Span {
        let mut cursor = 0;
        while let Some(relative) = source[cursor..].find(name) {
            let start = cursor + relative;
            let end = start + name.len();
            let left_ok = start == 0 || !is_ident_continue(source.as_bytes()[start - 1]);
            let right_ok = end == source.len() || !is_ident_continue(source.as_bytes()[end]);
            if left_ok && right_ok {
                return Span { start, end };
            }
            cursor = end;
        }
        Span { start: 0, end: 0 }
    }

    fn annotation_at(source: &str, offset: usize) -> Option<(String, String, Span)> {
        let line_start = source[..offset.min(source.len())]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        let line_end = source[offset.min(source.len())..]
            .find('\n')
            .map_or(source.len(), |index| offset.min(source.len()) + index);
        let line = &source[line_start..line_end];
        let at = line.find('@')?;
        let name_start = at + 1;
        let name_end = name_start
            + line[name_start..]
                .bytes()
                .take_while(|byte| is_ident_continue(*byte))
                .count();
        let annotation = &line[name_start..name_end];
        let open = line[name_end..].find('(').map(|index| name_end + index)?;
        let quote_start = open + line[open..].find('"').or_else(|| line[open..].find('\''))?;
        let quote = line.as_bytes()[quote_start];
        let value_start = quote_start + 1;
        let relative_end = line.as_bytes()[value_start..]
            .iter()
            .position(|byte| *byte == quote)?;
        let value_end = value_start + relative_end;
        let span = Span {
            start: line_start + value_start,
            end: line_start + value_end,
        };
        (offset >= line_start + at && offset <= line_start + value_end).then(|| {
            (
                annotation.to_owned(),
                line[value_start..value_end].to_owned(),
                span,
            )
        })
    }

    fn annotation_hover(&self, uri: &Url, source: &str, offset: usize) -> Option<(String, Span)> {
        let (annotation, value, span) = Self::annotation_at(source, offset)?;
        let mut markdown = format!(
            "### @{}\n\nValue: {}\n\nThis annotation connects the ORust declaration to generated Rust.",
            annotation, value
        );
        match annotation.as_str() {
            "rustImport" | "externRust" => {
                markdown.push_str(
                    "\n\nRelationship: imports a Rust function, type, or item into the ORust declaration.",
                );
                if let Some(path) = self.resolve_import_path(uri, &format!("rust:{value}")) {
                    let target_source = fs::read_to_string(&path).unwrap_or_default();
                    let member = value.rsplit("::").next().unwrap_or(value.as_str());
                    let target_span = Self::identifier_location(&target_source, member);
                    let target_line = Self::line_col(&target_source, target_span.start).line + 1;
                    let target_url = Url::from_file_path(&path).ok();
                    let target = target_url
                        .map(|url| format!("[{}]({url})", path.display()))
                        .unwrap_or_else(|| path.display().to_string());
                    markdown.push_str(&format!(
                        "\n\nLeads to: {target}, symbol {member}, line {target_line}.\n\nClick the linked path to open the Rust source."
                    ));
                } else {
                    markdown.push_str(
                        "\n\nLeads to: a Rust item resolved by Cargo or the generated crate; no local source file was found.",
                    );
                }
            }
            "rustType" => markdown.push_str(
                "\n\nRelationship: makes this ORust name an exact alias for the Rust type. Cargo resolves the path and rustc checks every use.",
            ),
            "rustName" => markdown.push_str(
                "\n\nRelationship: keeps the ORust source name while changing the generated Rust declaration name.",
            ),
            "derive" => markdown.push_str(
                "\n\nRelationship: forwards the selected Rust derives to the generated declaration.",
            ),
            "repr" => markdown.push_str(
                "\n\nRelationship: controls the generated Rust memory representation.",
            ),
            "cfg" => markdown.push_str(
                "\n\nRelationship: applies a Rust conditional-compilation expression.",
            ),
            "borrowed" => markdown.push_str(
                "\n\nRelationship: records the lifetime used by the generated borrowed boundary.",
            ),
            "owned" => markdown.push_str(
                "\n\nRelationship: marks the boundary as owned rather than borrowed.",
            ),
            _ => {}
        }
        Some((markdown, span))
    }

    async fn schedule_parse(&self, uri: Url, version: i32, source: String, publish: bool) {
        if let Ok(mut workspace) = self.workspace.write() {
            workspace.documents.insert(
                uri.clone(),
                Document {
                    version,
                    source: source.clone(),
                    open: publish,
                    program: None,
                    symbols: Vec::new(),
                },
            );
        }
        let backend = self.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(40)).await;
            let parsed = parse(&source);
            let (program, diagnostics, symbols) = match parsed {
                Ok(program) => {
                    let diagnostics = program
                        .lints
                        .iter()
                        .map(|lint| tower_lsp::lsp_types::Diagnostic {
                            range: Backend::range(&source, lint.span),
                            severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::WARNING),
                            code: Some(tower_lsp::lsp_types::NumberOrString::String(
                                lint.code.into(),
                            )),
                            source: Some("orust".into()),
                            message: lint.message.clone(),
                            ..Default::default()
                        })
                        .collect();
                    let symbols = Backend::collect_symbols(&source, &program);
                    (Some(program), diagnostics, symbols)
                }
                Err(error) => {
                    let diagnostic = tower_lsp::lsp_types::Diagnostic {
                        range: Backend::range(&source, error.span),
                        severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
                        code: Some(tower_lsp::lsp_types::NumberOrString::String(
                            "OR0001".into(),
                        )),
                        source: Some("orust".into()),
                        message: explain_parse_error(&error.message),
                        ..Default::default()
                    };
                    (None, vec![diagnostic], Vec::new())
                }
            };
            let should_publish = backend
                .workspace
                .write()
                .ok()
                .and_then(|mut workspace| {
                    let document = workspace.documents.get_mut(&uri)?;
                    if document.version != version {
                        return Some(false);
                    }
                    document.program = program;
                    document.symbols = symbols;
                    Some(publish && document.open)
                })
                .unwrap_or(false);
            if should_publish {
                backend
                    .client
                    .publish_diagnostics(uri, diagnostics, Some(version))
                    .await;
            }
        });
    }

    async fn load_workspace(&self, root: &Path) {
        let mut files = Vec::new();
        collect_or_files(root, &mut files);
        for path in files {
            let Ok(source) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(uri) = Url::from_file_path(&path) else {
                continue;
            };
            self.schedule_parse(uri, 0, source, false).await;
        }
    }

    fn rust_ranges(source: &str) -> Vec<Span> {
        let mut ranges = Vec::new();
        let mut cursor = 0;
        while let Some(relative) = source[cursor..].find("rust") {
            let start = cursor + relative;
            let after = start + 4;
            let Some(open_relative) = source[after..].find('{') else {
                break;
            };
            let open = after + open_relative;
            let mut depth = 0;
            let mut end = open;
            for (index, character) in source[open..].char_indices() {
                match character {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = open + index + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end > open {
                ranges.push(Span { start, end });
                cursor = end;
            } else {
                break;
            }
        }
        ranges
    }

    fn in_ranges(span: Span, ranges: &[Span]) -> bool {
        ranges
            .iter()
            .any(|range| span.start >= range.start && span.end <= range.end)
    }
}

fn is_ident_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn explain_parse_error(message: &str) -> String {
    if message.contains("expected Symbol(';')") {
        return format!(
            "{message}\n\nExample:\n  return value;\n\nEvery ORust statement must end with `;`."
        );
    }
    if message.contains("expected identifier") {
        return format!(
            "{message}\n\nExample:\n  String name;\n\nUse an identifier after the declaration keyword or type."
        );
    }
    if message.contains("unterminated string") {
        return format!(
            "{message}\n\nExample:\n  print(\"hello\");\n\nClose the string with the same quote character that opened it."
        );
    }
    message.to_owned()
}

fn completion_kind(kind: SymbolKind) -> CompletionItemKind {
    match kind {
        SymbolKind::CLASS => CompletionItemKind::CLASS,
        SymbolKind::INTERFACE => CompletionItemKind::INTERFACE,
        SymbolKind::ENUM => CompletionItemKind::ENUM,
        SymbolKind::ENUM_MEMBER => CompletionItemKind::ENUM_MEMBER,
        SymbolKind::FIELD => CompletionItemKind::FIELD,
        SymbolKind::CONSTRUCTOR => CompletionItemKind::CONSTRUCTOR,
        SymbolKind::FUNCTION | SymbolKind::METHOD => CompletionItemKind::FUNCTION,
        SymbolKind::VARIABLE => CompletionItemKind::VARIABLE,
        SymbolKind::STRUCT => CompletionItemKind::STRUCT,
        _ => CompletionItemKind::KEYWORD,
    }
}

fn collect_or_files(root: &Path, output: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "target" || name == "node_modules" || name == ".git")
        {
            continue;
        }
        if path.is_dir() {
            collect_or_files(&path, output);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("or") {
            output.push(path);
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let root = params
            .root_uri
            .as_ref()
            .and_then(|uri| uri.to_file_path().ok());
        if let Ok(mut workspace) = self.workspace.write() {
            workspace.root = root.clone();
        }
        if let Some(root) = root {
            self.load_workspace(&root).await;
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".into(), ":".into()]),
                    ..Default::default()
                }),
                hover_provider: Some(tower_lsp::lsp_types::HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(
                    tower_lsp::lsp_types::CodeActionProviderCapability::Simple(true),
                ),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".into(), ",".into()]),
                    ..Default::default()
                }),
                inlay_hint_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "orust-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: tower_lsp::lsp_types::InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ORust language server ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.schedule_parse(
            params.text_document.uri,
            params.text_document.version,
            params.text_document.text,
            true,
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(TextDocumentContentChangeEvent { text, .. }) =
            params.content_changes.into_iter().last()
        else {
            return;
        };
        self.schedule_parse(
            params.text_document.uri,
            params.text_document.version,
            text,
            true,
        )
        .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Ok(path) = params.text_document.uri.to_file_path() {
            if let Ok(source) = fs::read_to_string(path) {
                let version = self
                    .document(&params.text_document.uri)
                    .map(|document| document.version + 1)
                    .unwrap_or(0);
                self.schedule_parse(params.text_document.uri, version, source, true)
                    .await;
            }
        }
    }

    async fn did_close(&self, params: tower_lsp::lsp_types::DidCloseTextDocumentParams) {
        if let Ok(mut workspace) = self.workspace.write() {
            workspace.documents.remove(&params.text_document.uri);
        }
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let document = self.document(&params.text_document_position_params.text_document.uri);
        let Some(document) = document else {
            return Ok(None);
        };
        let offset = Self::offset_at(
            &document.source,
            params.text_document_position_params.position,
        );
        if let Some(program) = document.program.as_ref() {
            if let Some(import) = Self::import_at(program, offset) {
                if let Some(path) = self.resolve_import_path(
                    &params.text_document_position_params.text_document.uri,
                    &import.path,
                ) {
                    if let Ok(uri) = Url::from_file_path(path) {
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                            uri,
                            tower_lsp::lsp_types::Range::new(
                                Position::new(0, 0),
                                Position::new(0, 0),
                            ),
                        ))));
                    }
                }
            }
        }
        let Some((name, _)) = Self::word_at(&document.source, offset) else {
            return Ok(None);
        };
        if let Some(program) = document.program.as_ref() {
            if let Some(import_path) = Self::rust_import_for_name(program, &name) {
                if let Some(path) = self.resolve_import_path(
                    &params.text_document_position_params.text_document.uri,
                    &format!("rust:{import_path}"),
                ) {
                    if let Ok(uri) = Url::from_file_path(&path) {
                        let source = fs::read_to_string(&path).unwrap_or_default();
                        let member = import_path.rsplit("::").next().unwrap_or(&name);
                        let span = Self::identifier_location(&source, member);
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location::new(
                            uri,
                            Self::range(&source, span),
                        ))));
                    }
                }
            }
        }
        let current = params.text_document_position_params.text_document.uri;
        let target = self
            .visible_symbols(&current)
            .into_iter()
            .find(|(_, symbol)| symbol.name == name);
        Ok(target.and_then(|(uri, symbol)| {
            let source = self.document(&uri)?.source;
            Some(GotoDefinitionResponse::Scalar(Location::new(
                uri,
                Self::range(&source, symbol.span),
            )))
        }))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let Some(document) = self.document(&params.text_document_position.text_document.uri) else {
            return Ok(None);
        };
        let Some((name, _)) = Self::word_at(
            &document.source,
            Self::offset_at(&document.source, params.text_document_position.position),
        ) else {
            return Ok(None);
        };
        let mut locations = Vec::new();
        for (uri, candidate) in self
            .workspace
            .read()
            .map(|workspace| {
                workspace
                    .documents
                    .iter()
                    .map(|(uri, doc)| (uri.clone(), doc.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
        {
            let mut cursor = 0;
            while let Some(relative) = candidate.source[cursor..].find(&name) {
                let start = cursor + relative;
                let span = Span {
                    start,
                    end: start + name.len(),
                };
                cursor = span.end;
                let left_ok =
                    start == 0 || !is_ident_continue(candidate.source.as_bytes()[start - 1]);
                let right_ok = span.end == candidate.source.len()
                    || !is_ident_continue(candidate.source.as_bytes()[span.end]);
                if left_ok && right_ok {
                    locations.push(Location::new(
                        uri.clone(),
                        Self::range(&candidate.source, span),
                    ));
                }
            }
        }
        Ok(Some(locations))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let Some(document) = self.document(&params.text_document_position.text_document.uri) else {
            return Ok(None);
        };
        let Some((old, _)) = Self::word_at(
            &document.source,
            Self::offset_at(&document.source, params.text_document_position.position),
        ) else {
            return Ok(None);
        };
        let mut changes = HashMap::new();
        for (uri, candidate) in self
            .workspace
            .read()
            .map(|workspace| {
                workspace
                    .documents
                    .iter()
                    .map(|(uri, doc)| (uri.clone(), doc.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
        {
            let mut edits = Vec::new();
            let mut cursor = 0;
            let ranges = Self::rust_ranges(&candidate.source);
            while let Some(relative) = candidate.source[cursor..].find(&old) {
                let start = cursor + relative;
                let span = Span {
                    start,
                    end: start + old.len(),
                };
                cursor = span.end;
                if (start == 0 || !is_ident_continue(candidate.source.as_bytes()[start - 1]))
                    && (span.end == candidate.source.len()
                        || !is_ident_continue(candidate.source.as_bytes()[span.end]))
                    && !Self::in_ranges(span, &ranges)
                {
                    edits.push(TextEdit::new(
                        Self::range(&candidate.source, span),
                        params.new_name.clone(),
                    ));
                }
            }
            if !edits.is_empty() {
                changes.insert(uri, edits);
            }
        }
        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let keywords = [
            "class",
            "interface",
            "enum",
            "error",
            "void",
            "var",
            "new",
            "return",
            "if",
            "else",
            "while",
            "for",
            "lend",
            "copy",
            "shared",
            "rust",
            "import",
            "export",
            "show",
            "hide",
            "as",
        ];
        let mut items = keywords
            .into_iter()
            .map(|label| CompletionItem::new_simple(label.into(), "ORust keyword".into()))
            .collect::<Vec<_>>();
        items.extend(builtin_docs().iter().map(|builtin| CompletionItem {
            label: builtin.name.into(),
            kind: Some(completion_kind(builtin.kind)),
            detail: Some(builtin.signature.into()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: builtin.documentation.into(),
            })),
            ..Default::default()
        }));
        let mut seen = HashSet::new();
        for (_, symbol) in self.visible_symbols(&uri) {
            if seen.insert(symbol.name.clone()) {
                items.push(CompletionItem {
                    label: symbol.name,
                    kind: Some(completion_kind(symbol.kind)),
                    detail: Some(symbol.signature),
                    documentation: (!symbol.docs.is_empty())
                        .then_some(Documentation::String(symbol.docs)),
                    ..Default::default()
                });
            }
        }
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let position = params.text_document_position_params;
        let Some(document) = self.document(&position.text_document.uri) else {
            return Ok(None);
        };
        let offset = Self::offset_at(&document.source, position.position);
        if let Some((value, span)) =
            self.annotation_hover(&position.text_document.uri, &document.source, offset)
        {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: Some(Self::range(&document.source, span)),
            }));
        }
        let Some((name, word_span)) = Self::word_at(&document.source, offset) else {
            return Ok(None);
        };
        if let Some(program) = document.program.as_ref() {
            if let Some(import) = Self::import_at(program, offset) {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!(
                            "```orust\nimport '{}'\n```\n\nCtrl-click to open the imported source.",
                            import.path
                        ),
                    }),
                    range: Some(Self::range(&document.source, import.span)),
                }));
            }
        }
        let symbol = self
            .visible_symbols(&position.text_document.uri)
            .into_iter()
            .find(|(_, symbol)| symbol.name == name)
            .map(|(_, symbol)| symbol);
        if symbol.is_none() {
            if let Some(builtin) = builtin_doc(&name) {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!(
                            "```orust\n{}\n```\n\n{}",
                            builtin.signature, builtin.documentation
                        ),
                    }),
                    range: Some(Self::range(&document.source, word_span)),
                }));
            }
        }
        Ok(symbol.map(|symbol| {
            let mut value = format!("```orust\n{}\n```", symbol.signature);
            if let Some(rust_name) = symbol.rust_name.as_ref() {
                value.push_str(&format!("\n\nRust name: `{rust_name}`"));
            }
            if let Some(program) = document.program.as_ref() {
                if let Some(generated) = Self::generated_signature(program, &symbol) {
                    value.push_str(&format!("\n\nGenerated Rust: `{generated}`"));
                }
            }
            let ownership = if symbol.signature.contains("shared") {
                "shared"
            } else if symbol.signature.contains("lend mut") {
                "lend mut"
            } else if symbol.signature.contains("lend") {
                "lend"
            } else {
                "owned/copy inferred"
            };
            value.push_str(&format!("\n\nOwnership: `{ownership}`"));
            if !symbol.docs.is_empty() {
                value.push_str(&format!("\n\n{}", symbol.docs));
            }
            Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                }),
                range: Some(Self::range(&document.source, symbol.span)),
            }
        }))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(None);
        };
        let symbols = document
            .symbols
            .into_iter()
            .filter(|symbol| {
                symbol.exported || matches!(symbol.kind, SymbolKind::VARIABLE | SymbolKind::FIELD)
            })
            .map(|symbol| tower_lsp::lsp_types::DocumentSymbol {
                name: symbol.name,
                detail: Some(symbol.signature),
                kind: symbol.kind,
                tags: None,
                deprecated: None,
                range: Self::range(&document.source, symbol.span),
                selection_range: Self::range(&document.source, symbol.span),
                children: None,
            })
            .collect();
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        Ok(Some(
            self.all_symbols()
                .into_iter()
                .filter(|(_, symbol)| symbol.name.to_lowercase().contains(&query))
                .filter_map(|(uri, symbol)| {
                    let source = self.document(&uri)?.source;
                    Some(SymbolInformation {
                        name: symbol.name,
                        kind: symbol.kind,
                        tags: None,
                        deprecated: None,
                        location: Location::new(uri, Self::range(&source, symbol.span)),
                        container_name: None,
                    })
                })
                .collect(),
        ))
    }

    async fn code_action(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<Vec<CodeActionOrCommand>>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(None);
        };
        let mut actions = Vec::new();
        if params.context.diagnostics.iter().any(|diagnostic| {
            matches!(diagnostic.code.as_ref(), Some(tower_lsp::lsp_types::NumberOrString::String(code)) if code == "OR0601")
        }) {
            let edit = TextEdit::new(
                tower_lsp::lsp_types::Range::new(
                    Self::line_col(&document.source, 0),
                    Self::line_col(&document.source, 0),
                ),
                "export ".into(),
            );
            let workspace_edit = WorkspaceEdit {
                changes: Some(HashMap::from([(
                    params.text_document.uri.clone(),
                    vec![edit],
                )])),
                document_changes: None,
                change_annotations: None,
            };
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Add `export` to this declaration".into(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(params.context.diagnostics.clone()),
                edit: Some(workspace_edit),
                command: None,
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));
        }
        Ok(Some(actions))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(None);
        };
        let formatted = orust_syntax::format_source(&document.source);
        if formatted == document.source {
            return Ok(Some(Vec::new()));
        }
        let end = Self::line_col(&document.source, document.source.len());
        Ok(Some(vec![TextEdit::new(
            tower_lsp::lsp_types::Range::new(Position::new(0, 0), end),
            formatted,
        )]))
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let Some(document) = self.document(&params.text_document_position_params.text_document.uri)
        else {
            return Ok(None);
        };
        let offset = Self::offset_at(
            &document.source,
            params.text_document_position_params.position,
        );
        let prefix = &document.source[..offset.min(document.source.len())];
        let name = prefix
            .rsplit_once('(')
            .and_then(|(_, before)| before.split_whitespace().last())
            .unwrap_or("");
        let symbol = self
            .all_symbols()
            .into_iter()
            .find(|(_, symbol)| symbol.name == name)
            .map(|(_, symbol)| symbol);
        if let Some(symbol) = symbol {
            return Ok(Some(SignatureHelp {
                signatures: vec![SignatureInformation {
                    label: symbol.signature,
                    documentation: (!symbol.docs.is_empty())
                        .then_some(Documentation::String(symbol.docs)),
                    parameters: None,
                    active_parameter: None,
                }],
                active_signature: Some(0),
                active_parameter: Some(0),
            }));
        }
        Ok(builtin_doc(name).map(|builtin| SignatureHelp {
            signatures: vec![SignatureInformation {
                label: builtin.signature.into(),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: builtin.documentation.into(),
                })),
                parameters: None,
                active_parameter: None,
            }],
            active_signature: Some(0),
            active_parameter: Some(0),
        }))
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let Some(document) = self.document(&params.text_document.uri) else {
            return Ok(None);
        };
        let hints = document
            .symbols
            .into_iter()
            .filter(|symbol| {
                symbol.kind == SymbolKind::VARIABLE && symbol.signature == "inferred local"
            })
            .map(|symbol| InlayHint {
                position: Self::line_col(&document.source, symbol.span.end),
                label: InlayHintLabel::String(": inferred".into()),
                kind: None,
                text_edits: None,
                tooltip: None,
                padding_left: Some(true),
                padding_right: None,
                data: None,
            })
            .collect();
        Ok(Some(hints))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend {
        client,
        workspace: Arc::new(RwLock::new(Workspace::default())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_round_trip_utf16_columns() {
        let source = "void main() {\n  var café = \"😀\";\n}\n";
        let offset = source.find("café").unwrap() + "café".len();
        let position = Backend::line_col(source, offset);
        assert_eq!(Backend::offset_at(source, position), offset);
    }

    #[test]
    fn symbol_index_contains_declarations_and_members() {
        let source = "export class User { String name; User(this.name); String label() { return this.name; } }";
        let program = parse(source).unwrap();
        let symbols = Backend::collect_symbols(source, &program);
        assert!(symbols
            .iter()
            .any(|symbol| symbol.name == "User" && symbol.kind == SymbolKind::CLASS));
        assert!(symbols
            .iter()
            .any(|symbol| symbol.name == "name" && symbol.kind == SymbolKind::FIELD));
        assert!(symbols
            .iter()
            .any(|symbol| symbol.name == "label" && symbol.kind == SymbolKind::FUNCTION));
    }

    #[test]
    fn rename_protection_identifies_rust_passthrough() {
        let source = "void main() { rust { let value = 1; } var value = 2; }";
        let ranges = Backend::rust_ranges(source);
        let rust_value = source.find("value").unwrap();
        let orust_value = source.rfind("value").unwrap();
        assert!(Backend::in_ranges(
            Span {
                start: rust_value,
                end: rust_value + 5
            },
            &ranges
        ));
        assert!(!Backend::in_ranges(
            Span {
                start: orust_value,
                end: orust_value + 5
            },
            &ranges
        ));
    }

    #[test]
    fn builtin_catalog_documents_types_functions_and_methods() {
        for name in ["String", "List", "Option", "print", "toUpperCase", "add"] {
            let builtin = builtin_doc(name).expect("builtin documentation");
            assert!(!builtin.signature.is_empty());
            assert!(builtin.documentation.contains("**Example**"));
        }
    }
}
