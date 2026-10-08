use orust_syntax::parse;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::PathBuf,
    process::{Command, ExitCode},
};

fn write_generated_project_kind(
    program: &orust_syntax::Program,
    library: bool,
) -> Result<(PathBuf, orust_emit::GeneratedRust), String> {
    let project = PathBuf::from("target/orust");
    fs::create_dir_all(project.join("src")).map_err(|error| error.to_string())?;
    let target = if library { "lib" } else { "main" };
    let target_section = if library {
        "[lib]\npath = \"src/lib.rs\"\n\n"
    } else {
        ""
    };
    fs::write(
        project.join("Cargo.toml"),
        format!("[workspace]\n\n[package]\nname = \"orust-generated\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n{target_section}[dependencies]\norust-runtime = {{ path = \"../../crates/orust-runtime\" }}\n"),
    )
    .map_err(|error| error.to_string())?;
    let stale_target = if library { "main.rs" } else { "lib.rs" };
    let stale_target = project.join("src").join(stale_target);
    if stale_target.exists() {
        fs::remove_file(stale_target).map_err(|error| error.to_string())?;
    }
    if PathBuf::from("Cargo.lock").exists() {
        fs::copy("Cargo.lock", project.join("Cargo.lock")).map_err(|error| error.to_string())?;
    };
    let generated = orust_emit::emit_with_spans(program);
    fs::write(
        project.join("src").join(format!("{target}.rs")),
        &generated.code,
    )
    .map_err(|error| error.to_string())?;
    Ok((project, generated))
}

fn module_name(path: &std::path::Path) -> String {
    let raw = path.file_stem().unwrap().to_string_lossy();
    let mut name = String::new();
    for (index, ch) in raw.chars().enumerate() {
        if ch.is_ascii_uppercase() && index != 0 {
            name.push('_');
        }
        name.push(ch.to_ascii_lowercase());
    }
    let keywords = [
        "self", "super", "crate", "fn", "mod", "type", "match", "use", "ref", "move", "struct",
        "enum", "trait", "const", "static", "async", "await", "loop", "impl", "where", "pub",
    ];
    if keywords.contains(&name.as_str()) {
        format!("r#{name}")
    } else {
        name
    }
}

fn normalize_path(path: &std::path::Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn resolve_or_path(source_file: &std::path::Path, relative_path: &str) -> PathBuf {
    normalize_path(&source_file.parent().unwrap().join(relative_path))
}

fn emitted_item_name(name: &str, rust_name: &Option<String>) -> String {
    rust_name.clone().unwrap_or_else(|| name.to_string())
}

fn exported_item_names(
    source_file: &std::path::Path,
    program: &orust_syntax::Program,
    all_programs: &HashMap<PathBuf, orust_syntax::Program>,
) -> Vec<(String, String)> {
    let mut visiting = HashSet::new();
    exported_item_names_inner(source_file, program, all_programs, &mut visiting)
}

fn exported_item_names_inner(
    source_file: &std::path::Path,
    program: &orust_syntax::Program,
    all_programs: &HashMap<PathBuf, orust_syntax::Program>,
    visiting: &mut HashSet<PathBuf>,
) -> Vec<(String, String)> {
    if !visiting.insert(source_file.to_path_buf()) {
        return Vec::new();
    }
    let mut names = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            orust_syntax::Item::Class(value) if value.exported => Some((
                value.name.clone(),
                emitted_item_name(&value.name, &value.rust_name),
            )),
            orust_syntax::Item::Function(value) if value.exported => Some((
                value.name.clone(),
                emitted_item_name(&value.name, &value.rust_name),
            )),
            orust_syntax::Item::Interface(value) if value.exported => Some((
                value.name.clone(),
                emitted_item_name(&value.name, &value.rust_name),
            )),
            orust_syntax::Item::Enum(value) if value.exported => Some((
                value.name.clone(),
                emitted_item_name(&value.name, &value.rust_name),
            )),
            orust_syntax::Item::TypeAlias(value) if value.exported => Some((
                value.name.clone(),
                emitted_item_name(&value.name, &value.rust_name),
            )),
            orust_syntax::Item::Newtype(value) if value.exported => Some((
                value.name.clone(),
                emitted_item_name(&value.name, &value.rust_name),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    for export in &program.exports {
        let target = resolve_or_path(source_file, &export.path);
        let target = if target.extension().is_none() {
            target.with_extension("or")
        } else {
            target
        };
        let Some(target_program) = all_programs.get(&target) else {
            continue;
        };
        for (source_name, emitted_name) in
            exported_item_names_inner(&target, target_program, all_programs, visiting)
        {
            let shown = export.show.is_empty()
                || export
                    .show
                    .iter()
                    .any(|name| name == &source_name || name == &emitted_name);
            let hidden = export
                .hide
                .iter()
                .any(|name| name == &source_name || name == &emitted_name);
            if shown
                && !hidden
                && !names
                    .iter()
                    .any(|item| item == &(source_name.clone(), emitted_name.clone()))
            {
                names.push((source_name, emitted_name));
            }
        }
    }
    visiting.remove(source_file);
    names
}

fn import_use_lines(
    source_file: &std::path::Path,
    source_root: &std::path::Path,
    program: &orust_syntax::Program,
    all_programs: &HashMap<PathBuf, orust_syntax::Program>,
) -> String {
    let mut out = String::new();
    for import in &program.imports {
        if import.path.starts_with("rust:") {
            let rust_path = import.path.trim_start_matches("rust:");
            if rust_path.starts_with("./") || rust_path.starts_with("../") {
                let target = source_file.parent().unwrap().join(rust_path);
                let _ = target.strip_prefix(source_root);
            } else if !rust_path.is_empty() {
                out.push_str(&format!("use {rust_path};\n"));
            }
            continue;
        }
        let target = resolve_or_path(source_file, &import.path);
        let target = if target.extension().is_none() {
            target.with_extension("or")
        } else {
            target
        };
        let Ok(relative) = target.strip_prefix(source_root) else {
            continue;
        };
        let mut path = Vec::new();
        for component in relative.components() {
            path.push(module_name(std::path::Path::new(component.as_os_str())));
        }
        let rust_path = path.join("::");
        if let Some(alias) = &import.alias {
            out.push_str(&format!("use crate::{rust_path} as {alias};\n"));
        } else if let Some(target_program) = all_programs.get(&target) {
            let names: Vec<String> = exported_item_names(&target, target_program, all_programs)
                .into_iter()
                .filter(|(source_name, emitted_name)| {
                    import.show.is_empty()
                        || import
                            .show
                            .iter()
                            .any(|shown| shown == source_name || shown == emitted_name)
                })
                .filter(|(source_name, emitted_name)| {
                    !import
                        .hide
                        .iter()
                        .any(|hidden| hidden == source_name || hidden == emitted_name)
                })
                .map(|(source_name, emitted_name)| {
                    if source_name == emitted_name {
                        emitted_name
                    } else {
                        format!("{emitted_name} as {source_name}")
                    }
                })
                .collect();
            if !names.is_empty() {
                out.push_str(&format!(
                    "use crate::{rust_path}::{{{}}};\n",
                    names.join(", ")
                ));
            }
        }
    }
    out
}

fn export_use_lines(
    source_file: &std::path::Path,
    source_root: &std::path::Path,
    program: &orust_syntax::Program,
    all_programs: &HashMap<PathBuf, orust_syntax::Program>,
) -> String {
    let mut out = String::new();
    for export in &program.exports {
        let target = resolve_or_path(source_file, &export.path);
        let target = if target.extension().is_none() {
            target.with_extension("or")
        } else {
            target
        };
        let Ok(relative) = target.strip_prefix(source_root) else {
            continue;
        };
        let rust_path = relative
            .components()
            .map(|component| module_name(std::path::Path::new(component.as_os_str())))
            .collect::<Vec<_>>()
            .join("::");
        let Some(target_program) = all_programs.get(&target) else {
            continue;
        };
        let names: Vec<String> = exported_item_names(&target, target_program, all_programs)
            .into_iter()
            .filter(|(source_name, emitted_name)| {
                export.show.is_empty()
                    || export
                        .show
                        .iter()
                        .any(|shown| shown == source_name || shown == emitted_name)
            })
            .filter(|(source_name, emitted_name)| {
                !export
                    .hide
                    .iter()
                    .any(|hidden| hidden == source_name || hidden == emitted_name)
            })
            .map(|(_, emitted_name)| emitted_name)
            .collect();
        if !names.is_empty() {
            out.push_str(&format!(
                "pub use crate::{rust_path}::{{{}}};\n",
                names.join(", ")
            ));
        }
    }
    out
}

fn collect_or_files(root: &std::path::Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.file_name().is_some_and(|n| n == "target") {
            continue;
        }
        if path.is_dir() {
            collect_or_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "or") {
            files.push(path);
        }
    }
    Ok(())
}

fn emit_module_contents(
    out: &mut String,
    directory: &std::path::Path,
    source_root: &std::path::Path,
    programs: &HashMap<PathBuf, orust_syntax::Program>,
    rust_files: &[PathBuf],
    entry: &std::path::Path,
) -> Result<(), String> {
    let mut source_files = programs
        .keys()
        .filter(|file| file.parent() == Some(directory))
        .filter(|file| file.as_path() != entry)
        .cloned()
        .collect::<Vec<_>>();
    source_files.sort();
    for file in source_files {
        let program = programs.get(&file).expect("source file was collected");
        let mut code = import_use_lines(&file, source_root, program, programs);
        code.push_str(&export_use_lines(&file, source_root, program, programs));
        code.push_str(&orust_emit::emit_without_records(program));
        if module_name(&file) == "index" {
            out.push_str(&code);
        } else {
            out.push_str(&format!(
                "pub mod {} {{\n{}\n}}\n",
                module_name(&file),
                code
            ));
        }
    }

    let mut direct_rust_files = rust_files
        .iter()
        .filter(|file| file.parent() == Some(directory))
        .collect::<Vec<_>>();
    direct_rust_files.sort();
    for file in direct_rust_files {
        let relative = file.strip_prefix(source_root).map_err(|e| e.to_string())?;
        out.push_str(&format!(
            "pub mod {} {{ include!(\"{}\"); }}\n",
            module_name(file),
            relative.display()
        ));
    }

    let mut child_directories = HashSet::new();
    for file in programs.keys().chain(rust_files.iter()) {
        let Ok(relative) = file.strip_prefix(directory) else {
            continue;
        };
        if relative.components().count() > 1 {
            if let Some(first) = relative.components().next() {
                child_directories.insert(first.as_os_str().to_string_lossy().to_string());
            }
        }
    }
    let mut child_directories = child_directories.into_iter().collect::<Vec<_>>();
    child_directories.sort();
    for child in child_directories {
        out.push_str(&format!(
            "pub mod {} {{\n",
            module_name(std::path::Path::new(&child))
        ));
        emit_module_contents(
            out,
            &directory.join(&child),
            source_root,
            programs,
            rust_files,
            entry,
        )?;
        out.push_str("}\n");
    }
    Ok(())
}

fn collect_rust_files(root: &std::path::Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.file_name().is_some_and(|n| n == "target") {
            continue;
        }
        if path.is_dir() {
            collect_rust_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn explain_code(code: &str) -> Option<&'static str> {
    match code.to_ascii_uppercase().as_str() {
        "OR0001" | "E0382" | "E0505" => Some(
            "OR0001 — moved value\n\nWhat happened:\nA value was moved, then used again. Rust transfers ownership instead of silently copying it.\n\nMinimal example:\n    var a = String(\"hello\");\n    var b = a;\n    print(a);\n\nFixes:\nUse `copy a` when cloning is intended, borrow with `lend a`, or restructure the code so the original is used before the move.\n\nRust equivalent:\nThe generated Rust contains an ordinary move and rustc reports E0382/E0505.\n",
        ),
        "OR0002" | "E0499" | "E0502" => Some(
            "OR0002 — conflicting borrows\n\nWhat happened:\nRust does not allow a mutable borrow while another borrow is still active, or two mutable borrows at once.\n\nFixes:\nKeep borrows short, finish the read before mutating, or use an explicit owned copy.\n\nRust equivalent:\nThe generated Rust borrow checker reports E0499/E0502.\n",
        ),
        "OR0003" | "E0432" | "E0433" | "E0603" => Some(
            "OR0003 — unavailable import or module\n\nWhat happened:\nThe requested module, item, or visibility boundary cannot be resolved.\n\nFixes:\nCheck the `.or` import path, the exported declaration, and the Cargo dependency.\n\nRust equivalent:\nrustc reports E0432, E0433, or E0603 for the generated `use` path.\n",
        ),
        "OR0005" | "E0106" | "E0515" | "E0597" | "E0506" => Some(
            "OR0005 — invalid borrowed lifetime\n\nWhat happened:\nA borrowed value does not live long enough, escapes its owner, or is changed while still borrowed.\n\nFixes:\nDeclare the owner earlier, finish the borrower sooner, or use an owned value. ORust infers lifetimes; `lend` makes the intended borrow explicit.\n\nRust equivalent:\nrustc reports E0106, E0515, E0597, or E0506 after lifetime inference.\n",
        ),
        "OR0007" | "E0277" | "E0308" => Some(
            "OR0007 — incompatible types or trait requirements\n\nWhat happened:\nAn expression's type does not match the required type, or a required Rust trait is missing.\n\nFixes:\nUse an explicit conversion, correct the declared type, or implement the required interface.\n\nRust equivalent:\nrustc reports E0277 or E0308 on the generated Rust.\n",
        ),
        "OR0008" | "ARITHMETIC_OVERFLOW" | "OVERFLOWING_LITERALS" => Some(
            "OR0008 — numeric overflow\n\nWhat happened:\nA calculation or literal cannot be represented by its numeric type. ORust keeps overflow checks enabled.\n\nFixes:\nUse a wider type, `checkedAdd`/`toChecked`, or an explicitly wrapping or saturating operation.\n\nRust equivalent:\nrustc or the runtime reports arithmetic_overflow or overflowing_literals.\n",
        ),
        "OR0010" => Some(
            "OR0010 — index out of range\n\nWhat happened:\nA list or string index is negative or is not smaller than the collection length.\n\nFixes:\nCheck the index or use a range that stays within bounds.\n\nRust equivalent:\nORust emits a checked `usize` conversion before indexing.\n",
        ),
        "OR0011" => Some(
            "OR0011 — shared value already borrowed\n\nWhat happened:\nA shared class was borrowed in a conflicting way at runtime.\n\nFixes:\nEnd the first borrow before starting another, or redesign the operation to avoid overlapping mutable access.\n\nRust equivalent:\nThe runtime's shared borrow guard reports the conflict.\n",
        ),
        "OR0012" => Some(
            "OR0012 — string slice boundary\n\nWhat happened:\nA string range ended in the middle of a UTF-8 character.\n\nFixes:\nUse character-aware `substring` for character positions, or choose byte offsets that are character boundaries.\n\nRust equivalent:\nRust reports that a byte index is not a char boundary.\n",
        ),
        "OR0013" => Some(
            "OR0013 — divide by zero\n\nWhat happened:\nAn integer was divided by zero at runtime.\n\nFixes:\nCheck the divisor before division or use a fallible operation.\n\nRust equivalent:\nThe generated Rust panics on division by zero.\n",
        ),
        "OR0014" => Some(
            "OR0014 — unwrap on null\n\nWhat happened:\nAn optional value was unwrapped while it was null.\n\nFixes:\nUse a nullable pattern, `??`, or check the value before accessing it.\n\nRust equivalent:\nThe generated Rust called `Option::unwrap()` on `None`.\n",
        ),
        "OR0015" => Some(
            "OR0015 — trait implementation mismatch\n\nWhat happened:\nA class is missing a trait method, has the wrong signature, or conflicts with another implementation.\n\nFixes:\nMatch the interface declaration exactly, add associated types, and remove duplicate implementations.\n\nRust equivalent:\nrustc reports E0046, E0050, E0053, E0119, E0191, E0220, or E0407.\n",
        ),
        "OR0016" => Some(
            "OR0016 — refutable binding\n\nWhat happened:\nA destructuring pattern may fail to match.\n\nFixes:\nUse `else { ... }` with the pattern so the failure path diverges.\n\nRust equivalent:\nrustc reports E0005 for a refutable pattern in a plain `let`.\n",
        ),
        "OR0017" => Some(
            "OR0017 — unreachable pattern\n\nWhat happened:\nAn earlier match pattern already handles every value this pattern could receive.\n\nFixes:\nRemove the redundant case or reorder the patterns.\n\nRust equivalent:\nrustc emits the `unreachable_patterns` lint.\n",
        ),
        _ => None,
    }
}

fn add_dependency(spec: &str) -> Result<PathBuf, String> {
    let mut root = env::current_dir().map_err(|error| error.to_string())?;
    let manifest: Option<PathBuf> = loop {
        let found = ["orust.toml", "Cargo.toml"]
            .iter()
            .map(|name| root.join(name))
            .find(|candidate| candidate.exists());
        if let Some(manifest) = found {
            break Some(manifest);
        }
        if !root.pop() {
            break None;
        }
    };
    let Some(manifest) = manifest else {
        return Err(
            "could not find orust.toml or Cargo.toml in this directory or its parents".into(),
        );
    };
    let (name, requirement) = spec.split_once('@').unwrap_or((spec, "*"));
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return Err("dependency names cannot be empty or contain whitespace".into());
    }
    let mut contents = fs::read_to_string(&manifest).map_err(|error| error.to_string())?;
    if contents
        .lines()
        .any(|line| line.trim_start().starts_with(&format!("{name} =")))
    {
        return Err(format!("dependency `{name}` is already declared"));
    }
    if !contents.lines().any(|line| line.trim() == "[dependencies]") {
        if !contents.ends_with('\n') {
            contents.push('\n');
        }
        contents.push_str("\n[dependencies]\n");
    } else if !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(&format!("{name} = \"{requirement}\"\n"));
    fs::write(&manifest, contents).map_err(|error| error.to_string())?;
    Ok(manifest)
}

fn recursive_notes(program: &orust_syntax::Program) -> Vec<String> {
    let classes = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            orust_syntax::Item::Class(class) => Some(class.name.clone()),
            _ => None,
        })
        .collect::<HashSet<_>>();
    program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            orust_syntax::Item::Class(class) => Some(
                class
                    .fields
                    .iter()
                    .filter_map(|field| {
                        let target = field.ty.trim_end_matches('?');
                        classes.contains(target).then(|| {
                            format!(
                                "note: `{}`.`{}` is stored on the heap, one allocation per {}",
                                class.name, field.name, class.name
                            )
                        })
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten()
        .collect()
}

fn string_length_index_warnings(program: &orust_syntax::Program) {
    use orust_syntax::{ClosureBody, Expr, Stmt};
    use std::collections::HashSet;

    fn walk_expr(
        expression: &Expr,
        strings: &HashSet<String>,
        lengths: &mut HashSet<String>,
        indexed: &mut HashSet<String>,
    ) {
        match expression {
            Expr::Member { object, name } | Expr::OptionalMember { object, name } => {
                if name == "length"
                    && matches!(object.as_ref(), Expr::Name(value) if strings.contains(value))
                {
                    if let Expr::Name(value) = object.as_ref() {
                        lengths.insert(value.clone());
                    }
                }
                walk_expr(object, strings, lengths, indexed);
            }
            Expr::Index { object, index } => {
                if let Expr::Name(value) = object.as_ref() {
                    if strings.contains(value) {
                        indexed.insert(value.clone());
                    }
                }
                walk_expr(object, strings, lengths, indexed);
                walk_expr(index, strings, lengths, indexed);
            }
            Expr::Slice {
                object, start, end, ..
            } => {
                if let Expr::Name(value) = object.as_ref() {
                    if strings.contains(value) {
                        indexed.insert(value.clone());
                    }
                }
                walk_expr(object, strings, lengths, indexed);
                if let Some(start) = start {
                    walk_expr(start, strings, lengths, indexed);
                }
                if let Some(end) = end {
                    walk_expr(end, strings, lengths, indexed);
                }
            }
            Expr::NewArgs { args, .. } | Expr::Call { args, .. } | Expr::List(args) => {
                for argument in args {
                    walk_expr(argument, strings, lengths, indexed);
                }
                if let Expr::Call { callee, .. } = expression {
                    walk_expr(callee, strings, lengths, indexed);
                }
            }
            Expr::Borrow { value, .. }
            | Expr::Copy(value)
            | Expr::Await(value)
            | Expr::Spawn(value) => walk_expr(value, strings, lengths, indexed),
            Expr::Coalesce { left, right } | Expr::Binary { left, right, .. } => {
                walk_expr(left, strings, lengths, indexed);
                walk_expr(right, strings, lengths, indexed);
            }
            Expr::NamedArg { value, .. } => walk_expr(value, strings, lengths, indexed),
            Expr::Closure { body, .. } => match body {
                ClosureBody::Expr(value) => walk_expr(value, strings, lengths, indexed),
                ClosureBody::Block(body) => walk_statements(body, strings),
            },
            Expr::Int(_)
            | Expr::Float(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Name(_)
            | Expr::New(_)
            | Expr::Null
            | Expr::This => {}
        }
    }

    fn walk_statements(statements: &[orust_syntax::Spanned<Stmt>], strings: &HashSet<String>) {
        let mut strings = strings.clone();
        let mut lengths = HashSet::new();
        let mut indexed = HashSet::new();
        for statement in statements {
            match &statement.node {
                Stmt::Var {
                    name,
                    initializer,
                    declared_type,
                } => {
                    if declared_type.as_deref() == Some("String")
                        || matches!(initializer, Expr::String(_))
                    {
                        strings.insert(name.clone());
                    }
                    walk_expr(initializer, &strings, &mut lengths, &mut indexed);
                }
                Stmt::PatternVar {
                    initializer,
                    else_body,
                    ..
                } => {
                    walk_expr(initializer, &strings, &mut lengths, &mut indexed);
                    walk_statements(else_body, &strings);
                }
                Stmt::Print(value)
                | Stmt::Expr(value)
                | Stmt::Throw(value)
                | Stmt::Return(Some(value)) => {
                    walk_expr(value, &strings, &mut lengths, &mut indexed)
                }
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    walk_expr(condition, &strings, &mut lengths, &mut indexed);
                    walk_statements(then_body, &strings);
                    walk_statements(else_body, &strings);
                }
                Stmt::While { condition, body }
                | Stmt::WhileCase {
                    value: condition,
                    body,
                    ..
                } => {
                    walk_expr(condition, &strings, &mut lengths, &mut indexed);
                    walk_statements(body, &strings);
                }
                Stmt::For {
                    initializer: _,
                    condition,
                    step,
                    body,
                } => {
                    // The `for` initializer has no source span wrapper; its
                    // expressions are still covered by the condition/step
                    // walks and nested body walk below.
                    if let Some(condition) = condition {
                        walk_expr(condition, &strings, &mut lengths, &mut indexed);
                    }
                    if let Some(step) = step {
                        walk_expr(step, &strings, &mut lengths, &mut indexed);
                    }
                    walk_statements(body, &strings);
                }
                Stmt::ForIn { iterable, body, .. }
                | Stmt::AwaitFor {
                    stream: iterable,
                    body,
                    ..
                } => {
                    walk_expr(iterable, &strings, &mut lengths, &mut indexed);
                    walk_statements(body, &strings);
                }
                Stmt::TryCatch {
                    body, catch_body, ..
                } => {
                    walk_statements(body, &strings);
                    walk_statements(catch_body, &strings);
                }
                Stmt::Switch { value, cases } => {
                    walk_expr(value, &strings, &mut lengths, &mut indexed);
                    for case in cases {
                        walk_statements(&case.body, &strings);
                    }
                }
                Stmt::IfCase {
                    value,
                    then_body,
                    else_body,
                    ..
                } => {
                    walk_expr(value, &strings, &mut lengths, &mut indexed);
                    walk_statements(then_body, &strings);
                    walk_statements(else_body, &strings);
                }
                Stmt::Return(None) | Stmt::Rust(_) => {}
            }
        }
        // This visitor is deliberately conservative: a warning is useful even
        // when the two operations occur in nested branches.
        for name in lengths.intersection(&indexed) {
            eprintln!(
                "warning: `{name}.length` counts UTF-8 bytes; indexing or slicing uses byte boundaries. Use `charCount()`/`substring()` for character positions."
            );
        }
    }

    for item in &program.items {
        match &item.node {
            orust_syntax::Item::Function(function) => {
                let strings = function
                    .params
                    .iter()
                    .filter(|parameter| parameter.ty == "String")
                    .map(|parameter| parameter.name.clone())
                    .collect::<HashSet<_>>();
                walk_statements(&function.body, &strings);
            }
            orust_syntax::Item::Class(class) => {
                for method in &class.methods {
                    let strings = method
                        .params
                        .iter()
                        .filter(|parameter| parameter.ty == "String")
                        .map(|parameter| parameter.name.clone())
                        .collect::<HashSet<_>>();
                    walk_statements(&method.body, &strings);
                }
            }
            _ => {}
        }
    }
}

fn cargo_dependency_lines(config: Option<&str>) -> String {
    let Some(config) = config else {
        return String::new();
    };
    let mut in_dependencies = false;
    let mut output = String::new();
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_dependencies = trimmed == "[dependencies]";
            continue;
        }
        if in_dependencies && !trimmed.is_empty() && !trimmed.starts_with('#') {
            output.push_str(trimmed);
            output.push('\n');
        }
    }
    output
}

fn cargo_section_lines(config: Option<&str>, section: &str) -> String {
    let Some(config) = config else {
        return String::new();
    };
    let mut in_section = false;
    let mut output = String::new();
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }
        if in_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
            output.push_str(trimmed);
            output.push('\n');
        }
    }
    output
}

fn cargo_target_dependency_sections(config: Option<&str>) -> String {
    let Some(config) = config else {
        return String::new();
    };
    let mut current_header = None;
    let mut output = String::new();
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            let is_target_dependency = trimmed.starts_with("[target.")
                && (trimmed.ends_with(".dependencies]") || trimmed.ends_with(".dev-dependencies]"));
            current_header = is_target_dependency.then(|| trimmed.to_string());
            if is_target_dependency {
                output.push_str(trimmed);
                output.push('\n');
            }
            continue;
        }
        if current_header.is_some() && !trimmed.is_empty() && !trimmed.starts_with('#') {
            output.push_str(trimmed);
            output.push('\n');
        }
    }
    output
}

fn cargo_patch_sections(config: Option<&str>) -> String {
    let Some(config) = config else {
        return String::new();
    };
    let mut current_header = None;
    let mut output = String::new();
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            let is_patch = trimmed.starts_with("[patch.") || trimmed == "[replace]";
            current_header = is_patch.then(|| trimmed.to_string());
            if is_patch {
                output.push_str(trimmed);
                output.push('\n');
            }
            continue;
        }
        if current_header.is_some() && !trimmed.is_empty() && !trimmed.starts_with('#') {
            output.push_str(trimmed);
            output.push('\n');
        }
    }
    output
}

fn cargo_package_name(config: Option<&str>) -> String {
    let Some(config) = config else {
        return "orust-generated".into();
    };
    let mut in_package = false;
    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package {
            if let Some(value) = trimmed.strip_prefix("name") {
                let Some(value) = value.trim().strip_prefix('=') else {
                    continue;
                };
                let name = value.trim().trim_matches('"');
                if !name.is_empty() {
                    return name.into();
                }
            }
        }
    }
    "orust-generated".into()
}

fn clear_generated_source_tree(source: &std::path::Path) -> Result<(), String> {
    if !source.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|error| error.to_string())?;
        } else {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn cargo_command(operation: &str, manifest: &std::path::Path, features: Option<&str>) -> Command {
    let mut command = Command::new("cargo");
    if operation == "run" {
        command.args(["run", "--quiet"]);
    } else if operation == "check" {
        command.args(["check", "--message-format=json"]);
    } else {
        command.arg(operation);
    }
    command.args(["--manifest-path", manifest.to_str().unwrap()]);
    if let Some(features) = features {
        command.args(["--features", features]);
    }
    command
}

fn is_cargo_project_manifest(path: &std::path::Path) -> bool {
    fs::read_to_string(path)
        .map(|text| text.lines().any(|line| line.trim() == "[package]"))
        .unwrap_or(false)
}

fn write_generated_project_tree(
    entry_path: &std::path::Path,
    library: bool,
) -> Result<(PathBuf, orust_emit::GeneratedRust), String> {
    let entry_path = normalize_path(entry_path);
    let mut source_root = entry_path.parent().unwrap().to_path_buf();
    let mut config = None;
    for candidate in entry_path.ancestors() {
        let orust_manifest = candidate.join("orust.toml");
        let cargo_manifest = candidate.join("Cargo.toml");
        let path = if orust_manifest.exists() {
            Some(orust_manifest)
        } else if cargo_manifest.exists() && is_cargo_project_manifest(&cargo_manifest) {
            Some(cargo_manifest)
        } else {
            None
        };
        if let Some(path) = path {
            source_root = candidate.to_path_buf();
            config = Some(path);
            break;
        }
    }
    let entry = config
        .as_ref()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("entry =")
                    .map(|v| v.trim().trim_matches('"').to_string())
            })
        })
        .map(|value| source_root.join(value))
        .unwrap_or_else(|| {
            let is_manifest = entry_path
                .file_name()
                .is_some_and(|name| name == "Cargo.toml" || name == "orust.toml");
            if entry_path.is_dir() || is_manifest {
                source_root.join("src/main.or")
            } else {
                entry_path.to_path_buf()
            }
        });
    let mut files = Vec::new();
    collect_or_files(&source_root, &mut files)?;
    let mut programs = HashMap::new();
    for file in files {
        programs.insert(
            file.clone(),
            parse(&fs::read_to_string(&file).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?,
        );
    }
    let project = PathBuf::from("target/orust");
    fs::create_dir_all(project.join("src")).map_err(|e| e.to_string())?;
    clear_generated_source_tree(&project.join("src"))?;
    let mut rust_files = Vec::new();
    collect_rust_files(&source_root, &mut rust_files)?;
    for file in &rust_files {
        let Ok(relative) = file.strip_prefix(&source_root) else {
            continue;
        };
        let destination = project.join("src").join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(file, destination).map_err(|e| e.to_string())?;
    }
    let runtime_path = std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            cwd.ancestors()
                .find(|path| path.join("crates/orust-runtime/Cargo.toml").exists())
                .map(|path| path.join("crates/orust-runtime"))
        })
        .unwrap_or_else(|| PathBuf::from("../../crates/orust-runtime"));
    let target_section = if library {
        "[lib]\npath = \"src/lib.rs\"\n\n"
    } else {
        ""
    };
    let config_text = config
        .as_ref()
        .and_then(|path| fs::read_to_string(path).ok());
    let package_name = cargo_package_name(config_text.as_deref());
    let features = cargo_section_lines(config_text.as_deref(), "[features]");
    let feature_section = if features.is_empty() {
        String::new()
    } else {
        format!("[features]\n{features}\n")
    };
    let dev_dependencies = cargo_section_lines(config_text.as_deref(), "[dev-dependencies]");
    let dev_dependency_section = if dev_dependencies.is_empty() {
        String::new()
    } else {
        format!("\n[dev-dependencies]\n{dev_dependencies}")
    };
    let build_dependencies = cargo_section_lines(config_text.as_deref(), "[build-dependencies]");
    let build_dependency_section = if build_dependencies.is_empty() {
        String::new()
    } else {
        format!("\n[build-dependencies]\n{build_dependencies}")
    };
    let workspace_dependencies =
        cargo_section_lines(config_text.as_deref(), "[workspace.dependencies]");
    let workspace_dependency_section = if workspace_dependencies.is_empty() {
        String::new()
    } else {
        format!("\n[workspace.dependencies]\n{workspace_dependencies}")
    };
    let target_dependencies = cargo_target_dependency_sections(config_text.as_deref());
    let target_dependency_section = if target_dependencies.is_empty() {
        String::new()
    } else {
        format!("\n{target_dependencies}")
    };
    let patch_sections = cargo_patch_sections(config_text.as_deref());
    let patch_section = if patch_sections.is_empty() {
        String::new()
    } else {
        format!("\n{patch_sections}")
    };
    let manifest = format!("[workspace]{workspace_dependency_section}\n\n[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n{target_section}{feature_section}[dependencies]\norust-runtime = {{ path = \"{}\" }}\n{}{dev_dependency_section}{build_dependency_section}{target_dependency_section}{patch_section}", runtime_path.display(), cargo_dependency_lines(config_text.as_deref()));
    fs::write(project.join("Cargo.toml"), manifest).map_err(|e| e.to_string())?;
    if let Some(workspace_root) = runtime_path.parent().and_then(|path| path.parent()) {
        let lock = workspace_root.join("Cargo.lock");
        if lock.exists() {
            fs::copy(lock, project.join("Cargo.lock")).map_err(|e| e.to_string())?;
        }
    }
    let mut root = String::new();
    let all_programs = programs.values().collect::<Vec<_>>();
    root.push_str(&orust_emit::emit_project_record_module(&all_programs));
    for file in &rust_files {
        if file.parent() == Some(source_root.as_path()) {
            root.push_str(&format!("pub mod {};\n", module_name(file)));
        }
    }
    for (file, program) in &programs {
        if file.parent() != Some(source_root.as_path()) || file == &entry {
            continue;
        }
        let mut code = import_use_lines(file, &source_root, program, &programs);
        code.push_str(&export_use_lines(file, &source_root, program, &programs));
        code.push_str(&orust_emit::emit_without_records(program));
        root.push_str(&format!("pub mod {} {{\n{}\n}}\n", module_name(file), code));
    }
    let mut top_dirs = HashSet::new();
    for file in programs.keys() {
        let Ok(relative) = file.strip_prefix(&source_root) else {
            continue;
        };
        if relative == entry.strip_prefix(&source_root).unwrap_or(relative) {
            continue;
        }
        if let Some(first) = relative.components().next() {
            if source_root.join(first.as_os_str()).is_dir() {
                top_dirs.insert(first.as_os_str().to_string_lossy().to_string());
            }
        }
    }
    let mut top_dirs = top_dirs.into_iter().collect::<Vec<_>>();
    top_dirs.sort();
    for dir in top_dirs {
        let dir_path = source_root.join(&dir);
        root.push_str(&format!(
            "pub mod {} {{\n",
            module_name(std::path::Path::new(&dir))
        ));
        emit_module_contents(
            &mut root,
            &dir_path,
            &source_root,
            &programs,
            &rust_files,
            &entry,
        )?;
        root.push_str("}\n");
    }
    let entry_program = programs
        .get(&entry)
        .ok_or_else(|| format!("entry file not found: {}", entry.display()))?;
    root.push_str(&import_use_lines(
        &entry,
        &source_root,
        entry_program,
        &programs,
    ));
    root.push_str(&export_use_lines(
        &entry,
        &source_root,
        entry_program,
        &programs,
    ));
    root.push_str(&orust_emit::emit_without_records(entry_program));
    fs::write(
        project
            .join("src")
            .join(if library { "lib.rs" } else { "main.rs" }),
        &root,
    )
    .map_err(|e| e.to_string())?;
    Ok((
        project,
        orust_emit::GeneratedRust {
            code: root,
            spans: Vec::new(),
        },
    ))
}

fn warn_never_awaited(program: &orust_syntax::Program) {
    let async_functions: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            orust_syntax::Item::Function(function) if function.is_async => {
                Some(function.name.clone())
            }
            _ => None,
        })
        .collect();

    for item in &program.items {
        let orust_syntax::Item::Function(function) = &item.node else {
            continue;
        };
        let mut pending = HashMap::new();
        scan_statements(&function.body, &async_functions, &mut pending);
        for (variable, function_name) in pending {
            eprintln!(
                "warning: `{function_name}()` creates a Future in `{variable}`, but it is never awaited or spawned; add `await` or `spawn`"
            );
        }
    }
}

fn scan_statements(
    statements: &[orust_syntax::Spanned<orust_syntax::Stmt>],
    async_functions: &HashSet<String>,
    pending: &mut HashMap<String, String>,
) {
    for statement in statements {
        match &statement.node {
            orust_syntax::Stmt::Var {
                name, initializer, ..
            } => {
                if let Some(function_name) = direct_async_call(initializer, async_functions) {
                    pending.insert(name.clone(), function_name.to_string());
                } else {
                    scan_expr(initializer, async_functions, pending, false);
                }
            }
            orust_syntax::Stmt::Expr(expression)
            | orust_syntax::Stmt::Print(expression)
            | orust_syntax::Stmt::Return(Some(expression))
            | orust_syntax::Stmt::Throw(expression) => {
                scan_expr(expression, async_functions, pending, false)
            }
            orust_syntax::Stmt::PatternVar {
                initializer,
                else_body,
                ..
            } => {
                scan_expr(initializer, async_functions, pending, false);
                scan_statements(else_body, async_functions, pending);
            }
            orust_syntax::Stmt::If {
                then_body,
                else_body,
                condition,
            } => {
                scan_expr(condition, async_functions, pending, false);
                scan_statements(then_body, async_functions, pending);
                scan_statements(else_body, async_functions, pending);
            }
            orust_syntax::Stmt::While { condition, body } => {
                scan_expr(condition, async_functions, pending, false);
                scan_statements(body, async_functions, pending);
            }
            orust_syntax::Stmt::WhileCase { value, body, .. } => {
                scan_expr(value, async_functions, pending, false);
                scan_statements(body, async_functions, pending);
            }
            orust_syntax::Stmt::For { body, .. } => scan_statements(body, async_functions, pending),
            orust_syntax::Stmt::ForIn { iterable, body, .. } => {
                scan_expr(iterable, async_functions, pending, false);
                scan_statements(body, async_functions, pending);
            }
            orust_syntax::Stmt::TryCatch {
                body, catch_body, ..
            } => {
                scan_statements(body, async_functions, pending);
                scan_statements(catch_body, async_functions, pending);
            }
            orust_syntax::Stmt::Switch { value, cases } => {
                scan_expr(value, async_functions, pending, false);
                for case in cases {
                    scan_statements(&case.body, async_functions, pending);
                }
            }
            orust_syntax::Stmt::IfCase {
                value,
                then_body,
                else_body,
                ..
            } => {
                scan_expr(value, async_functions, pending, false);
                scan_statements(then_body, async_functions, pending);
                scan_statements(else_body, async_functions, pending);
            }
            orust_syntax::Stmt::AwaitFor { stream, body, .. } => {
                scan_expr(stream, async_functions, pending, true);
                scan_statements(body, async_functions, pending);
            }
            orust_syntax::Stmt::Return(None) | orust_syntax::Stmt::Rust(_) => {}
        }
    }
}

fn direct_async_call<'a>(
    expression: &'a orust_syntax::Expr,
    async_functions: &HashSet<String>,
) -> Option<&'a str> {
    let orust_syntax::Expr::Call { callee, .. } = expression else {
        return None;
    };
    let orust_syntax::Expr::Name(name) = callee.as_ref() else {
        return None;
    };
    async_functions.contains(name).then_some(name.as_str())
}

fn scan_expr(
    expression: &orust_syntax::Expr,
    async_functions: &HashSet<String>,
    pending: &mut HashMap<String, String>,
    consumed: bool,
) {
    match expression {
        orust_syntax::Expr::Await(value) | orust_syntax::Expr::Spawn(value) => {
            if let orust_syntax::Expr::Name(name) = value.as_ref() {
                pending.remove(name);
            }
            scan_expr(value, async_functions, pending, true);
        }
        orust_syntax::Expr::Call { callee, args } => {
            if !consumed {
                if let Some(function_name) = direct_async_call(expression, async_functions) {
                    eprintln!(
                        "warning: `{function_name}()` creates a Future, but it is never awaited or spawned; add `await` or `spawn`"
                    );
                }
            }
            scan_expr(callee, async_functions, pending, consumed);
            for argument in args {
                scan_expr(argument, async_functions, pending, consumed);
            }
        }
        orust_syntax::Expr::NamedArg { value, .. } => {
            scan_expr(value, async_functions, pending, consumed)
        }
        orust_syntax::Expr::Member { object, .. }
        | orust_syntax::Expr::Borrow { value: object, .. }
        | orust_syntax::Expr::Copy(object) => scan_expr(object, async_functions, pending, consumed),
        orust_syntax::Expr::Index { object, index } => {
            scan_expr(object, async_functions, pending, consumed);
            scan_expr(index, async_functions, pending, consumed);
        }
        orust_syntax::Expr::Slice {
            object, start, end, ..
        } => {
            scan_expr(object, async_functions, pending, consumed);
            if let Some(start) = start {
                scan_expr(start, async_functions, pending, consumed);
            }
            if let Some(end) = end {
                scan_expr(end, async_functions, pending, consumed);
            }
        }
        orust_syntax::Expr::OptionalMember { object, .. } => {
            scan_expr(object, async_functions, pending, consumed)
        }
        orust_syntax::Expr::Coalesce { left, right }
        | orust_syntax::Expr::Binary { left, right, .. } => {
            scan_expr(left, async_functions, pending, consumed);
            scan_expr(right, async_functions, pending, consumed);
        }
        orust_syntax::Expr::List(values) => {
            for value in values {
                scan_expr(value, async_functions, pending, consumed);
            }
        }
        orust_syntax::Expr::Closure { body, .. } => match body {
            orust_syntax::ClosureBody::Expr(value) => {
                scan_expr(value, async_functions, pending, consumed)
            }
            orust_syntax::ClosureBody::Block(statements) => {
                scan_statements(statements, async_functions, pending)
            }
        },
        orust_syntax::Expr::Int(_)
        | orust_syntax::Expr::Float(_)
        | orust_syntax::Expr::Bool(_)
        | orust_syntax::Expr::String(_)
        | orust_syntax::Expr::Name(_)
        | orust_syntax::Expr::New(_)
        | orust_syntax::Expr::NewArgs { .. }
        | orust_syntax::Expr::Null
        | orust_syntax::Expr::This => {}
    }
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let mut path = args.next();
    let mut library = command == "build" && path.as_deref() == Some("--lib");
    if library {
        path = args.next();
    }
    let mut verbose = path.as_deref() == Some("--verbose");
    if verbose {
        path = args.next();
        if command == "build" && path.as_deref() == Some("--lib") {
            library = true;
            path = args.next();
        }
    }
    let mut cargo_features = None;
    if matches!(
        command.as_str(),
        "check" | "build" | "run" | "test" | "emit"
    ) {
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--features" => {
                    cargo_features = args.next();
                    if cargo_features.is_none() {
                        eprintln!("error: --features requires a comma-separated feature list");
                        return ExitCode::from(2);
                    }
                }
                "--verbose" => verbose = true,
                other => {
                    eprintln!("error: unknown option `{other}`");
                    return ExitCode::from(2);
                }
            }
        }
    }
    if command == "new" {
        let Some(project) = path else {
            eprintln!("usage: orust new <project-directory> [--lib | --workspace]");
            return ExitCode::from(2);
        };
        let mut library = false;
        let mut workspace = false;
        for flag in args {
            match flag.as_str() {
                "--lib" => library = true,
                "--workspace" => workspace = true,
                other => {
                    eprintln!("error: unknown `orust new` option `{other}`");
                    return ExitCode::from(2);
                }
            }
        }
        let directory = PathBuf::from(&project);
        if directory.exists() {
            eprintln!("error: project directory already exists: {project}");
            return ExitCode::FAILURE;
        }
        let source_name = if library { "lib.or" } else { "main.or" };
        let source = if library {
            "export void hello() {\n  print(\"Hello from ORust\");\n}\n"
        } else {
            "void main() {\n  print(\"Hello from ORust\");\n}\n"
        };
        let manifest = if workspace {
            "[workspace]\nmembers = [\".\"]\n\n[package]\nname = \"orust-project\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
        } else {
            "[package]\nname = \"orust-project\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
        };
        let result = fs::create_dir_all(directory.join("src"))
            .and_then(|_| fs::create_dir_all(directory.join("tests")))
            .and_then(|_| fs::write(directory.join("orust.toml"), manifest))
            .and_then(|_| fs::write(directory.join("src").join(source_name), source))
            .and_then(|_| {
                fs::write(
                    directory.join("tests").join("smoke.or"),
                    "test smoke() {\n  expect(true);\n}\n",
                )
            })
            .and_then(|_| {
                fs::write(
                    directory.join(".gitignore"),
                    "target/\n.orust/\n",
                )
            })
            .and_then(|_| {
                fs::write(
                    directory.join("README.md"),
                    if library {
                        "# ORust project\n\nRun `orust build --lib src/lib.or` to compile the library.\n"
                    } else {
                        "# ORust project\n\nRun `orust run src/main.or` to run it.\n"
                    },
                )
            });
        if let Err(error) = result {
            eprintln!("error: could not create project: {error}");
            return ExitCode::FAILURE;
        }
        println!("created ORust project: {project}");
        return ExitCode::SUCCESS;
    }
    if command == "explain" {
        let Some(code) = path else {
            eprintln!("usage: orust explain <OR-code-or-rustc-code>");
            return ExitCode::from(2);
        };
        match explain_code(&code) {
            Some(explanation) => {
                print!("{explanation}");
                return ExitCode::SUCCESS;
            }
            None => {
                eprintln!("error: no explanation is registered for `{code}`");
                eprintln!("known codes are listed in docs/error-codes.md");
                return ExitCode::from(2);
            }
        }
    }
    if command == "add" {
        let Some(spec) = path else {
            eprintln!("usage: orust add <crate[@version]>");
            return ExitCode::from(2);
        };
        match add_dependency(&spec) {
            Ok(manifest) => {
                println!("added `{spec}` to {}", manifest.display());
                return ExitCode::SUCCESS;
            }
            Err(error) => {
                eprintln!("error: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    if !matches!(
        command.as_str(),
        "emit" | "check" | "build" | "run" | "test"
    ) || path.is_none()
    {
        eprintln!("usage: orust <check|emit|build|run|test> <file.or> [--features <list>]\n       orust new <project-directory> [--lib | --workspace]\n       orust add <crate[@version]>\n       orust explain <OR-code-or-rustc-code>");
        return ExitCode::from(2);
    }
    let path = path.unwrap();
    let is_project = PathBuf::from(&path).ancestors().any(|ancestor| {
        ancestor.join("orust.toml").exists()
            || (ancestor.join("Cargo.toml").exists()
                && is_cargo_project_manifest(&ancestor.join("Cargo.toml")))
    });
    if is_project {
        match write_generated_project_tree(std::path::Path::new(&path), library) {
            Ok((_project, generated)) if command == "emit" => {
                print!("{}", generated.code);
                return ExitCode::SUCCESS;
            }
            Ok((project, _generated)) => {
                let manifest = project.join("Cargo.toml");
                let result = cargo_command(&command, &manifest, cargo_features.as_deref()).output();
                return match result {
                    Ok(output) if output.status.success() => {
                        if verbose && command == "check" {
                            if let Ok(source) = fs::read_to_string(&path) {
                                if let Ok(program) = parse(&source) {
                                    for note in recursive_notes(&program) {
                                        eprintln!("{note}");
                                    }
                                }
                            }
                        }
                        if command == "check" {
                            if let Ok(source) = fs::read_to_string(&path) {
                                if let Ok(program) = parse(&source) {
                                    string_length_index_warnings(&program);
                                }
                            }
                        }
                        if matches!(command.as_str(), "check" | "build" | "test") {
                            println!("ok: {path}");
                        } else {
                            print!("{}", String::from_utf8_lossy(&output.stdout));
                        }
                        ExitCode::SUCCESS
                    }
                    Ok(output) => {
                        eprint!("{}", String::from_utf8_lossy(&output.stderr));
                        ExitCode::FAILURE
                    }
                    Err(error) => {
                        eprintln!("error: could not invoke cargo: {error}");
                        ExitCode::FAILURE
                    }
                };
            }
            Err(error) => {
                eprintln!("error: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    match fs::read_to_string(&path)
        .map_err(|e| e.to_string())
        .and_then(|s| parse(&s).map_err(|e| e.to_string()))
    {
        Ok(program) if command == "emit" => {
            print!("{}", orust_emit::emit(&program));
            ExitCode::SUCCESS
        }
        Ok(program) => match write_generated_project_kind(&program, library) {
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
            Ok((project, generated)) => {
                warn_never_awaited(&program);
                let manifest = project.join("Cargo.toml");
                let manifest = manifest.to_str().unwrap();
                let result = cargo_command(
                    &command,
                    std::path::Path::new(manifest),
                    cargo_features.as_deref(),
                )
                .output();
                match result {
                    Ok(output) if output.status.success() => {
                        if verbose && command == "check" {
                            for note in recursive_notes(&program) {
                                eprintln!("{note}");
                            }
                        }
                        if command == "check" {
                            string_length_index_warnings(&program);
                        }
                        if matches!(command.as_str(), "check" | "build" | "test") {
                            println!("ok: {path}");
                        } else {
                            print!("{}", String::from_utf8_lossy(&output.stdout));
                        }
                        ExitCode::SUCCESS
                    }
                    Ok(output) => {
                        let source = fs::read_to_string(&path).unwrap_or_default();
                        for diagnostic in orust_diag::parse_json_diagnostics(
                            &String::from_utf8_lossy(&output.stdout),
                            &generated.code,
                            &generated.spans,
                        ) {
                            eprint!("{}", orust_diag::translate(&diagnostic, &source, &path));
                        }
                        eprint!("{}", String::from_utf8_lossy(&output.stderr));
                        ExitCode::FAILURE
                    }
                    Err(error) => {
                        eprintln!("error: could not invoke cargo: {error}");
                        ExitCode::FAILURE
                    }
                }
            }
        },
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cargo_package_name, cargo_patch_sections, cargo_section_lines,
        cargo_target_dependency_sections, explain_code, exported_item_names,
    };
    use std::{collections::HashMap, path::PathBuf};

    #[test]
    fn explains_orust_and_rust_codes() {
        assert!(explain_code("OR0005").is_some());
        assert!(explain_code("E0382").is_some());
        for code in ["OR0012", "OR0013", "OR0014", "OR0015", "OR0016", "OR0017"] {
            assert!(
                explain_code(code).is_some(),
                "missing explanation for {code}"
            );
        }
        assert!(explain_code("unknown").is_none());
    }

    #[test]
    fn preserves_the_source_cargo_package_name_for_generated_projects() {
        assert_eq!(
            cargo_package_name(Some(
                "[package]\nname = \"my-orust-crate\"\nversion = \"1.0\"\n\n[dependencies]\n",
            )),
            "my-orust-crate"
        );
        assert_eq!(
            cargo_package_name(Some(
                "[package]\nname = \"orust-library\"\nversion = \"0.1.0\"\nentry = \"src/lib.or\"",
            )),
            "orust-library"
        );
        assert_eq!(cargo_package_name(None), "orust-generated");
    }

    #[test]
    fn preserves_features_and_dev_dependencies_for_generated_cargo_projects() {
        let manifest = "[package]\nname = \"demo\"\n\n[features]\ndefault = [\"json\"]\njson = [\"serde_json\"]\n\n[dev-dependencies]\npretty_assertions = \"1\"\n";
        assert_eq!(
            cargo_section_lines(Some(manifest), "[features]"),
            "default = [\"json\"]\njson = [\"serde_json\"]\n"
        );
        assert_eq!(
            cargo_section_lines(Some(manifest), "[dev-dependencies]"),
            "pretty_assertions = \"1\"\n"
        );
    }

    #[test]
    fn preserves_target_specific_dependency_sections() {
        let manifest = "[package]\nname = \"demo\"\n\n[target.'cfg(unix)'.dependencies]\nlibc = \"0.2\"\n\n[target.'cfg(windows)'.dev-dependencies]\nwinapi = \"0.3\"\n\n[profile.release]\nopt-level = 3\n";
        assert_eq!(
            cargo_target_dependency_sections(Some(manifest)),
            "[target.'cfg(unix)'.dependencies]\nlibc = \"0.2\"\n[target.'cfg(windows)'.dev-dependencies]\nwinapi = \"0.3\"\n"
        );
    }

    #[test]
    fn preserves_workspace_and_build_dependency_sections() {
        let manifest =
            "[workspace.dependencies]\nserde = \"1\"\n\n[build-dependencies]\ncc = \"1\"\n";
        assert_eq!(
            cargo_section_lines(Some(manifest), "[workspace.dependencies]"),
            "serde = \"1\"\n"
        );
        assert_eq!(
            cargo_section_lines(Some(manifest), "[build-dependencies]"),
            "cc = \"1\"\n"
        );
    }

    #[test]
    fn preserves_cargo_patch_and_replace_sections() {
        let manifest = "[patch.crates-io]\nserde = { path = \"vendor/serde\" }\n\n[replace]\n\"old:1.0.0\" = { git = \"https://example.invalid/old\" }\n\n[profile.release]\nopt-level = 3\n";
        assert_eq!(
            cargo_patch_sections(Some(manifest)),
            "[patch.crates-io]\nserde = { path = \"vendor/serde\" }\n[replace]\n\"old:1.0.0\" = { git = \"https://example.invalid/old\" }\n"
        );
    }

    #[test]
    fn cyclic_reexports_do_not_recurse_forever() {
        let a = PathBuf::from("a.or");
        let b = PathBuf::from("b.or");
        let mut programs = HashMap::new();
        programs.insert(a.clone(), orust_syntax::parse("export 'b.or';").unwrap());
        programs.insert(b, orust_syntax::parse("export 'a.or';").unwrap());
        assert!(exported_item_names(&a, programs.get(&a).unwrap(), &programs).is_empty());
    }
}
