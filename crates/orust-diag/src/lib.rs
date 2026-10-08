use orust_emit::SpanMapping;
use orust_syntax::Span;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: String,
    pub code: Option<String>,
    pub message: String,
    pub source_span: Option<Span>,
}

pub fn orust_code(rust_code: Option<&str>) -> Option<&'static str> {
    match rust_code {
        Some("E0382") | Some("E0505") => Some("OR0001"),
        Some("E0499") | Some("E0502") => Some("OR0002"),
        Some("E0432") | Some("E0433") | Some("E0603") => Some("OR0003"),
        Some("E0616") | Some("E0624") | Some("E0446") => Some("OR0004"),
        Some("E0106") | Some("E0515") | Some("E0597") | Some("E0506") => Some("OR0005"),
        Some("E0728") => Some("OR0006"),
        Some("E0277") | Some("E0308") => Some("OR0007"),
        Some("arithmetic_overflow") | Some("overflowing_literals") => Some("OR0008"),
        Some("E0046") | Some("E0050") | Some("E0053") | Some("E0119") | Some("E0191")
        | Some("E0220") | Some("E0407") => Some("OR0015"),
        Some("E0005") => Some("OR0016"),
        Some("unreachable_patterns") => Some("OR0017"),
        _ => None,
    }
}

pub fn rewrite_runtime_panic(message: &str) -> String {
    if message.contains("already borrowed: BorrowMutError") {
        "shared class was already being edited; use one `lend mut` at a time".into()
    } else if message.contains("already borrowed: BorrowError") {
        "shared class was already being viewed mutably; finish the edit before using `lend`".into()
    } else if let Some(rest) = message.strip_prefix("byte index ") {
        if let Some(index) = rest.split_whitespace().next() {
            format!("byte {index} is in the middle of a character")
        } else {
            message.into()
        }
    } else if message.contains("attempt to divide by zero") {
        "integer division by zero".into()
    } else if message.contains("called `Option::unwrap()` on a `None` value") {
        "tried to unwrap null; check the value before using it".into()
    } else {
        message.into()
    }
}

pub fn translate(diagnostic: &Diagnostic, source: &str, file_name: &str) -> String {
    let name = quoted_name(&diagnostic.message).unwrap_or_else(|| "value".into());
    let headline = match diagnostic.code.as_deref() {
        Some("E0382") => format!("`{name}` no longer has its Counter"),
        Some("E0499") => {
            format!("two things are trying to edit `{name}` at once. Only one editor at a time.")
        }
        Some("E0502") if diagnostic.message.contains("iter") => {
            format!("this lazy iterator chain is still reading `{name}`; call `.toList()` first or finish iterating before modifying it.")
        }
        Some("E0502") => format!("you changed `{name}` while it was still being looked at."),
        Some("E0106") => "this reference needs a lifetime; try explicit `lend(a)` syntax.".into(),
        Some("E0515") => format!(
            "`{name}` returns a borrowed value that is still looking at a local variable. Return an owned value or pass the source in from outside."
        ),
        Some("E0506") => format!("you changed `{name}` while it was still being looked at."),
        Some("E0505") => {
            format!("tried to give away `{name}` while something was still looking at it.")
        }
        Some("E0373") => "this spawned task may outlive the values it borrowed.".into(),
        Some("E0277") if diagnostic.message.contains("trait") => {
            format!("`{name}` is missing a trait bound required by this generic operation.")
        }
        Some("E0277") => {
            "this spawned task uses a value that cannot be sent safely between threads.".into()
        }
        Some("E0597") if diagnostic.message.contains("drop") => format!(
            "`{name}` runs cleanup code that may still look at a borrowed value, so that value must outlive `{name}.`"
        ),
        Some("E0597") => format!("`{name}` does not live long enough for this async work."),
        Some("E0521") => "a borrowed value is escaping into async work that may outlive it.".into(),
        Some("E0728") => "`await` can only be used inside an async function.".into(),
        Some("E0038") => "this interface cannot be used as a dynamic trait object.".into(),
        Some("E0599") => format!("`{name}` does not have the method this generic code requested."),
        Some("E0004") => "this match does not handle every enum case.".into(),
        Some("E0046") => "this class says it implements an interface but is missing a required method.".into(),
        Some("E0050") | Some("E0053") => "this implementation method has the wrong parameter or return signature.".into(),
        Some("E0119") => "this trait implementation conflicts with another implementation.".into(),
        Some("E0191") | Some("E0220") => "this implementation is missing or names an unknown associated type.".into(),
        Some("E0407") => "this method is not declared by the interface it implements.".into(),
        Some("E0005") => "this pattern may not match every value; add an `else` branch.".into(),
        Some("unreachable_patterns") => "this pattern can never be reached because an earlier pattern matches first.".into(),
        Some("E0432") => "this import names an item that the target file does not export.".into(),
        Some("E0433") => "this namespace is not available; check the aliased import path.".into(),
        Some("E0603") => "this item is private to its defining file; add `export` there.".into(),
        Some("E0616") => "this field is private to the class; expose it through a method or remove `private`.".into(),
        Some("E0624") => "this method is private and cannot be called from this file.".into(),
        Some("E0252") | Some("E0254") | Some("E0255") => "this imported name collides with another name; use `show`, `hide`, or `as`.".into(),
        Some("E0446") => "a public signature mentions a private type; export the type or make the member private.".into(),
        _ => diagnostic.message.clone(),
    };
    let (line, column, source_line) = diagnostic
        .source_span
        .map(|span| source_location(source, span.start))
        .unwrap_or((1, 1, ""));
    let mut output = format!(
        "error: {headline}\n --> {file_name}:{line}:{column}\n  |\n{line} | {source_line}\n  |\n"
    );
    match diagnostic.code.as_deref() {
        Some("E0382") => output.push_str(&format!("help: give it its own Counter: `copy {name}`\nhelp: let it look without owning: `lend {name}`\nhelp: share it explicitly with `shared class`\n")),
        Some("E0502") if diagnostic.message.contains("iter") => output.push_str("help: materialize the chain with `.toList()`, or finish the chain before calling `add()`/`remove()`\n"),
        Some("E0499") | Some("E0502") | Some("E0505") => output.push_str("help: use `lend`, `lend mut`, or `copy` to make ownership explicit\n"),
        Some("E0106") => output.push_str("help: use `lend(a)` when the lifetime relationship is ambiguous\n"),
        Some("E0373") => output.push_str("help: use `spawn(copy value)` when the task must own its data\nhelp: use `lend value` only when the task finishes before the borrowed value does\n"),
        Some("E0277") if diagnostic.message.contains("trait") => output.push_str("help: add an `extends Trait` bound to the generic parameter, or pass a type that implements the required trait\n"),
        Some("E0277") => output.push_str("help: spawned work must be `Send`; use an explicitly shared class backed by `Arc<tokio::sync::Mutex<...>>`\nhelp: keep non-Send values on the current task instead of passing them to `spawn`\n"),
        Some("E0515") => output.push_str("help: return an owned value, or pass the borrowed source into the function\n"),
        Some("E0506") => output.push_str("help: finish using the borrowed value before assigning to its source\n"),
        Some("E0597") if diagnostic.message.contains("drop") => output.push_str("help: declare the borrowed source before the object with `on drop`\n"),
        Some("E0597") | Some("E0521") => output.push_str("help: make the async operation finish before the borrowed value leaves scope, or use `copy`/an explicitly shared class\n"),
        Some("E0728") => output.push_str("help: mark the function `async` before using `await`\n"),
        Some("E0038") => output.push_str("help: make the interface dyn-compatible by removing `Self`-returning or generic methods, or use a concrete class\n"),
        Some("E0599") => output.push_str("help: add the method through a trait bound, or use a concrete type with that method\n"),
        Some("E0004") => output.push_str("help: add the missing enum pattern to `switch`, or include a deliberate wildcard case\n"),
        Some("E0046") => output.push_str("help: define every interface method in the class, using the same ORust signature\n"),
        Some("E0050") | Some("E0053") => output.push_str("help: compare the interface declaration with the implementation's parameter and return types\n"),
        Some("E0119") => output.push_str("help: remove the duplicate `implements` path or keep only one trait implementation\n"),
        Some("E0191") | Some("E0220") => output.push_str("help: add the required associated `type`, or use the exact associated type name\n"),
        Some("E0407") => output.push_str("help: remove the extra method or add it to the interface declaration\n"),
        Some("E0005") => output.push_str("help: use `var Pattern = value else { ... }` for a refutable binding\n"),
        Some("unreachable_patterns") => output.push_str("help: remove the unreachable pattern or reorder the match cases\n"),
        Some("E0432") | Some("E0433") => output.push_str("help: check the relative path and add an explicit `show Name` clause for the exported item\n"),
        Some("E0603") | Some("E0616") | Some("E0624") => output.push_str("help: use `export` for cross-file items, or keep access inside the defining module\n"),
        Some("E0252") | Some("E0254") | Some("E0255") => output.push_str("help: disambiguate the imports with `show`, `hide`, or an `as` namespace alias\n"),
        Some("E0446") => output.push_str("help: export the referenced type or make the public member private\n"),
        _ => {}
    }
    output
}

fn quoted_name(message: &str) -> Option<String> {
    let start = message.find('`')? + 1;
    let end = message[start..].find('`')? + start;
    Some(message[start..end].to_string())
}

fn source_location(source: &str, offset: usize) -> (usize, usize, &str) {
    let safe_offset = offset.min(source.len());
    let line = source[..safe_offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let line_start = source[..safe_offset]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let line_end = source[safe_offset..]
        .find('\n')
        .map(|index| safe_offset + index)
        .unwrap_or(source.len());
    (
        line,
        safe_offset - line_start + 1,
        &source[line_start..line_end],
    )
}

pub fn parse_json_diagnostics(
    output: &str,
    generated: &str,
    mappings: &[SpanMapping],
) -> Vec<Diagnostic> {
    output
        .lines()
        .filter_map(|line| {
            let value: Value = serde_json::from_str(line).ok()?;
            if value.get("reason")?.as_str()? != "compiler-message" {
                return None;
            }
            let message = value.get("message")?;
            let level = message.get("level")?.as_str()?.to_string();
            let text = message.get("message")?.as_str()?.to_string();
            let code = message
                .get("code")
                .and_then(|code| code.get("code"))
                .and_then(Value::as_str)
                .map(str::to_string);
            let source_span = message
                .get("spans")
                .and_then(Value::as_array)
                .and_then(|spans| spans.first())
                .and_then(|span| {
                    let line = span.get("line_start")?.as_u64()? as usize;
                    let column = span.get("column_start")?.as_u64()? as usize;
                    let offset = line_column_to_offset(generated, line, column)?;
                    mappings
                        .iter()
                        .find(|mapping| {
                            offset >= mapping.generated_start && offset < mapping.generated_end
                        })
                        .map(|mapping| mapping.source_span)
                });
            Some(Diagnostic {
                level,
                code,
                message: text,
                source_span,
            })
        })
        .collect()
}

fn line_column_to_offset(source: &str, line: usize, column: usize) -> Option<usize> {
    if line == 0 || column == 0 {
        return None;
    }
    let line_start = source
        .split_inclusive('\n')
        .take(line - 1)
        .map(str::len)
        .sum::<usize>();
    Some(line_start + column - 1)
}
