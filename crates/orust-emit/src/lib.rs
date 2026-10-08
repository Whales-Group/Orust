use orust_syntax::{ClosureBody, Expr, Function, Item, Param, Program, Span, Stmt};
use std::collections::{HashMap, HashSet};

fn declared_name(name: &str, rust_name: &Option<String>) -> String {
    rust_name.clone().unwrap_or_else(|| name.to_string())
}

fn record_fields_from_type(ty: &str) -> Option<Vec<(String, String)>> {
    let body = ty.strip_prefix("record{")?.strip_suffix('}')?;
    if body.is_empty() {
        return Some(Vec::new());
    }
    Some(
        body.split(',')
            .filter_map(|field| field.split_once(':'))
            .map(|(name, ty)| (name.to_string(), ty.to_string()))
            .collect(),
    )
}

fn record_name(fields: &[(String, String)]) -> String {
    let mut name = String::from("Record");
    for (field, ty) in fields {
        for value in [field.as_str(), ty.as_str()] {
            name.push('_');
            for ch in value.chars() {
                name.push(if ch.is_ascii_alphanumeric() { ch } else { '_' });
            }
        }
    }
    name
}

fn collect_record_type(ty: &str, records: &mut HashMap<String, Vec<(String, String)>>) {
    if let Some(fields) = record_fields_from_type(ty) {
        records
            .entry(record_name(&fields))
            .or_insert(fields.clone());
        for (_, field_ty) in fields {
            collect_record_type(&field_ty, records);
        }
    }
    if let Some(inner) = ty.strip_prefix('&') {
        collect_record_type(inner.strip_prefix("mut ").unwrap_or(inner), records);
    }
    if let Some(inner) = ty.strip_suffix('?') {
        collect_record_type(inner, records);
    }
}

fn collect_record_expr(e: &Expr, records: &mut HashMap<String, Vec<(String, String)>>) {
    match e {
        Expr::NewArgs { name, args } if name == "__orust_record__" => {
            let fields = args
                .iter()
                .filter_map(|arg| match arg {
                    Expr::NamedArg { name, value } => {
                        Some((name.clone(), inferred_expr_type(value)))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            records.entry(record_name(&fields)).or_insert(fields);
            for arg in args {
                collect_record_expr(arg, records);
            }
        }
        Expr::Call { callee, args } => {
            collect_record_expr(callee, records);
            for arg in args {
                collect_record_expr(arg, records);
            }
        }
        Expr::NewArgs { args, .. } | Expr::List(args) => {
            for arg in args {
                collect_record_expr(arg, records);
            }
        }
        Expr::Index { object, index } => {
            collect_record_expr(object, records);
            collect_record_expr(index, records);
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            collect_record_expr(object, records);
            if let Some(start) = start {
                collect_record_expr(start, records);
            }
            if let Some(end) = end {
                collect_record_expr(end, records);
            }
        }
        Expr::OptionalMember { object, .. }
        | Expr::Member { object, .. }
        | Expr::Borrow { value: object, .. }
        | Expr::Copy(object)
        | Expr::Await(object)
        | Expr::Spawn(object) => collect_record_expr(object, records),
        Expr::Coalesce { left, right } | Expr::Binary { left, right, .. } => {
            collect_record_expr(left, records);
            collect_record_expr(right, records);
        }
        Expr::NamedArg { value, .. } => collect_record_expr(value, records),
        Expr::Closure { body, .. } => match body {
            ClosureBody::Expr(value) => collect_record_expr(value, records),
            ClosureBody::Block(body) => collect_record_statements(body, records),
        },
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Null
        | Expr::This => {}
        Expr::Name(_) => {}
        Expr::New(_) => {}
    }
}

fn inferred_expr_type(e: &Expr) -> String {
    match e {
        Expr::Int(_) => "int",
        Expr::Float(_) => "double",
        Expr::Bool(_) => "bool",
        Expr::String(_) => "String",
        Expr::NewArgs { name, args } if name == "__orust_record__" => {
            let fields = args
                .iter()
                .filter_map(|arg| match arg {
                    Expr::NamedArg { name, value } => {
                        Some(format!("{name}:{}", inferred_expr_type(value)))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            return format!("record{{{}}}", fields.join(","));
        }
        Expr::List(_) => "List",
        Expr::Null => "dynamic",
        _ => "dynamic",
    }
    .into()
}

fn collect_record_statements(
    statements: &[orust_syntax::Spanned<Stmt>],
    records: &mut HashMap<String, Vec<(String, String)>>,
) {
    for statement in statements {
        match &statement.node {
            Stmt::Var {
                initializer,
                declared_type,
                ..
            } => {
                if let Some(ty) = declared_type {
                    collect_record_type(ty, records);
                }
                collect_record_expr(initializer, records);
            }
            Stmt::PatternVar {
                initializer,
                else_body,
                ..
            } => {
                collect_record_expr(initializer, records);
                collect_record_statements(else_body, records);
            }
            Stmt::Print(value)
            | Stmt::Return(Some(value))
            | Stmt::Expr(value)
            | Stmt::Throw(value) => collect_record_expr(value, records),
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                collect_record_expr(condition, records);
                collect_record_statements(then_body, records);
                collect_record_statements(else_body, records);
            }
            Stmt::While { condition, body }
            | Stmt::WhileCase {
                value: condition,
                body,
                ..
            } => {
                collect_record_expr(condition, records);
                collect_record_statements(body, records);
            }
            Stmt::For {
                initializer,
                condition,
                step,
                body,
            } => {
                if let Some(initializer) = initializer {
                    if let Stmt::Var {
                        initializer,
                        declared_type,
                        ..
                    } = initializer.as_ref()
                    {
                        if let Some(ty) = declared_type {
                            collect_record_type(ty, records);
                        }
                        collect_record_expr(initializer, records);
                    }
                }
                if let Some(condition) = condition {
                    collect_record_expr(condition, records);
                }
                if let Some(step) = step {
                    collect_record_expr(step, records);
                }
                collect_record_statements(body, records);
            }
            Stmt::ForIn { iterable, body, .. }
            | Stmt::AwaitFor {
                stream: iterable,
                body,
                ..
            } => {
                collect_record_expr(iterable, records);
                collect_record_statements(body, records);
            }
            Stmt::TryCatch {
                body, catch_body, ..
            } => {
                collect_record_statements(body, records);
                collect_record_statements(catch_body, records);
            }
            Stmt::Switch { value, cases } => {
                collect_record_expr(value, records);
                for case in cases {
                    collect_record_statements(&case.body, records);
                }
            }
            Stmt::IfCase {
                value,
                then_body,
                else_body,
                ..
            } => {
                collect_record_expr(value, records);
                collect_record_statements(then_body, records);
                collect_record_statements(else_body, records);
            }
            Stmt::Return(None) | Stmt::Rust(_) => {}
        }
    }
}

fn class_has_borrowed_fields(class: &orust_syntax::Class) -> bool {
    class.fields.iter().any(|field| field.ty.starts_with('&'))
}

fn class_generic_decl(class: &orust_syntax::Class) -> String {
    let base = generic_decl(&class.generics, &class.bounds);
    if !class_has_borrowed_fields(class) {
        return base;
    }
    if base.is_empty() {
        "<'a>".into()
    } else {
        format!("<'a, {}", &base[1..])
    }
}

fn class_generic_use(class: &orust_syntax::Class) -> String {
    let base = generic_use(&class.generics);
    if !class_has_borrowed_fields(class) {
        return base;
    }
    if base.is_empty() {
        "<'a>".into()
    } else {
        format!("<'a, {}", &base[1..])
    }
}

fn class_field_type(ty: &str, interfaces: &HashSet<String>, with_spawn: bool) -> String {
    if let Some(inner) = ty.strip_prefix('&') {
        let mutable = inner.starts_with("mut ");
        let inner = inner.strip_prefix("mut ").unwrap_or(inner);
        if inner == "String" {
            return format!("&'a {}str", if mutable { "mut " } else { "" });
        }
        if let Some(element) = inner
            .strip_prefix("List<")
            .and_then(|value| value.strip_suffix('>'))
        {
            return format!(
                "&'a {}[{}]",
                if mutable { "mut " } else { "" },
                rust_type_for(element, interfaces, with_spawn)
            );
        }
        return format!(
            "&'a {}{}",
            if mutable { "mut " } else { "" },
            rust_type_for(inner, interfaces, with_spawn)
        );
    }
    rust_type_for(ty, interfaces, with_spawn)
}

/// Break recursive value-type cycles at the syntax boundary. Rust requires an
/// indirection for recursive structs/enums; ORust keeps that implementation
/// detail invisible and makes the choice deterministic so generated code is
/// stable across runs.
fn box_recursive_fields(program: &mut Program) {
    let class_names: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Class(class) => Some(class.name.clone()),
            _ => None,
        })
        .collect();
    let classes: Vec<(String, Vec<(String, String)>)> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Class(class) => Some((
                class.name.clone(),
                class
                    .fields
                    .iter()
                    .filter_map(|field| {
                        let base = direct_class_type(&field.ty)?;
                        class_names
                            .contains(base)
                            .then(|| (field.name.clone(), base.into()))
                    })
                    .collect(),
            )),
            _ => None,
        })
        .collect();

    // Compute strongly connected components with a small deterministic DFS.
    // The graph is intentionally limited to direct value edges: collections,
    // shared values, and existing Box edges are already indirect in Rust.
    let mut components = Vec::new();
    let mut visited = HashSet::new();
    for (name, _) in &classes {
        if visited.contains(name) {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![name.clone()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            component.push(current.clone());
            for (_, target) in classes
                .iter()
                .find(|(candidate, _)| candidate == &current)
                .map(|(_, fields)| fields.as_slice())
                .unwrap_or(&[])
            {
                stack.push(target.clone());
            }
        }
        // The reverse reachability check turns the DFS tree into an SCC for
        // the small class graph without introducing a graph dependency.
        let strongly_connected: Vec<String> = component
            .iter()
            .filter(|candidate| {
                component
                    .iter()
                    .all(|other| other == *candidate || reaches(&classes, candidate, other))
            })
            .cloned()
            .collect();
        if strongly_connected.len() > 1
            || strongly_connected.iter().any(|member| {
                classes
                    .iter()
                    .find(|(name, _)| name == member)
                    .is_some_and(|(_, fields)| fields.iter().any(|(_, target)| target == member))
            })
        {
            components.push(strongly_connected);
        }
    }

    let mut boxed = HashSet::new();
    for component in components {
        let members: HashSet<&str> = component.iter().map(String::as_str).collect();
        let mut candidates = Vec::new();
        for (class_name, fields) in &classes {
            if !members.contains(class_name.as_str()) {
                continue;
            }
            for (field_name, target) in fields {
                if members.contains(target.as_str()) {
                    let field_ty = program
                        .items
                        .iter()
                        .find_map(|item| match &item.node {
                            Item::Class(class) if class.name == *class_name => class
                                .fields
                                .iter()
                                .find(|field| field.name == *field_name)
                                .map(|field| field.ty.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    candidates.push((
                        field_ty.ends_with('?'),
                        class_name.clone(),
                        field_name.clone(),
                    ));
                }
            }
        }
        candidates.sort_by(|left, right| left.1.cmp(&right.1).then(left.2.cmp(&right.2)));
        // A recursive struct may have several direct recursive fields (for
        // example a binary tree). Boxing only one still leaves the other
        // infinite-size field unbounded, so all direct recursive fields in a
        // recursive component are represented indirectly.
        for candidate in candidates {
            boxed.insert((candidate.1, candidate.2));
        }
    }

    for item in &mut program.items {
        if let Item::Class(class) = &mut item.node {
            for field in &mut class.fields {
                if boxed.contains(&(class.name.clone(), field.name.clone())) {
                    field.ty = box_type(&field.ty);
                }
            }
        }
    }
    box_recursive_enum_fields(program);
    box_mixed_recursive_fields(program);
}

fn box_mixed_recursive_fields(program: &mut Program) {
    let names: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Class(class) => Some(class.name.clone()),
            Item::Enum(enum_decl) => Some(enum_decl.name.clone()),
            _ => None,
        })
        .collect();
    let graph: Vec<(String, Vec<(String, String)>)> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Class(class) => Some((
                class.name.clone(),
                class
                    .fields
                    .iter()
                    .filter_map(|field| {
                        let target = direct_class_type(&field.ty)?;
                        names
                            .contains(target)
                            .then(|| (field.name.clone(), target.to_string()))
                    })
                    .collect(),
            )),
            Item::Enum(enum_decl) => Some((
                enum_decl.name.clone(),
                enum_decl
                    .variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter().enumerate())
                    .filter_map(|(index, field)| {
                        let target = direct_class_type(field)?;
                        names
                            .contains(target)
                            .then(|| (index.to_string(), target.to_string()))
                    })
                    .collect(),
            )),
            _ => None,
        })
        .collect();
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    for (name, _) in &graph {
        if visited.contains(name) {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![name.clone()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            component.push(current.clone());
            if let Some((_, fields)) = graph.iter().find(|(candidate, _)| candidate == &current) {
                stack.extend(fields.iter().map(|(_, target)| target.clone()));
            }
        }
        let strongly_connected: Vec<String> = component
            .iter()
            .filter(|candidate| {
                component
                    .iter()
                    .all(|other| other == *candidate || reaches(&graph, candidate, other))
            })
            .cloned()
            .collect();
        if strongly_connected.len() > 1
            || strongly_connected.iter().any(|member| {
                graph
                    .iter()
                    .find(|(name, _)| name == member)
                    .is_some_and(|(_, fields)| fields.iter().any(|(_, target)| target == member))
            })
        {
            components.push(strongly_connected);
        }
    }
    for component in components {
        let members: HashSet<&str> = component.iter().map(String::as_str).collect();
        for item in &mut program.items {
            match &mut item.node {
                Item::Class(class) if members.contains(class.name.as_str()) => {
                    for field in &mut class.fields {
                        if let Some(target) = direct_class_type(&field.ty) {
                            if members.contains(target) {
                                field.ty = box_type(&field.ty);
                            }
                        }
                    }
                }
                Item::Enum(enum_decl) if members.contains(enum_decl.name.as_str()) => {
                    for variant in &mut enum_decl.variants {
                        for field in &mut variant.fields {
                            if let Some(target) = direct_class_type(field) {
                                if members.contains(target) {
                                    *field = box_type(field);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn box_recursive_enum_fields(program: &mut Program) {
    let enum_names: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Enum(enum_decl) => Some(enum_decl.name.clone()),
            _ => None,
        })
        .collect();
    let graph: Vec<(String, Vec<(String, String)>)> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Enum(enum_decl) => Some((
                enum_decl.name.clone(),
                enum_decl
                    .variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter())
                    .filter_map(|field| {
                        let target = direct_class_type(field)?;
                        enum_names
                            .contains(target)
                            .then(|| (String::new(), target.to_string()))
                    })
                    .collect(),
            )),
            _ => None,
        })
        .collect();
    let mut visited = HashSet::new();
    let mut components = Vec::new();
    for (name, _) in &graph {
        if visited.contains(name) {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![name.clone()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            component.push(current.clone());
            if let Some((_, fields)) = graph.iter().find(|(candidate, _)| candidate == &current) {
                stack.extend(fields.iter().map(|(_, target)| target.clone()));
            }
        }
        let strongly_connected: Vec<String> = component
            .iter()
            .filter(|candidate| {
                component
                    .iter()
                    .all(|other| other == *candidate || reaches(&graph, candidate, other))
            })
            .cloned()
            .collect();
        if strongly_connected.len() > 1
            || strongly_connected.iter().any(|member| {
                graph
                    .iter()
                    .find(|(name, _)| name == member)
                    .is_some_and(|(_, fields)| fields.iter().any(|(_, target)| target == member))
            })
        {
            components.push(strongly_connected);
        }
    }

    for component in components {
        let members: HashSet<&str> = component.iter().map(String::as_str).collect();
        let mut candidates = Vec::new();
        for item in &program.items {
            let Item::Enum(enum_decl) = &item.node else {
                continue;
            };
            if !members.contains(enum_decl.name.as_str()) {
                continue;
            }
            for (variant_index, variant) in enum_decl.variants.iter().enumerate() {
                for (field_index, field) in variant.fields.iter().enumerate() {
                    let Some(target) = direct_class_type(field) else {
                        continue;
                    };
                    if members.contains(target) {
                        candidates.push((
                            field.ends_with('?'),
                            enum_decl.name.clone(),
                            variant_index,
                            field_index,
                        ));
                    }
                }
            }
        }
        // A recursive enum variant can contain more than one direct recursive
        // payload (for example `Add(Expr, Expr)`). Box every direct edge in
        // the recursive component; boxing only one edge still leaves an
        // infinite-size variant in that case. The candidate ordering above is
        // retained for deterministic diagnostics and future minimal boxing.
        for (_, enum_name, variant_index, field_index) in candidates {
            if let Some(Item::Enum(enum_decl)) = program.items.iter_mut().find_map(|item| {
                (matches!(&item.node, Item::Enum(value) if value.name == enum_name))
                    .then_some(&mut item.node)
            }) {
                if let Some(field) = enum_decl
                    .variants
                    .get_mut(variant_index)
                    .and_then(|variant| variant.fields.get_mut(field_index))
                {
                    *field = box_type(field);
                }
            }
        }
    }
}

fn direct_class_type(ty: &str) -> Option<&str> {
    let ty = ty.strip_suffix('?').unwrap_or(ty);
    if ty.starts_with('&')
        || ty.starts_with("List<")
        || ty.starts_with("Map<")
        || ty.starts_with("Set<")
        || ty.starts_with("Box<")
        || ty.starts_with("shared ")
    {
        return None;
    }
    Some(ty)
}

fn reaches(classes: &[(String, Vec<(String, String)>)], from: &str, to: &str) -> bool {
    let mut seen = HashSet::new();
    let mut stack = vec![from.to_string()];
    while let Some(current) = stack.pop() {
        if current == to {
            return true;
        }
        if !seen.insert(current.clone()) {
            continue;
        }
        if let Some((_, fields)) = classes.iter().find(|(name, _)| name == &current) {
            stack.extend(fields.iter().map(|(_, target)| target.clone()));
        }
    }
    false
}

fn box_type(ty: &str) -> String {
    if let Some(inner) = ty.strip_suffix('?') {
        format!("Box<{inner}>?")
    } else {
        format!("Box<{ty}>")
    }
}

fn box_field_value(ty: &str, value: String) -> String {
    if !ty.starts_with("Box<") {
        return value;
    }
    if ty.ends_with('?') {
        if value == "None" {
            "None".into()
        } else if let Some(inner) = value
            .strip_prefix("Some(")
            .and_then(|v| v.strip_suffix(')'))
        {
            format!("Some(Box::new({inner}))")
        } else {
            format!("Some(Box::new({value}))")
        }
    } else {
        format!("Box::new({value})")
    }
}

fn constructor_param_type(ty: &str, interfaces: &HashSet<String>, with_spawn: bool) -> String {
    if let Some(inner) = ty.strip_suffix('?').and_then(|value| {
        value
            .strip_prefix("Box<")
            .and_then(|value| value.strip_suffix('>'))
    }) {
        return format!(
            "Option<{}>",
            class_field_type(inner, interfaces, with_spawn)
        );
    }
    if let Some(inner) = ty
        .strip_prefix("Box<")
        .and_then(|value| value.strip_suffix('>'))
    {
        return class_field_type(inner, interfaces, with_spawn);
    }
    class_field_type(ty, interfaces, with_spawn)
}

fn visibility_prefix(visibility: orust_syntax::Visibility, exported: bool) -> &'static str {
    match visibility {
        orust_syntax::Visibility::Private => "",
        orust_syntax::Visibility::Internal => "pub(crate) ",
        orust_syntax::Visibility::Public if exported => "pub ",
        orust_syntax::Visibility::Public => "",
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpanMapping {
    pub generated_start: usize,
    pub generated_end: usize,
    pub source_span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedRust {
    pub code: String,
    pub spans: Vec<SpanMapping>,
}

fn name_map(program: &Program) -> HashMap<String, String> {
    program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Class(v) => v
                .rust_name
                .as_ref()
                .map(|name| (v.name.clone(), name.clone())),
            Item::Function(v) => v
                .rust_name
                .as_ref()
                .map(|name| (v.name.clone(), name.clone())),
            Item::Interface(v) => v
                .rust_name
                .as_ref()
                .map(|name| (v.name.clone(), name.clone())),
            Item::Enum(v) => v
                .rust_name
                .as_ref()
                .map(|name| (v.name.clone(), name.clone())),
            Item::Error(v) => v
                .rust_name
                .as_ref()
                .map(|name| (v.name.clone(), name.clone())),
            Item::Extension(_) => None,
            Item::TypeAlias(v) => v
                .rust_name
                .as_ref()
                .map(|name| (v.name.clone(), name.clone())),
            Item::Newtype(v) => v
                .rust_name
                .as_ref()
                .map(|name| (v.name.clone(), name.clone())),
        })
        .collect()
}

fn rename_type(value: &mut String, names: &HashMap<String, String>) {
    for (source, target) in names {
        if value == source {
            *value = target.clone();
        } else {
            *value = value.replace(source, target);
        }
    }
}

fn add_borrowed_class_lifetimes(value: &mut String, borrowed: &HashSet<String>) {
    let ty = value.trim().to_string();
    if let Some(inner) = ty.strip_prefix('&') {
        let prefix = if inner.starts_with("mut ") {
            "&mut "
        } else {
            "&"
        };
        let inner = inner.strip_prefix("mut ").unwrap_or(inner);
        let mut nested = inner.to_string();
        add_borrowed_class_lifetimes(&mut nested, borrowed);
        *value = format!("{prefix}{nested}");
    } else if let Some(inner) = ty.strip_suffix('?') {
        let mut nested = inner.to_string();
        add_borrowed_class_lifetimes(&mut nested, borrowed);
        *value = format!("{nested}?");
    } else if borrowed.contains(&ty) && !ty.contains('<') {
        *value = format!("{ty}<'_>");
    } else if let Some(open) = ty.find('<') {
        if ty.ends_with('>') {
            let base = &ty[..open];
            let args = &ty[open + 1..ty.len() - 1];
            let mut rendered = Vec::new();
            for arg in args.split(", ") {
                let mut nested = arg.to_string();
                add_borrowed_class_lifetimes(&mut nested, borrowed);
                rendered.push(nested);
            }
            *value = format!("{base}<{}>", rendered.join(", "));
        }
    }
}

fn rename_expr(expression: &mut Expr, names: &HashMap<String, String>) {
    match expression {
        Expr::Name(name) | Expr::New(name) => {
            if let Some(target) = names.get(name) {
                *name = target.clone();
            }
        }
        Expr::NewArgs { name, args } => {
            if let Some(target) = names.get(name) {
                *name = target.clone();
            }
            for arg in args {
                rename_expr(arg, names);
            }
        }
        Expr::Index { object, index } => {
            rename_expr(object, names);
            rename_expr(index, names);
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            rename_expr(object, names);
            if let Some(start) = start {
                rename_expr(start, names);
            }
            if let Some(end) = end {
                rename_expr(end, names);
            }
        }
        Expr::Call { callee, args } => {
            rename_expr(callee, names);
            for arg in args {
                rename_expr(arg, names);
            }
        }
        Expr::Member { object, .. } | Expr::OptionalMember { object, .. } => {
            rename_expr(object, names)
        }
        Expr::Borrow { value, .. }
        | Expr::Copy(value)
        | Expr::Await(value)
        | Expr::Spawn(value) => rename_expr(value, names),
        Expr::Coalesce { left, right } | Expr::Binary { left, right, .. } => {
            rename_expr(left, names);
            rename_expr(right, names);
        }
        Expr::List(values) => {
            for value in values {
                rename_expr(value, names);
            }
        }
        Expr::NamedArg { value, .. } => rename_expr(value, names),
        Expr::Closure { body, .. } => match body {
            orust_syntax::ClosureBody::Expr(value) => rename_expr(value, names),
            orust_syntax::ClosureBody::Block(body) => rename_statements(body, names),
        },
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Null
        | Expr::This => {}
    }
}

fn rename_statements(
    statements: &mut [orust_syntax::Spanned<Stmt>],
    names: &HashMap<String, String>,
) {
    for statement in statements {
        match &mut statement.node {
            Stmt::Var { initializer, .. }
            | Stmt::Print(initializer)
            | Stmt::Expr(initializer)
            | Stmt::Return(Some(initializer))
            | Stmt::Throw(initializer) => rename_expr(initializer, names),
            Stmt::PatternVar {
                initializer,
                else_body,
                ..
            } => {
                rename_expr(initializer, names);
                rename_statements(else_body, names);
            }
            Stmt::Return(None) | Stmt::Rust(_) => {}
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                rename_expr(condition, names);
                rename_statements(then_body, names);
                rename_statements(else_body, names);
            }
            Stmt::While { condition, body }
            | Stmt::WhileCase {
                value: condition,
                body,
                ..
            } => {
                rename_expr(condition, names);
                rename_statements(body, names);
            }
            Stmt::For {
                initializer,
                condition,
                step,
                body,
            } => {
                if let Some(initializer) = initializer {
                    if let Stmt::Var { initializer, .. } = initializer.as_mut() {
                        rename_expr(initializer, names);
                    }
                }
                if let Some(condition) = condition {
                    rename_expr(condition, names);
                }
                if let Some(step) = step {
                    rename_expr(step, names);
                }
                rename_statements(body, names);
            }
            Stmt::ForIn { iterable, body, .. } => {
                rename_expr(iterable, names);
                rename_statements(body, names);
            }
            Stmt::TryCatch {
                body, catch_body, ..
            } => {
                rename_statements(body, names);
                rename_statements(catch_body, names);
            }
            Stmt::Switch { value, cases } => {
                rename_expr(value, names);
                for case in cases {
                    rename_statements(&mut case.body, names);
                }
            }
            Stmt::IfCase {
                value,
                then_body,
                else_body,
                ..
            } => {
                rename_expr(value, names);
                rename_statements(then_body, names);
                rename_statements(else_body, names);
            }
            Stmt::AwaitFor { stream, body, .. } => {
                rename_expr(stream, names);
                rename_statements(body, names);
            }
        }
    }
}

fn rewrite_namespace_expr(expression: &mut Expr, namespaces: &HashMap<String, String>) {
    match expression {
        Expr::Member { object, .. } | Expr::OptionalMember { object, .. } => {
            if let Expr::Name(name) = object.as_ref() {
                if let Some(namespace) = namespaces.get(name) {
                    **object = Expr::Name(namespace.clone());
                }
            }
            rewrite_namespace_expr(object, namespaces);
        }
        Expr::Call { callee, args } => {
            rewrite_namespace_expr(callee, namespaces);
            for arg in args {
                rewrite_namespace_expr(arg, namespaces);
            }
        }
        Expr::Borrow { value, .. }
        | Expr::Copy(value)
        | Expr::Await(value)
        | Expr::Spawn(value) => rewrite_namespace_expr(value, namespaces),
        Expr::Coalesce { left, right } | Expr::Binary { left, right, .. } => {
            rewrite_namespace_expr(left, namespaces);
            rewrite_namespace_expr(right, namespaces);
        }
        Expr::List(values) => {
            for value in values {
                rewrite_namespace_expr(value, namespaces);
            }
        }
        Expr::NamedArg { value, .. } => rewrite_namespace_expr(value, namespaces),
        Expr::Closure { body, .. } => match body {
            orust_syntax::ClosureBody::Expr(value) => rewrite_namespace_expr(value, namespaces),
            orust_syntax::ClosureBody::Block(body) => {
                rewrite_namespace_statements(body, namespaces)
            }
        },
        Expr::NewArgs { args, .. } => {
            for arg in args {
                rewrite_namespace_expr(arg, namespaces);
            }
        }
        Expr::Index { object, index } => {
            rewrite_namespace_expr(object, namespaces);
            rewrite_namespace_expr(index, namespaces);
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            rewrite_namespace_expr(object, namespaces);
            if let Some(start) = start {
                rewrite_namespace_expr(start, namespaces);
            }
            if let Some(end) = end {
                rewrite_namespace_expr(end, namespaces);
            }
        }
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::New(_)
        | Expr::Name(_)
        | Expr::Null
        | Expr::This => {}
    }
}

fn rewrite_namespace_statements(
    statements: &mut [orust_syntax::Spanned<Stmt>],
    namespaces: &HashMap<String, String>,
) {
    for statement in statements {
        match &mut statement.node {
            Stmt::Var { initializer, .. }
            | Stmt::Print(initializer)
            | Stmt::Expr(initializer)
            | Stmt::Return(Some(initializer))
            | Stmt::Throw(initializer) => rewrite_namespace_expr(initializer, namespaces),
            Stmt::PatternVar {
                initializer,
                else_body,
                ..
            } => {
                rewrite_namespace_expr(initializer, namespaces);
                rewrite_namespace_statements(else_body, namespaces);
            }
            Stmt::Return(None) | Stmt::Rust(_) => {}
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                rewrite_namespace_expr(condition, namespaces);
                rewrite_namespace_statements(then_body, namespaces);
                rewrite_namespace_statements(else_body, namespaces);
            }
            Stmt::While { condition, body }
            | Stmt::WhileCase {
                value: condition,
                body,
                ..
            } => {
                rewrite_namespace_expr(condition, namespaces);
                rewrite_namespace_statements(body, namespaces);
            }
            Stmt::For {
                condition,
                step,
                body,
                ..
            } => {
                if let Some(condition) = condition {
                    rewrite_namespace_expr(condition, namespaces);
                }
                if let Some(step) = step {
                    rewrite_namespace_expr(step, namespaces);
                }
                rewrite_namespace_statements(body, namespaces);
            }
            Stmt::ForIn { iterable, body, .. } => {
                rewrite_namespace_expr(iterable, namespaces);
                rewrite_namespace_statements(body, namespaces);
            }
            Stmt::TryCatch {
                body, catch_body, ..
            } => {
                rewrite_namespace_statements(body, namespaces);
                rewrite_namespace_statements(catch_body, namespaces);
            }
            Stmt::Switch { value, cases } => {
                rewrite_namespace_expr(value, namespaces);
                for case in cases {
                    rewrite_namespace_statements(&mut case.body, namespaces);
                }
            }
            Stmt::IfCase {
                value,
                then_body,
                else_body,
                ..
            } => {
                rewrite_namespace_expr(value, namespaces);
                rewrite_namespace_statements(then_body, namespaces);
                rewrite_namespace_statements(else_body, namespaces);
            }
            Stmt::AwaitFor { stream, body, .. } => {
                rewrite_namespace_expr(stream, namespaces);
                rewrite_namespace_statements(body, namespaces);
            }
        }
    }
}

fn rewrite_source_references(program: &mut Program) {
    let names = name_map(program);
    let borrowed_classes: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Class(class) if class.fields.iter().any(|field| field.ty.starts_with('&')) => {
                Some(class.name.clone())
            }
            _ => None,
        })
        .collect();
    let namespaces: HashMap<String, String> = program
        .imports
        .iter()
        .filter_map(|import| {
            if let Some(alias) = &import.alias {
                return Some((alias.clone(), format!("__orust_ns__{alias}")));
            }
            import.path.strip_prefix("rust:").and_then(|path| {
                path.rsplit('/')
                    .next()?
                    .strip_suffix(".rs")
                    .map(|name| (name.to_string(), format!("__orust_ns__crate::{name}")))
            })
        })
        .collect();
    for item in &mut program.items {
        match &mut item.node {
            Item::Class(class) => {
                for field in &mut class.fields {
                    rename_type(&mut field.ty, &names);
                    if let Some(value) = &mut field.initializer {
                        rename_expr(value, &names);
                    }
                }
                for method in &mut class.methods {
                    rename_type(&mut method.return_type, &names);
                    add_borrowed_class_lifetimes(&mut method.return_type, &borrowed_classes);
                    for param in &mut method.params {
                        rename_type(&mut param.ty, &names);
                        add_borrowed_class_lifetimes(&mut param.ty, &borrowed_classes);
                    }
                    rename_statements(&mut method.body, &names);
                    rewrite_namespace_statements(&mut method.body, &namespaces);
                }
                if let Some(body) = &mut class.drop_body {
                    rename_statements(body, &names);
                    rewrite_namespace_statements(body, &namespaces);
                }
                for implementation in &mut class.implements {
                    if let Some(target) = names.get(implementation) {
                        *implementation = target.clone();
                    }
                }
            }
            Item::Function(function) => {
                rename_type(&mut function.return_type, &names);
                add_borrowed_class_lifetimes(&mut function.return_type, &borrowed_classes);
                for param in &mut function.params {
                    rename_type(&mut param.ty, &names);
                    add_borrowed_class_lifetimes(&mut param.ty, &borrowed_classes);
                }
                rename_statements(&mut function.body, &names);
                rewrite_namespace_statements(&mut function.body, &namespaces);
            }
            Item::Interface(interface) => {
                for method in &mut interface.methods {
                    rename_type(&mut method.return_type, &names);
                    add_borrowed_class_lifetimes(&mut method.return_type, &borrowed_classes);
                    for param in &mut method.params {
                        rename_type(&mut param.ty, &names);
                        add_borrowed_class_lifetimes(&mut param.ty, &borrowed_classes);
                    }
                }
            }
            Item::Enum(enum_decl) => {
                for variant in &mut enum_decl.variants {
                    for field in &mut variant.fields {
                        rename_type(field, &names);
                    }
                }
            }
            Item::Error(error) => {
                for field in &mut error.fields {
                    rename_type(&mut field.ty, &names);
                    add_borrowed_class_lifetimes(&mut field.ty, &borrowed_classes);
                }
                for case in &mut error.cases {
                    for field in &mut case.fields {
                        rename_type(&mut field.ty, &names);
                        add_borrowed_class_lifetimes(&mut field.ty, &borrowed_classes);
                    }
                }
            }
            Item::Extension(extension) => {
                for method in &mut extension.methods {
                    rename_type(&mut method.return_type, &names);
                    add_borrowed_class_lifetimes(&mut method.return_type, &borrowed_classes);
                    rename_statements(&mut method.body, &names);
                    rewrite_namespace_statements(&mut method.body, &namespaces);
                }
            }
            Item::TypeAlias(_) | Item::Newtype(_) => {}
        }
    }
    let mut boxed_optional_fields = HashSet::new();
    let mut boxed_value_fields = HashSet::new();
    for item in &program.items {
        if let Item::Class(class) = &item.node {
            for field in &class.fields {
                if field.ty.starts_with("Box<") {
                    if field.ty.ends_with('?') {
                        boxed_optional_fields.insert(field.name.clone());
                    } else {
                        boxed_value_fields.insert(field.name.clone());
                    }
                }
            }
        }
    }
    for item in &mut program.items {
        match &mut item.node {
            Item::Class(class) => {
                for method in &mut class.methods {
                    rewrite_boxed_field_statements(
                        &mut method.body,
                        &boxed_optional_fields,
                        &boxed_value_fields,
                    );
                }
            }
            Item::Function(function) => rewrite_boxed_field_statements(
                &mut function.body,
                &boxed_optional_fields,
                &boxed_value_fields,
            ),
            Item::Extension(extension) => {
                for method in &mut extension.methods {
                    rewrite_boxed_field_statements(
                        &mut method.body,
                        &boxed_optional_fields,
                        &boxed_value_fields,
                    );
                }
            }
            Item::Interface(_)
            | Item::Enum(_)
            | Item::Error(_)
            | Item::TypeAlias(_)
            | Item::Newtype(_) => {}
        }
    }
    let newtypes: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Newtype(newtype) => Some(declared_name(&newtype.name, &newtype.rust_name)),
            _ => None,
        })
        .collect();
    for item in &mut program.items {
        match &mut item.node {
            Item::Class(class) => {
                for constructor in &mut class.constructors {
                    rewrite_newtype_value_statements(
                        &mut constructor.body,
                        &newtypes,
                        &HashMap::new(),
                    );
                }
                for method in &mut class.methods {
                    let variables = method
                        .params
                        .iter()
                        .filter(|parameter| {
                            newtypes.contains(&parameter.ty) || parameter.ty.ends_with('?')
                        })
                        .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
                        .collect();
                    rewrite_newtype_value_statements(&mut method.body, &newtypes, &variables);
                }
            }
            Item::Function(function) => {
                let variables = function
                    .params
                    .iter()
                    .filter(|parameter| {
                        newtypes.contains(&parameter.ty) || parameter.ty.ends_with('?')
                    })
                    .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
                    .collect();
                rewrite_newtype_value_statements(&mut function.body, &newtypes, &variables);
            }
            Item::Extension(extension) => {
                for method in &mut extension.methods {
                    let variables = method
                        .params
                        .iter()
                        .filter(|parameter| {
                            newtypes.contains(&parameter.ty) || parameter.ty.ends_with('?')
                        })
                        .map(|parameter| (parameter.name.clone(), parameter.ty.clone()))
                        .collect();
                    rewrite_newtype_value_statements(&mut method.body, &newtypes, &variables);
                }
            }
            Item::Interface(_)
            | Item::Enum(_)
            | Item::Error(_)
            | Item::TypeAlias(_)
            | Item::Newtype(_) => {}
        }
    }
}

fn rewrite_boxed_field_expr(
    expression: &mut Expr,
    boxed_optional_fields: &HashSet<String>,
    boxed_value_fields: &HashSet<String>,
) {
    match expression {
        Expr::Member { object, name } => {
            rewrite_boxed_field_expr(object, boxed_optional_fields, boxed_value_fields);
            if boxed_optional_fields.contains(name) {
                *name = format!("__orust_box_optional__{name}");
            } else if boxed_value_fields.contains(name) {
                *name = format!("__orust_box_value__{name}");
            }
        }
        Expr::OptionalMember { object, .. } => {
            rewrite_boxed_field_expr(object, boxed_optional_fields, boxed_value_fields);
        }
        Expr::Call { callee, args } => {
            rewrite_boxed_field_expr(callee, boxed_optional_fields, boxed_value_fields);
            for argument in args {
                rewrite_boxed_field_expr(argument, boxed_optional_fields, boxed_value_fields);
            }
        }
        Expr::NewArgs { args, .. } | Expr::List(args) => {
            for argument in args {
                rewrite_boxed_field_expr(argument, boxed_optional_fields, boxed_value_fields);
            }
        }
        Expr::Index { object, index } => {
            rewrite_boxed_field_expr(object, boxed_optional_fields, boxed_value_fields);
            rewrite_boxed_field_expr(index, boxed_optional_fields, boxed_value_fields);
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            rewrite_boxed_field_expr(object, boxed_optional_fields, boxed_value_fields);
            if let Some(start) = start {
                rewrite_boxed_field_expr(start, boxed_optional_fields, boxed_value_fields);
            }
            if let Some(end) = end {
                rewrite_boxed_field_expr(end, boxed_optional_fields, boxed_value_fields);
            }
        }
        Expr::Borrow { value, .. }
        | Expr::Copy(value)
        | Expr::Await(value)
        | Expr::Spawn(value) => {
            rewrite_boxed_field_expr(value, boxed_optional_fields, boxed_value_fields)
        }
        Expr::Binary { left, right, .. } | Expr::Coalesce { left, right } => {
            rewrite_boxed_field_expr(left, boxed_optional_fields, boxed_value_fields);
            rewrite_boxed_field_expr(right, boxed_optional_fields, boxed_value_fields);
        }
        Expr::NamedArg { value, .. } => {
            rewrite_boxed_field_expr(value, boxed_optional_fields, boxed_value_fields)
        }
        Expr::Closure { body, .. } => match body {
            ClosureBody::Expr(value) => {
                rewrite_boxed_field_expr(value, boxed_optional_fields, boxed_value_fields)
            }
            ClosureBody::Block(body) => {
                rewrite_boxed_field_statements(body, boxed_optional_fields, boxed_value_fields)
            }
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

fn rewrite_boxed_field_statements(
    statements: &mut [orust_syntax::Spanned<Stmt>],
    boxed_optional_fields: &HashSet<String>,
    boxed_value_fields: &HashSet<String>,
) {
    for statement in statements {
        match &mut statement.node {
            Stmt::Var { initializer, .. }
            | Stmt::Print(initializer)
            | Stmt::Expr(initializer)
            | Stmt::Return(Some(initializer))
            | Stmt::Throw(initializer) => {
                rewrite_boxed_field_expr(initializer, boxed_optional_fields, boxed_value_fields)
            }
            Stmt::PatternVar {
                initializer,
                else_body,
                ..
            } => {
                rewrite_boxed_field_expr(initializer, boxed_optional_fields, boxed_value_fields);
                rewrite_boxed_field_statements(
                    else_body,
                    boxed_optional_fields,
                    boxed_value_fields,
                );
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                rewrite_boxed_field_expr(condition, boxed_optional_fields, boxed_value_fields);
                rewrite_boxed_field_statements(
                    then_body,
                    boxed_optional_fields,
                    boxed_value_fields,
                );
                rewrite_boxed_field_statements(
                    else_body,
                    boxed_optional_fields,
                    boxed_value_fields,
                );
            }
            Stmt::While { condition, body }
            | Stmt::WhileCase {
                value: condition,
                body,
                ..
            } => {
                rewrite_boxed_field_expr(condition, boxed_optional_fields, boxed_value_fields);
                rewrite_boxed_field_statements(body, boxed_optional_fields, boxed_value_fields);
            }
            Stmt::For {
                initializer,
                condition,
                step,
                body,
            } => {
                if let Some(initializer) = initializer {
                    if let Stmt::Var { initializer, .. } = initializer.as_mut() {
                        rewrite_boxed_field_expr(
                            initializer,
                            boxed_optional_fields,
                            boxed_value_fields,
                        );
                    }
                }
                if let Some(condition) = condition {
                    rewrite_boxed_field_expr(condition, boxed_optional_fields, boxed_value_fields);
                }
                if let Some(step) = step {
                    rewrite_boxed_field_expr(step, boxed_optional_fields, boxed_value_fields);
                }
                rewrite_boxed_field_statements(body, boxed_optional_fields, boxed_value_fields);
            }
            Stmt::ForIn { iterable, body, .. } => {
                rewrite_boxed_field_expr(iterable, boxed_optional_fields, boxed_value_fields);
                rewrite_boxed_field_statements(body, boxed_optional_fields, boxed_value_fields);
            }
            Stmt::TryCatch {
                body, catch_body, ..
            } => {
                rewrite_boxed_field_statements(body, boxed_optional_fields, boxed_value_fields);
                rewrite_boxed_field_statements(
                    catch_body,
                    boxed_optional_fields,
                    boxed_value_fields,
                );
            }
            Stmt::Switch { value, cases } => {
                rewrite_boxed_field_expr(value, boxed_optional_fields, boxed_value_fields);
                for case in cases {
                    rewrite_boxed_field_statements(
                        &mut case.body,
                        boxed_optional_fields,
                        boxed_value_fields,
                    );
                }
            }
            Stmt::IfCase {
                value,
                then_body,
                else_body,
                ..
            } => {
                rewrite_boxed_field_expr(value, boxed_optional_fields, boxed_value_fields);
                rewrite_boxed_field_statements(
                    then_body,
                    boxed_optional_fields,
                    boxed_value_fields,
                );
                rewrite_boxed_field_statements(
                    else_body,
                    boxed_optional_fields,
                    boxed_value_fields,
                );
            }
            Stmt::AwaitFor { stream, body, .. } => {
                rewrite_boxed_field_expr(stream, boxed_optional_fields, boxed_value_fields);
                rewrite_boxed_field_statements(body, boxed_optional_fields, boxed_value_fields);
            }
            Stmt::Return(None) | Stmt::Rust(_) => {}
        }
    }
}

fn rewrite_newtype_value_expr(
    expression: &mut Expr,
    variables: &HashMap<String, String>,
    newtypes: &HashSet<String>,
) {
    match expression {
        Expr::Member { object, name } | Expr::OptionalMember { object, name } => {
            rewrite_newtype_value_expr(object, variables, newtypes);
            if name == "value"
                && matches!(object.as_ref(), Expr::Name(variable) if variables.contains_key(variable))
            {
                *name = "__orust_newtype_value__".into();
            }
        }
        Expr::Call { callee, args } => {
            rewrite_newtype_value_expr(callee, variables, newtypes);
            for argument in args {
                rewrite_newtype_value_expr(argument, variables, newtypes);
            }
        }
        Expr::NewArgs { name, args } => {
            if newtypes.contains(name) {
                *name = format!("__orust_newtype__{name}");
            }
            for argument in args {
                rewrite_newtype_value_expr(argument, variables, newtypes);
            }
        }
        Expr::List(args) => {
            for argument in args {
                rewrite_newtype_value_expr(argument, variables, newtypes);
            }
        }
        Expr::Index { object, index } => {
            rewrite_newtype_value_expr(object, variables, newtypes);
            rewrite_newtype_value_expr(index, variables, newtypes);
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            rewrite_newtype_value_expr(object, variables, newtypes);
            if let Some(start) = start {
                rewrite_newtype_value_expr(start, variables, newtypes);
            }
            if let Some(end) = end {
                rewrite_newtype_value_expr(end, variables, newtypes);
            }
        }
        Expr::Borrow { value, .. }
        | Expr::Copy(value)
        | Expr::Await(value)
        | Expr::Spawn(value) => rewrite_newtype_value_expr(value, variables, newtypes),
        Expr::Binary { left, right, .. } | Expr::Coalesce { left, right } => {
            rewrite_newtype_value_expr(left, variables, newtypes);
            rewrite_newtype_value_expr(right, variables, newtypes);
        }
        Expr::NamedArg { value, .. } => rewrite_newtype_value_expr(value, variables, newtypes),
        Expr::Closure { body, .. } => match body {
            orust_syntax::ClosureBody::Expr(value) => {
                rewrite_newtype_value_expr(value, variables, newtypes)
            }
            orust_syntax::ClosureBody::Block(body) => {
                rewrite_newtype_value_statements(body, newtypes, variables)
            }
        },
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Null
        | Expr::This => {}
        Expr::Name(name) if variables.get(name).is_some_and(|ty| ty.ends_with('?')) => {
            *name = format!("__orust_nullable__{name}");
        }
        Expr::Name(_) => {}
        Expr::New(name) if newtypes.contains(name) => {
            *name = format!("__orust_newtype__{name}");
        }
        Expr::New(_) => {}
    }
}

fn rewrite_newtype_value_statements(
    statements: &mut [orust_syntax::Spanned<Stmt>],
    newtypes: &HashSet<String>,
    initial_variables: &HashMap<String, String>,
) {
    let mut variables = initial_variables.clone();
    for statement in statements {
        match &mut statement.node {
            Stmt::Var {
                name,
                initializer,
                declared_type,
            } => {
                let inferred = declared_type.clone().or_else(|| match initializer {
                    Expr::New(type_name) => Some(type_name.clone()),
                    Expr::NewArgs {
                        name: type_name, ..
                    } => Some(type_name.clone()),
                    _ => None,
                });
                if let Some(ty) = inferred.filter(|ty| newtypes.contains(ty)) {
                    variables.insert(name.clone(), ty);
                }
                if declared_type.as_deref().is_some_and(|ty| ty.ends_with('?')) {
                    variables.insert(name.clone(), declared_type.clone().unwrap());
                }
                rewrite_newtype_value_expr(initializer, &variables, newtypes);
            }
            Stmt::PatternVar {
                initializer,
                else_body,
                ..
            } => {
                rewrite_newtype_value_expr(initializer, &variables, newtypes);
                rewrite_newtype_value_statements(else_body, newtypes, &variables);
            }
            Stmt::Print(value)
            | Stmt::Expr(value)
            | Stmt::Return(Some(value))
            | Stmt::Throw(value) => rewrite_newtype_value_expr(value, &variables, newtypes),
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                rewrite_newtype_value_expr(condition, &variables, newtypes);
                rewrite_newtype_value_statements(then_body, newtypes, &variables);
                rewrite_newtype_value_statements(else_body, newtypes, &variables);
            }
            Stmt::While { condition, body }
            | Stmt::WhileCase {
                value: condition,
                body,
                ..
            } => {
                rewrite_newtype_value_expr(condition, &variables, newtypes);
                rewrite_newtype_value_statements(body, newtypes, &variables);
            }
            Stmt::For {
                initializer,
                condition,
                step,
                body,
            } => {
                if let Some(initializer) = initializer {
                    match initializer.as_mut() {
                        Stmt::Var { initializer, .. }
                        | Stmt::Print(initializer)
                        | Stmt::Expr(initializer)
                        | Stmt::Return(Some(initializer))
                        | Stmt::Throw(initializer) => {
                            rewrite_newtype_value_expr(initializer, &variables, newtypes)
                        }
                        Stmt::PatternVar {
                            initializer,
                            else_body,
                            ..
                        } => {
                            rewrite_newtype_value_expr(initializer, &variables, newtypes);
                            rewrite_newtype_value_statements(else_body, newtypes, &variables);
                        }
                        _ => {}
                    }
                }
                if let Some(condition) = condition {
                    rewrite_newtype_value_expr(condition, &variables, newtypes);
                }
                if let Some(step) = step {
                    rewrite_newtype_value_expr(step, &variables, newtypes);
                }
                rewrite_newtype_value_statements(body, newtypes, &variables);
            }
            Stmt::ForIn { iterable, body, .. }
            | Stmt::AwaitFor {
                stream: iterable,
                body,
                ..
            } => {
                rewrite_newtype_value_expr(iterable, &variables, newtypes);
                rewrite_newtype_value_statements(body, newtypes, &variables);
            }
            Stmt::TryCatch {
                body, catch_body, ..
            } => {
                rewrite_newtype_value_statements(body, newtypes, &variables);
                rewrite_newtype_value_statements(catch_body, newtypes, &variables);
            }
            Stmt::Switch { value, cases } => {
                rewrite_newtype_value_expr(value, &variables, newtypes);
                for case in cases {
                    rewrite_newtype_value_statements(&mut case.body, newtypes, &variables);
                }
            }
            Stmt::IfCase {
                value,
                then_body,
                else_body,
                ..
            } => {
                rewrite_newtype_value_expr(value, &variables, newtypes);
                rewrite_newtype_value_statements(then_body, newtypes, &variables);
                rewrite_newtype_value_statements(else_body, newtypes, &variables);
            }
            Stmt::Return(None) | Stmt::Rust(_) => {}
        }
    }
}

fn constructor_args_name(class_name: &str, constructor_name: &str) -> String {
    let mut suffix = String::new();
    for (index, character) in constructor_name.chars().enumerate() {
        if index == 0 {
            suffix.push(character.to_ascii_uppercase());
        } else {
            suffix.push(character);
        }
    }
    format!("{class_name}{suffix}Args")
}

fn constructor_encoding(class_name: &str, constructor_name: &str, params: &[Param]) -> String {
    format!(
        "__orust_ctor__{class_name}__{constructor_name}__{}__{}__{}",
        params
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>()
            .join(","),
        if params.iter().any(|parameter| parameter.default.is_some()) {
            "d"
        } else {
            "r"
        },
        params
            .iter()
            .map(|parameter| parameter.ty.as_str())
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn error_encoding(class_name: &str, variant: &str, fields: &[String]) -> String {
    format!(
        "__orust_error__{class_name}__{variant}__{}",
        fields.join(",")
    )
}

fn rewrite_constructor_expr(
    expression: &mut Expr,
    constructors: &HashMap<(String, String), Vec<Param>>,
    errors: &HashMap<(String, String), Vec<String>>,
) {
    fn needs_encoding(constructor_name: &str, params: &[Param]) -> bool {
        constructor_name != "new"
            || params.iter().any(|parameter| parameter.default.is_some())
            || params
                .iter()
                .any(|parameter| parameter.ty.starts_with("Box<") || parameter.ty.ends_with('?'))
    }

    fn target_parts(target: &str) -> (&str, &str) {
        target.split_once('.').unwrap_or((target, "new"))
    }
    match expression {
        Expr::New(name) => {
            let (class_name, constructor_name) = target_parts(name);
            if let Some(params) =
                constructors.get(&(class_name.to_string(), constructor_name.to_string()))
            {
                if needs_encoding(constructor_name, params) {
                    *expression = Expr::NewArgs {
                        name: constructor_encoding(class_name, constructor_name, params),
                        args: Vec::new(),
                    };
                }
            } else if let Some(fields) =
                errors.get(&(class_name.to_string(), constructor_name.to_string()))
            {
                *expression = Expr::NewArgs {
                    name: error_encoding(class_name, constructor_name, fields),
                    args: Vec::new(),
                };
            }
        }
        Expr::NewArgs { name, args } => {
            for argument in args.iter_mut() {
                rewrite_constructor_expr(argument, constructors, errors);
            }
            let (class_name, constructor_name) = target_parts(name);
            if let Some(params) =
                constructors.get(&(class_name.to_string(), constructor_name.to_string()))
            {
                if needs_encoding(constructor_name, params) {
                    let encoded = constructor_encoding(class_name, constructor_name, params);
                    *name = encoded;
                }
            } else if let Some(fields) =
                errors.get(&(class_name.to_string(), constructor_name.to_string()))
            {
                *name = error_encoding(class_name, constructor_name, fields);
            }
        }
        Expr::Call { callee, args } => {
            rewrite_constructor_expr(callee, constructors, errors);
            for argument in args {
                rewrite_constructor_expr(argument, constructors, errors);
            }
        }
        Expr::Member { object, .. } | Expr::OptionalMember { object, .. } => {
            rewrite_constructor_expr(object, constructors, errors)
        }
        Expr::Borrow { value, .. }
        | Expr::Copy(value)
        | Expr::Await(value)
        | Expr::Spawn(value) => rewrite_constructor_expr(value, constructors, errors),
        Expr::Coalesce { left, right } | Expr::Binary { left, right, .. } => {
            rewrite_constructor_expr(left, constructors, errors);
            rewrite_constructor_expr(right, constructors, errors);
        }
        Expr::Index { object, index } => {
            rewrite_constructor_expr(object, constructors, errors);
            rewrite_constructor_expr(index, constructors, errors);
        }
        Expr::Slice {
            object, start, end, ..
        } => {
            rewrite_constructor_expr(object, constructors, errors);
            if let Some(start) = start {
                rewrite_constructor_expr(start, constructors, errors);
            }
            if let Some(end) = end {
                rewrite_constructor_expr(end, constructors, errors);
            }
        }
        Expr::List(values) => {
            for value in values {
                rewrite_constructor_expr(value, constructors, errors);
            }
        }
        Expr::NamedArg { value, .. } => rewrite_constructor_expr(value, constructors, errors),
        Expr::Closure { body, .. } => match body {
            ClosureBody::Expr(value) => rewrite_constructor_expr(value, constructors, errors),
            ClosureBody::Block(body) => rewrite_constructor_statements(body, constructors, errors),
        },
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Name(_)
        | Expr::Null
        | Expr::This => {}
    }
}

fn rewrite_constructor_statements(
    statements: &mut [orust_syntax::Spanned<Stmt>],
    constructors: &HashMap<(String, String), Vec<Param>>,
    errors: &HashMap<(String, String), Vec<String>>,
) {
    for statement in statements {
        match &mut statement.node {
            Stmt::Var { initializer, .. }
            | Stmt::Print(initializer)
            | Stmt::Expr(initializer)
            | Stmt::Return(Some(initializer))
            | Stmt::Throw(initializer) => {
                rewrite_constructor_expr(initializer, constructors, errors)
            }
            Stmt::PatternVar {
                initializer,
                else_body,
                ..
            } => {
                rewrite_constructor_expr(initializer, constructors, errors);
                rewrite_constructor_statements(else_body, constructors, errors);
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                rewrite_constructor_expr(condition, constructors, errors);
                rewrite_constructor_statements(then_body, constructors, errors);
                rewrite_constructor_statements(else_body, constructors, errors);
            }
            Stmt::While { condition, body }
            | Stmt::WhileCase {
                value: condition,
                body,
                ..
            } => {
                rewrite_constructor_expr(condition, constructors, errors);
                rewrite_constructor_statements(body, constructors, errors);
            }
            Stmt::For {
                initializer,
                condition,
                step,
                body,
            } => {
                if let Some(initializer) = initializer {
                    if let Stmt::Var { initializer, .. } = initializer.as_mut() {
                        rewrite_constructor_expr(initializer, constructors, errors);
                    }
                }
                if let Some(condition) = condition {
                    rewrite_constructor_expr(condition, constructors, errors);
                }
                if let Some(step) = step {
                    rewrite_constructor_expr(step, constructors, errors);
                }
                rewrite_constructor_statements(body, constructors, errors);
            }
            Stmt::ForIn { iterable, body, .. } => {
                rewrite_constructor_expr(iterable, constructors, errors);
                rewrite_constructor_statements(body, constructors, errors);
            }
            Stmt::TryCatch {
                body, catch_body, ..
            } => {
                rewrite_constructor_statements(body, constructors, errors);
                rewrite_constructor_statements(catch_body, constructors, errors);
            }
            Stmt::Switch { value, cases } => {
                rewrite_constructor_expr(value, constructors, errors);
                for case in cases {
                    rewrite_constructor_statements(&mut case.body, constructors, errors);
                }
            }
            Stmt::IfCase {
                value,
                then_body,
                else_body,
                ..
            } => {
                rewrite_constructor_expr(value, constructors, errors);
                rewrite_constructor_statements(then_body, constructors, errors);
                rewrite_constructor_statements(else_body, constructors, errors);
            }
            Stmt::AwaitFor { stream, body, .. } => {
                rewrite_constructor_expr(stream, constructors, errors);
                rewrite_constructor_statements(body, constructors, errors);
            }
            Stmt::Return(None) | Stmt::Rust(_) => {}
        }
    }
}

fn rewrite_enum_constructors(program: &mut Program) {
    let variants = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Enum(enum_decl) => Some(
                enum_decl
                    .variants
                    .iter()
                    .map(|variant| {
                        (
                            (enum_decl.name.clone(), variant.name.clone()),
                            variant.fields.clone(),
                        )
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten()
        .collect::<HashMap<_, _>>();

    fn rewrite_expr(expression: &mut Expr, variants: &HashMap<(String, String), Vec<String>>) {
        match expression {
            Expr::New(name) => {
                if let Some((enum_name, variant_name)) = name.split_once('.') {
                    if let Some(fields) =
                        variants.get(&(enum_name.to_string(), variant_name.to_string()))
                    {
                        *expression = Expr::NewArgs {
                            name: format!(
                                "__orust_enum__{enum_name}__{variant_name}__{}",
                                fields.join(",")
                            ),
                            args: Vec::new(),
                        };
                    }
                }
            }
            Expr::NewArgs { name, args } => {
                for argument in args.iter_mut() {
                    rewrite_expr(argument, variants);
                }
                if let Some((enum_name, variant_name)) = name.split_once('.') {
                    if let Some(fields) =
                        variants.get(&(enum_name.to_string(), variant_name.to_string()))
                    {
                        *name = format!(
                            "__orust_enum__{enum_name}__{variant_name}__{}",
                            fields.join(",")
                        );
                    }
                }
            }
            Expr::Call { callee, args } => {
                rewrite_expr(callee, variants);
                for argument in args {
                    rewrite_expr(argument, variants);
                }
            }
            Expr::Member { object, .. }
            | Expr::OptionalMember { object, .. }
            | Expr::Borrow { value: object, .. }
            | Expr::Copy(object)
            | Expr::Await(object)
            | Expr::Spawn(object) => rewrite_expr(object, variants),
            Expr::Coalesce { left, right } | Expr::Binary { left, right, .. } => {
                rewrite_expr(left, variants);
                rewrite_expr(right, variants);
            }
            Expr::Index { object, index } => {
                rewrite_expr(object, variants);
                rewrite_expr(index, variants);
            }
            Expr::Slice {
                object, start, end, ..
            } => {
                rewrite_expr(object, variants);
                if let Some(start) = start {
                    rewrite_expr(start, variants);
                }
                if let Some(end) = end {
                    rewrite_expr(end, variants);
                }
            }
            Expr::List(values) => {
                for value in values {
                    rewrite_expr(value, variants);
                }
            }
            Expr::NamedArg { value, .. } => rewrite_expr(value, variants),
            Expr::Closure { body, .. } => match body {
                ClosureBody::Expr(value) => rewrite_expr(value, variants),
                ClosureBody::Block(body) => rewrite_statements(body, variants),
            },
            Expr::Int(_)
            | Expr::Float(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Name(_)
            | Expr::Null
            | Expr::This => {}
        }
    }

    fn mark_boxed_pattern_bindings(
        pattern: &str,
        bindings: &mut Vec<String>,
        variants: &HashMap<(String, String), Vec<String>>,
    ) {
        let Some(open) = pattern.find('(') else {
            return;
        };
        let variant = pattern[..open]
            .trim()
            .rsplit("::")
            .next()
            .unwrap_or_default();
        let Some(close) = pattern[open + 1..].find(')') else {
            return;
        };
        let inner = &pattern[open + 1..open + 1 + close];
        let Some(fields) = variants
            .iter()
            .find_map(|((_, name), fields)| (name == variant).then_some(fields))
        else {
            return;
        };
        let names = inner.split(',').map(str::trim).collect::<Vec<_>>();
        for (index, field) in fields.iter().enumerate() {
            if field.starts_with("Box<") {
                if let Some(name) = names
                    .get(index)
                    .filter(|name| !name.is_empty() && **name != "_")
                {
                    bindings.push(format!("__orust_box_binding__{name}"));
                }
            }
        }
    }

    fn qualify_pattern(pattern: &str, variants: &HashMap<(String, String), Vec<String>>) -> String {
        pattern
            .split('|')
            .map(|branch| {
                let branch = branch.trim();
                let (head, guard) = branch
                    .split_once(" when ")
                    .map_or((branch, None), |(head, guard)| (head.trim(), Some(guard)));
                if head.contains("::") {
                    return branch.to_string();
                }
                let Some(open) = head.find('(') else {
                    return branch.to_string();
                };
                let name = head[..open].trim();
                let Some(enum_name) = variants.iter().find_map(|((enum_name, variant), _)| {
                    (variant == name).then_some(enum_name.as_str())
                }) else {
                    return branch.to_string();
                };
                let qualified = format!("{enum_name}::{head}");
                guard.map_or(qualified.clone(), |guard| format!("{qualified} if {guard}"))
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }

    fn rewrite_statements(
        statements: &mut [orust_syntax::Spanned<Stmt>],
        variants: &HashMap<(String, String), Vec<String>>,
    ) {
        for statement in statements {
            match &mut statement.node {
                Stmt::Var { initializer, .. }
                | Stmt::Print(initializer)
                | Stmt::Expr(initializer)
                | Stmt::Return(Some(initializer))
                | Stmt::Throw(initializer) => rewrite_expr(initializer, variants),
                Stmt::PatternVar {
                    initializer,
                    else_body,
                    ..
                } => {
                    rewrite_expr(initializer, variants);
                    rewrite_statements(else_body, variants);
                }
                Stmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    rewrite_expr(condition, variants);
                    rewrite_statements(then_body, variants);
                    rewrite_statements(else_body, variants);
                }
                Stmt::While { condition, body }
                | Stmt::WhileCase {
                    value: condition,
                    body,
                    ..
                } => {
                    rewrite_expr(condition, variants);
                    rewrite_statements(body, variants);
                }
                Stmt::For {
                    initializer,
                    condition,
                    step,
                    body,
                } => {
                    if let Some(initializer) = initializer {
                        if let Stmt::Var { initializer, .. } = initializer.as_mut() {
                            rewrite_expr(initializer, variants);
                        }
                    }
                    if let Some(condition) = condition {
                        rewrite_expr(condition, variants);
                    }
                    if let Some(step) = step {
                        rewrite_expr(step, variants);
                    }
                    rewrite_statements(body, variants);
                }
                Stmt::ForIn { iterable, body, .. } => {
                    rewrite_expr(iterable, variants);
                    rewrite_statements(body, variants);
                }
                Stmt::TryCatch {
                    body, catch_body, ..
                } => {
                    rewrite_statements(body, variants);
                    rewrite_statements(catch_body, variants);
                }
                Stmt::Switch { value, cases } => {
                    rewrite_expr(value, variants);
                    for case in cases {
                        case.pattern = qualify_pattern(&case.pattern, variants);
                        mark_boxed_pattern_bindings(&case.pattern, &mut case.bindings, variants);
                        rewrite_statements(&mut case.body, variants);
                    }
                }
                Stmt::IfCase {
                    value,
                    pattern,
                    bindings,
                    then_body,
                    else_body,
                    ..
                } => {
                    rewrite_expr(value, variants);
                    *pattern = qualify_pattern(pattern, variants);
                    mark_boxed_pattern_bindings(pattern, bindings, variants);
                    rewrite_statements(then_body, variants);
                    rewrite_statements(else_body, variants);
                }
                Stmt::AwaitFor { stream, body, .. } => {
                    rewrite_expr(stream, variants);
                    rewrite_statements(body, variants);
                }
                Stmt::Return(None) | Stmt::Rust(_) => {}
            }
        }
    }

    for item in &mut program.items {
        match &mut item.node {
            Item::Class(class) => {
                for constructor in &mut class.constructors {
                    rewrite_statements(&mut constructor.body, &variants);
                }
                for method in &mut class.methods {
                    rewrite_statements(&mut method.body, &variants);
                }
            }
            Item::Function(function) => rewrite_statements(&mut function.body, &variants),
            Item::Extension(extension) => {
                for method in &mut extension.methods {
                    rewrite_statements(&mut method.body, &variants);
                }
            }
            Item::Interface(_)
            | Item::Enum(_)
            | Item::Error(_)
            | Item::TypeAlias(_)
            | Item::Newtype(_) => {}
        }
    }
}

fn rewrite_constructor_calls(program: &mut Program) {
    let constructors: HashMap<(String, String), Vec<Param>> = program
        .items
        .iter()
        .flat_map(|item| match &item.node {
            Item::Class(class) => class
                .constructors
                .iter()
                .map(|constructor| {
                    (
                        (
                            declared_name(&class.name, &class.rust_name),
                            constructor.name.clone(),
                        ),
                        constructor.params.clone(),
                    )
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .collect();
    let errors: HashMap<(String, String), Vec<String>> = program
        .items
        .iter()
        .flat_map(|item| match &item.node {
            Item::Error(error) if error.cases.is_empty() => vec![(
                (declared_name(&error.name, &error.rust_name), "new".into()),
                error
                    .fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect(),
            )],
            Item::Error(error) => error
                .cases
                .iter()
                .map(|case| {
                    (
                        (
                            declared_name(&error.name, &error.rust_name),
                            case.name.clone(),
                        ),
                        case.fields.iter().map(|field| field.name.clone()).collect(),
                    )
                })
                .collect(),
            _ => Vec::new(),
        })
        .collect();
    if constructors.is_empty() && errors.is_empty() {
        return;
    }
    for item in &mut program.items {
        match &mut item.node {
            Item::Class(class) => {
                for method in &mut class.methods {
                    rewrite_constructor_statements(&mut method.body, &constructors, &errors);
                }
                for constructor in &mut class.constructors {
                    rewrite_constructor_statements(&mut constructor.body, &constructors, &errors);
                }
            }
            Item::Function(function) => {
                rewrite_constructor_statements(&mut function.body, &constructors, &errors);
            }
            _ => {}
        }
    }
}

pub fn emit_with_spans(program: &Program) -> GeneratedRust {
    let code = emit(program);
    let mut spans = Vec::new();
    let mut cursor = 0;
    for (index, item) in program.items.iter().enumerate() {
        let marker = match &item.node {
            Item::Class(class) => {
                format!("struct {}", declared_name(&class.name, &class.rust_name))
            }
            Item::Function(function) => {
                format!("fn {}", declared_name(&function.name, &function.rust_name))
            }
            Item::Interface(interface) => format!(
                "trait {}",
                declared_name(&interface.name, &interface.rust_name)
            ),
            Item::Enum(enum_decl) => format!(
                "enum {}",
                declared_name(&enum_decl.name, &enum_decl.rust_name)
            ),
            Item::Error(error) => format!(
                "{} {}",
                if error.cases.is_empty() {
                    "struct"
                } else {
                    "enum"
                },
                declared_name(&error.name, &error.rust_name)
            ),
            Item::Extension(extension) => format!("trait {}Extensions", extension.target),
            Item::TypeAlias(alias) => {
                format!("type {}", declared_name(&alias.name, &alias.rust_name))
            }
            Item::Newtype(newtype) => {
                format!(
                    "struct {}",
                    declared_name(&newtype.name, &newtype.rust_name)
                )
            }
        };
        if let Some(relative_start) = code[cursor..].find(&marker) {
            let start = cursor + relative_start;
            let end = program
                .items
                .get(index + 1)
                .and_then(|next| match &next.node {
                    Item::Class(class) => code[start..]
                        .find(&format!(
                            "struct {}",
                            declared_name(&class.name, &class.rust_name)
                        ))
                        .map(|offset| start + offset),
                    Item::Function(function) => code[start..]
                        .find(&format!(
                            "fn {}",
                            declared_name(&function.name, &function.rust_name)
                        ))
                        .map(|offset| start + offset),
                    Item::Interface(interface) => code[start..]
                        .find(&format!(
                            "trait {}",
                            declared_name(&interface.name, &interface.rust_name)
                        ))
                        .map(|offset| start + offset),
                    Item::Enum(enum_decl) => code[start..]
                        .find(&format!(
                            "enum {}",
                            declared_name(&enum_decl.name, &enum_decl.rust_name)
                        ))
                        .map(|offset| start + offset),
                    Item::Error(error) => code[start..]
                        .find(&format!(
                            "{} {}",
                            if error.cases.is_empty() {
                                "struct"
                            } else {
                                "enum"
                            },
                            declared_name(&error.name, &error.rust_name)
                        ))
                        .map(|offset| start + offset),
                    Item::Extension(extension) => code[start..]
                        .find(&format!("trait {}Extensions", extension.target))
                        .map(|offset| start + offset),
                    Item::TypeAlias(alias) => code[start..]
                        .find(&format!(
                            "type {}",
                            declared_name(&alias.name, &alias.rust_name)
                        ))
                        .map(|offset| start + offset),
                    Item::Newtype(newtype) => code[start..]
                        .find(&format!(
                            "struct {}",
                            declared_name(&newtype.name, &newtype.rust_name)
                        ))
                        .map(|offset| start + offset),
                })
                .unwrap_or(code.len());
            spans.push(SpanMapping {
                generated_start: start,
                generated_end: end,
                source_span: item.span,
            });
            cursor = start;
        }
    }
    GeneratedRust { code, spans }
}

pub fn emit(program: &Program) -> String {
    emit_internal(program, true)
}

/// Emit a program whose record declarations are supplied by an enclosing
/// generated project module. This keeps anonymous record shapes shared across
/// source files instead of creating one Rust type per module.
pub fn emit_without_records(program: &Program) -> String {
    emit_internal(program, false)
}

fn emit_internal(program: &Program, include_record_module: bool) -> String {
    let mut program = program.clone();
    box_recursive_fields(&mut program);
    rewrite_source_references(&mut program);
    rewrite_constructor_calls(&mut program);
    rewrite_enum_constructors(&mut program);
    let mut out = String::new();
    emit_rust_uses(&mut out, &program);
    emit_std_imports(&mut out, &program);
    let uses_spawn = program_uses_spawn(&program);
    let uses_copy = program_uses_copy(&program);
    let class_names: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Class(class) => Some(class.name.clone()),
            _ => None,
        })
        .collect();
    let mut records = HashMap::new();
    for item in &program.items {
        match &item.node {
            Item::Class(class) => {
                for field in &class.fields {
                    collect_record_type(&field.ty, &mut records);
                }
                for constructor in &class.constructors {
                    for parameter in &constructor.params {
                        collect_record_type(&parameter.ty, &mut records);
                    }
                    collect_record_statements(&constructor.body, &mut records);
                }
                for method in &class.methods {
                    collect_record_type(&method.return_type, &mut records);
                    for parameter in &method.params {
                        collect_record_type(&parameter.ty, &mut records);
                    }
                    collect_record_statements(&method.body, &mut records);
                }
            }
            Item::Function(function) => {
                collect_record_type(&function.return_type, &mut records);
                for parameter in &function.params {
                    collect_record_type(&parameter.ty, &mut records);
                }
                collect_record_statements(&function.body, &mut records);
            }
            Item::Interface(interface) => {
                for method in &interface.methods {
                    collect_record_type(&method.return_type, &mut records);
                    for parameter in &method.params {
                        collect_record_type(&parameter.ty, &mut records);
                    }
                }
            }
            Item::TypeAlias(alias) => collect_record_type(&alias.ty, &mut records),
            Item::Newtype(newtype) => collect_record_type(&newtype.inner, &mut records),
            Item::Enum(_) | Item::Error(_) | Item::Extension(_) => {}
        }
    }
    let interfaces: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Interface(interface) => Some(interface.name.clone()),
            _ => None,
        })
        .collect();
    let async_interfaces: HashSet<String> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Interface(interface)
                if interface.methods.iter().any(|method| method.is_async) =>
            {
                Some(interface.name.clone())
            }
            _ => None,
        })
        .collect();
    let call_signatures: HashMap<String, Vec<Param>> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Function(function) => Some((function.name.clone(), function.params.clone())),
            _ => None,
        })
        .collect();
    let has_async_interface = program.items.iter().any(|item| {
        matches!(&item.node, Item::Interface(interface) if interface.methods.iter().any(|method| method.is_async))
    });
    let has_await_for = program.items.iter().any(|item| match &item.node {
        Item::Class(class) => class
            .methods
            .iter()
            .any(|function| statements_have_await_for(&function.body)),
        Item::Function(function) => statements_have_await_for(&function.body),
        Item::Interface(_)
        | Item::Enum(_)
        | Item::Error(_)
        | Item::TypeAlias(_)
        | Item::Newtype(_) => false,
        Item::Extension(extension) => extension
            .methods
            .iter()
            .any(|function| statements_have_await_for(&function.body)),
    });
    if program
        .items
        .iter()
        .any(|item| matches!(&item.node, Item::Function(function) if function.is_async))
    {
        out.push_str("use orust_runtime::*;\n");
    }
    if has_async_interface {
        out.push_str("use orust_runtime::async_trait;\n");
    }
    if has_await_for {
        out.push_str("use orust_runtime::StreamExt;\n");
    }
    if include_record_module {
        out.push_str(&record_module_code(&records, &interfaces, uses_spawn));
    }
    for (item_index, item) in program.items.iter().enumerate() {
        if let Item::Function(function) = &item.node {
            if let Some(cfg) = program
                .attributes
                .get(item_index)
                .and_then(|attributes| attributes.cfg.as_deref())
            {
                out.push_str(&format!("#[cfg({cfg})]\n"));
            }
            emit_parameter_struct(&mut out, function, &interfaces, uses_spawn);
        }
    }
    let mut method_mutability: HashMap<String, HashMap<String, bool>> = HashMap::new();
    for item in &program.items {
        if let Item::Class(class) = &item.node {
            let methods = class
                .methods
                .iter()
                .map(|method| (method.name.clone(), function_needs_mut(method)))
                .collect();
            method_mutability.insert(class.name.clone(), methods);
        }
    }
    for (item_index, item) in program.items.iter().enumerate() {
        if let Some(attributes) = program.attributes.get(item_index) {
            emit_item_attributes(
                &mut out,
                attributes,
                !matches!(item.node, Item::Class(_) | Item::Newtype(_)),
            );
        }
        match &item.node {
            Item::Class(c) => {
                if c.shared {
                    emit_shared_class(&mut out, c, uses_spawn, &interfaces);
                    continue;
                }
                let mut derives = vec!["Debug"];
                if let Some(attributes) = program.attributes.get(item_index) {
                    for derive in &attributes.derives {
                        if !derives.iter().any(|existing| existing == derive) {
                            derives.push(derive.as_str());
                        }
                    }
                }
                let inferred_clone = uses_copy
                    && c.fields
                        .iter()
                        .all(|field| cloneable_type(&field.ty, &class_names));
                if c.data || inferred_clone || c.implements.iter().any(|name| name == "Clone") {
                    derives.push("Clone");
                }
                if c.data {
                    derives.push("PartialEq");
                }
                let comparable = c
                    .implements
                    .iter()
                    .any(|name| name == "Comparable" || name.starts_with("Comparable<"));
                if comparable {
                    if !c.data {
                        derives.push("PartialEq");
                    }
                    derives.push("Eq");
                }
                let hashable = c.implements.iter().any(|name| name == "Hashable");
                if hashable {
                    if !c.data && !comparable {
                        derives.extend(["PartialEq", "Eq"]);
                    }
                    derives.push("Hash");
                }
                out.push_str(&format!("#[derive({})]\n", derives.join(", ")));
                let generic_decl = class_generic_decl(c);
                let generic_use = class_generic_use(c);
                out.push_str(&format!(
                    "{}struct {}{} {{\n",
                    if c.exported { "pub " } else { "" },
                    declared_name(&c.name, &c.rust_name),
                    generic_decl
                ));
                for f in &c.fields {
                    out.push_str(&format!(
                        "    {}{}: {},\n",
                        visibility_prefix(f.visibility, c.exported),
                        f.name,
                        class_field_type(&f.ty, &interfaces, uses_spawn)
                    ));
                }
                out.push_str("}\n");
                if !c.constructors.is_empty() {
                    for constructor in &c.constructors {
                        if constructor
                            .params
                            .iter()
                            .any(|parameter| parameter.default.is_some())
                        {
                            emit_constructor_args(
                                &mut out,
                                c,
                                constructor,
                                &interfaces,
                                uses_spawn,
                            );
                        }
                    }
                }
                out.push_str("impl");
                out.push_str(&generic_decl);
                out.push(' ');
                out.push_str(&declared_name(&c.name, &c.rust_name));
                out.push_str(&generic_use);
                out.push_str(" {\n");
                if c.constructors.is_empty() {
                    out.push_str(&format!(
                        "    {}fn new() -> Self {{ Self {{\n",
                        if c.exported { "pub " } else { "" }
                    ));
                    for f in &c.fields {
                        let value = f
                            .initializer
                            .as_ref()
                            .map(expr)
                            .unwrap_or_else(|| "Default::default()".into());
                        out.push_str(&format!(
                            "        {}: {},\n",
                            f.name,
                            box_field_value(&f.ty, value)
                        ));
                    }
                    out.push_str("    } }\n");
                } else {
                    for constructor in &c.constructors {
                        emit_constructor(
                            &mut out,
                            c,
                            constructor,
                            &interfaces,
                            uses_spawn,
                            &call_signatures,
                        );
                    }
                }
                let field_names: Vec<String> =
                    c.fields.iter().map(|field| field.name.clone()).collect();
                if c.implements.is_empty()
                    || c.implements.iter().all(|name| {
                        name == "Clone"
                            || name == "Hashable"
                            || name == "Default"
                            || name == "Comparable"
                            || name.starts_with("Comparable<")
                    })
                {
                    for m in &c.methods {
                        let mut method = m.clone();
                        method.exported = c.exported;
                        emit_function(
                            &mut out,
                            &method,
                            true,
                            Some(&field_names),
                            &method_mutability,
                            &interfaces,
                            uses_spawn,
                            &call_signatures,
                            false,
                        );
                    }
                }
                out.push_str("}\n");
                if let Some(drop_body) = &c.drop_body {
                    let drop_function = Function {
                        exported: false,
                        visibility: orust_syntax::Visibility::Private,
                        rust_name: None,
                        rust_import: None,
                        return_type: "void".into(),
                        name: "drop".into(),
                        generics: Vec::new(),
                        bounds: Vec::new(),
                        is_async: false,
                        throws: false,
                        throws_type: None,
                        test_name: None,
                        params: Vec::new(),
                        body: drop_body.clone(),
                        span: Span { start: 0, end: 0 },
                    };
                    out.push_str(&format!(
                        "impl{} Drop for {}{} {{\n",
                        generic_decl,
                        declared_name(&c.name, &c.rust_name),
                        generic_use
                    ));
                    emit_function(
                        &mut out,
                        &drop_function,
                        true,
                        Some(&field_names),
                        &method_mutability,
                        &interfaces,
                        uses_spawn,
                        &call_signatures,
                        true,
                    );
                    out.push_str("}\n");
                }
                for trait_name in &c.implements {
                    if matches!(trait_name.as_str(), "Clone" | "Hashable")
                        || trait_name == "Comparable"
                        || trait_name.starts_with("Comparable<")
                    {
                        if trait_name == "Comparable" || trait_name.starts_with("Comparable<") {
                            out.push_str(&format!(
                                "impl{} Ord for {}{} {{\n    fn cmp(&self, other: &Self) -> std::cmp::Ordering {{ self.compareTo(other).cmp(&0) }}\n}}\nimpl{} PartialOrd for {}{} {{\n    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {{ Some(self.cmp(other)) }}\n}}\n",
                                generic_decl,
                                declared_name(&c.name, &c.rust_name),
                                generic_use,
                                generic_decl,
                                declared_name(&c.name, &c.rust_name),
                                generic_use
                            ));
                        }
                        continue;
                    }
                    if let Some(iterable_item) = trait_name
                        .strip_prefix("Iterable<")
                        .and_then(|value| value.strip_suffix('>'))
                    {
                        let item = rust_type_for(iterable_item, &interfaces, uses_spawn);
                        out.push_str(&format!(
                            "impl<'a> IntoIterator for &'a {}{} {{\n",
                            declared_name(&c.name, &c.rust_name),
                            generic_use
                        ));
                        out.push_str(&format!(
                            "    type Item = {item};\n    type IntoIter = Box<dyn Iterator<Item = {item}> + 'a>;\n    fn into_iter(self) -> Self::IntoIter {{ Box::new(self.iterator()) }}\n}}\n"
                        ));
                        continue;
                    }
                    if trait_name == "Default" {
                        out.push_str(&format!(
                            "impl{} Default for {}{} {{\n    fn default() -> Self {{ Self::new() }}\n}}\n",
                            generic_decl,
                            declared_name(&c.name, &c.rust_name),
                            generic_use
                        ));
                        continue;
                    }
                    if let Some(source_type) = trait_name
                        .strip_prefix("From<")
                        .and_then(|value| value.strip_suffix('>'))
                    {
                        let source_type = rust_type_for(source_type, &interfaces, uses_spawn);
                        out.push_str(&format!(
                            "impl{} From<{}> for {}{} {{\n",
                            generic_decl,
                            source_type,
                            declared_name(&c.name, &c.rust_name),
                            generic_use
                        ));
                        out.push_str(&format!(
                            "    fn from(value: {source_type}) -> Self {{ Self::from(value) }}\n}}\n"
                        ));
                        continue;
                    }
                    let iterator_item = trait_name
                        .strip_prefix("Iterator<")
                        .and_then(|value| value.strip_suffix('>'));
                    if iterator_item.is_some() {
                        out.push_str(&format!(
                            "impl{} Iterator for {}{} {{\n",
                            generic_decl,
                            declared_name(&c.name, &c.rust_name),
                            generic_use
                        ));
                        out.push_str(&format!(
                            "    type Item = {};\n",
                            rust_type_for(iterator_item.unwrap_or("void"), &interfaces, uses_spawn)
                        ));
                    } else {
                        if async_interfaces.contains(trait_name) {
                            out.push_str("#[async_trait]\n");
                        }
                        out.push_str(&format!(
                            "impl{} {} for {}{} {{\n",
                            generic_decl, trait_name, c.name, generic_use
                        ));
                    }
                    let methods: Vec<&Function> = if iterator_item.is_some() {
                        c.methods
                            .iter()
                            .filter(|method| method.name == "next")
                            .collect()
                    } else if trait_name
                        .strip_prefix("Iterable<")
                        .and_then(|value| value.strip_suffix('>'))
                        .is_some()
                    {
                        c.methods
                            .iter()
                            .filter(|method| method.name == "iterator")
                            .collect()
                    } else if let Some(interface) =
                        program.items.iter().find_map(|item| match &item.node {
                            Item::Interface(interface) if interface.name == *trait_name => {
                                Some(interface)
                            }
                            _ => None,
                        })
                    {
                        for (associated_name, associated_type) in &c.associated_types {
                            if interface
                                .associated_types
                                .iter()
                                .any(|expected| expected == associated_name)
                            {
                                out.push_str(&format!(
                                    "    type {associated_name} = {};\n",
                                    rust_type_for(associated_type, &interfaces, uses_spawn)
                                ));
                            }
                        }
                        c.methods
                            .iter()
                            .filter(|method| {
                                interface
                                    .methods
                                    .iter()
                                    .any(|expected| expected.name == method.name)
                            })
                            .collect()
                    } else {
                        c.methods.iter().collect()
                    };
                    for m in methods {
                        emit_function(
                            &mut out,
                            m,
                            true,
                            Some(&field_names),
                            &method_mutability,
                            &interfaces,
                            uses_spawn,
                            &call_signatures,
                            false,
                        );
                    }
                    out.push_str("}\n");
                }
                emit_operator_impls(&mut out, c, &interfaces, uses_spawn);
                if c.methods.iter().any(|method| method.name == "toString") {
                    emit_display_impl(&mut out, c);
                }
            }
            Item::Function(f) if f.test_name.is_some() => {}
            Item::Function(f) => {
                if f.is_async && f.name == "main" {
                    out.push_str("#[tokio::main]\n");
                }
                emit_function(
                    &mut out,
                    f,
                    false,
                    None,
                    &method_mutability,
                    &interfaces,
                    uses_spawn,
                    &call_signatures,
                    false,
                );
            }
            Item::Interface(interface) => {
                if interface.methods.iter().any(|method| method.is_async) {
                    out.push_str("#[async_trait]\n");
                }
                let mut supertraits = interface.extends.clone();
                if uses_spawn {
                    supertraits.push("Send + Sync".into());
                }
                let trait_bounds = if supertraits.is_empty() {
                    String::new()
                } else {
                    format!(": {}", supertraits.join(" + "))
                };
                out.push_str(&format!(
                    "{}trait {}{} {{\n",
                    if interface.exported { "pub " } else { "" },
                    declared_name(&interface.name, &interface.rust_name),
                    trait_bounds
                ));
                for associated_type in &interface.associated_types {
                    out.push_str(&format!("    type {associated_type};\n"));
                }
                for method in &interface.methods {
                    out.push_str(&format!(
                        "    {}fn {}(&self",
                        if method.is_async { "async " } else { "" },
                        method.name
                    ));
                    for parameter in &method.params {
                        out.push_str(&format!(
                            ", {}: {}",
                            parameter.name,
                            rust_type_for(&parameter.ty, &interfaces, uses_spawn)
                        ));
                    }
                    out.push_str(&format!(
                        ") -> {}",
                        rust_type_for(&method.return_type, &interfaces, uses_spawn)
                    ));
                    if method.body.is_empty() {
                        out.push_str(";\n");
                    } else {
                        out.push_str(" {\n");
                        emit_block(&mut out, &method.body, 2);
                        out.push_str("    }\n");
                    }
                }
                out.push_str("}\n");
            }
            Item::Error(error) => emit_error(&mut out, error, &interfaces, uses_spawn),
            Item::Enum(enum_decl) => emit_enum(&mut out, enum_decl, &interfaces, uses_spawn),
            Item::Extension(extension) => emit_extension(
                &mut out,
                extension,
                &interfaces,
                uses_spawn,
                &method_mutability,
            ),
            Item::TypeAlias(alias) => {
                if let Some(path) = &alias.rust_import {
                    out.push_str(&format!(
                        "{}use {} as {};\n",
                        if alias.exported { "pub " } else { "" },
                        path,
                        declared_name(&alias.name, &alias.rust_name)
                    ));
                } else {
                    out.push_str(&format!(
                        "{}type {}{} = {};\n",
                        if alias.exported { "pub " } else { "" },
                        declared_name(&alias.name, &alias.rust_name),
                        generic_decl(&alias.generics, &[]),
                        rust_type_for(&alias.ty, &interfaces, uses_spawn)
                    ));
                }
            }
            Item::Newtype(newtype) => {
                let inner = rust_type_for(&newtype.inner, &interfaces, uses_spawn);
                let derives = if matches!(
                    newtype.inner.as_str(),
                    "int" | "bool" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64"
                ) {
                    "Debug, Clone, PartialEq, Eq, Hash, Copy"
                } else if newtype.inner == "double" {
                    "Debug, Clone, PartialEq"
                } else {
                    "Debug, Clone, PartialEq, Eq, Hash"
                };
                let mut derive_list = derives.split(", ").map(str::to_string).collect::<Vec<_>>();
                if let Some(attributes) = program.attributes.get(item_index) {
                    for derive in &attributes.derives {
                        if !derive_list.iter().any(|existing| existing == derive) {
                            derive_list.push(derive.clone());
                        }
                    }
                }
                out.push_str(&format!(
                    "#[derive({})]\n{}struct {}(pub {});\n",
                    derive_list.join(", "),
                    if newtype.exported { "pub " } else { "" },
                    declared_name(&newtype.name, &newtype.rust_name),
                    inner
                ));
                out.push_str(&format!(
                    "impl {} {{ pub fn value(&self) -> {} {{ {} }} }}\n",
                    declared_name(&newtype.name, &newtype.rust_name),
                    if matches!(
                        newtype.inner.as_str(),
                        "bool"
                            | "int"
                            | "double"
                            | "u8"
                            | "u16"
                            | "u32"
                            | "u64"
                            | "i8"
                            | "i16"
                            | "i32"
                            | "i64"
                    ) {
                        inner.clone()
                    } else {
                        format!("&{inner}")
                    },
                    if matches!(
                        newtype.inner.as_str(),
                        "bool"
                            | "int"
                            | "double"
                            | "u8"
                            | "u16"
                            | "u32"
                            | "u64"
                            | "i8"
                            | "i16"
                            | "i32"
                            | "i64"
                    ) {
                        "self.0"
                    } else {
                        "&self.0"
                    }
                ));
            }
        }
    }
    let tests: Vec<&Function> = program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Function(function) if function.test_name.is_some() => Some(function),
            _ => None,
        })
        .collect();
    if !tests.is_empty() {
        out.push_str("#[cfg(test)]\nmod tests {\n    use super::*;\n");
        for test in tests {
            if test.is_async {
                out.push_str("    #[tokio::test]\n");
            } else {
                out.push_str("    #[test]\n");
            }
            let mut rendered = String::new();
            emit_function(
                &mut rendered,
                test,
                false,
                None,
                &method_mutability,
                &interfaces,
                uses_spawn,
                &call_signatures,
                false,
            );
            for line in rendered.lines() {
                out.push_str("    ");
                out.push_str(line);
                out.push('\n');
            }
        }
        out.push_str("}\n");
    }
    out
}

fn emit_rust_uses(out: &mut String, program: &Program) {
    for rust_use in &program.rust_uses {
        out.push_str("use ");
        out.push_str(rust_use);
        out.push_str(";\n");
    }
}

fn emit_std_imports(out: &mut String, program: &Program) {
    for import in &program.imports {
        let Some(module) = import.path.strip_prefix("std:") else {
            continue;
        };
        let rust_module = match module {
            "math" => "std_math",
            "env" => "std_env",
            "io" => "std_io",
            _ => continue,
        };
        let names = if import.show.is_empty() {
            "*".into()
        } else {
            import
                .show
                .iter()
                .filter(|name| !import.hide.iter().any(|hidden| hidden == *name))
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        };
        out.push_str(&format!("use orust_runtime::{rust_module}::{{{names}}};\n"));
    }
}

fn emit_item_attributes(
    out: &mut String,
    attributes: &orust_syntax::ItemAttributes,
    include_derive: bool,
) {
    if let Some(cfg) = &attributes.cfg {
        out.push_str(&format!("#[cfg({cfg})]\n"));
    }
    if let Some(repr) = &attributes.repr {
        out.push_str(&format!("#[repr({repr})]\n"));
    }
    if include_derive && !attributes.derives.is_empty() {
        out.push_str(&format!("#[derive({})]\n", attributes.derives.join(", ")));
    }
    if attributes.unsafe_rust {
        out.push_str("// ORust declaration permits an unsafe Rust implementation.\n");
    }
    if let Some(lifetime) = &attributes.borrowed_lifetime {
        out.push_str(&format!(
            "// ORust borrowed boundary: lifetime {lifetime} is enforced by rustc.\n"
        ));
    }
    if attributes.owned {
        out.push_str("// ORust owned boundary: Rust move semantics are authoritative.\n");
    }
}

fn emit_display_impl(out: &mut String, class: &orust_syntax::Class) {
    out.push_str(&format!(
        "impl{} std::fmt::Display for {}{} {{\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n        write!(f, \"{{}}\", self.toString())\n    }}\n}}\n",
        class_generic_decl(class),
        declared_name(&class.name, &class.rust_name),
        class_generic_use(class)
    ));
}

fn emit_error(
    out: &mut String,
    error: &orust_syntax::Error,
    interfaces: &HashSet<String>,
    with_spawn: bool,
) {
    let name = declared_name(&error.name, &error.rust_name);
    if !error.cases.is_empty() {
        out.push_str(&format!(
            "#[derive(Debug)]\n{}enum {} {{\n",
            if error.exported { "pub " } else { "" },
            name
        ));
        for case in &error.cases {
            if case.fields.is_empty() {
                out.push_str(&format!("    {},\n", case.name));
            } else {
                out.push_str(&format!("    {} {{\n", case.name));
                for field in &case.fields {
                    out.push_str(&format!(
                        "        {}: {},\n",
                        field.name,
                        rust_type_for(&field.ty, interfaces, with_spawn)
                    ));
                }
                out.push_str("    },\n");
            }
        }
        out.push_str("}\n");
        out.push_str(&format!(
            "impl std::fmt::Display for {name} {{\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n        match self {{\n"
        ));
        for case in &error.cases {
            let pattern = if case.fields.is_empty() {
                format!("{name}::{}", case.name)
            } else {
                format!("{name}::{} {{ .. }}", case.name)
            };
            let message = case.message.as_deref().unwrap_or(&case.name);
            out.push_str(&format!(
                "            {pattern} => write!(f, \"{message}\"),\n"
            ));
        }
        out.push_str("        }\n    }\n}\n");
        out.push_str(&format!("impl std::error::Error for {name} {{}}\n"));
        return;
    }
    out.push_str(&format!(
        "#[derive(Debug)]\n{}struct {} {{\n",
        if error.exported { "pub " } else { "" },
        name
    ));
    for field in &error.fields {
        out.push_str(&format!(
            "    pub {}: {},\n",
            field.name,
            rust_type_for(&field.ty, interfaces, with_spawn)
        ));
    }
    out.push_str("}\n");
    out.push_str(&format!(
        "impl std::fmt::Display for {name} {{\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n        write!(f, \"{name}\")\n    }}\n}}\n"
    ));
    out.push_str(&format!("impl std::error::Error for {name} {{}}\n"));
}

fn emit_parameter_struct(
    out: &mut String,
    function: &Function,
    interfaces: &HashSet<String>,
    with_spawn: bool,
) {
    if function
        .params
        .iter()
        .all(|parameter| parameter.default.is_none())
    {
        return;
    }
    out.push_str("#[derive(Default)]\n");
    out.push_str(&format!(
        "{}struct {}Args {{\n",
        visibility_prefix(function.visibility, function.exported),
        declared_name(&function.name, &function.rust_name)
    ));
    for parameter in &function.params {
        out.push_str(&format!(
            "    {}: Option<{}>,\n",
            parameter.name,
            function_type(&parameter.ty, false, interfaces, with_spawn)
        ));
    }
    out.push_str("}\n");
}

fn emit_operator_impls(
    out: &mut String,
    class: &orust_syntax::Class,
    interfaces: &HashSet<String>,
    with_spawn: bool,
) {
    let operators = [
        ("operator_add", "std::ops::Add", "add"),
        ("operator_sub", "std::ops::Sub", "sub"),
        ("operator_mul", "std::ops::Mul", "mul"),
        ("operator_div", "std::ops::Div", "div"),
    ];
    for (method_name, trait_name, rust_name) in operators {
        let Some(method) = class
            .methods
            .iter()
            .find(|method| method.name == method_name)
        else {
            continue;
        };
        let Some(parameter) = method.params.first() else {
            continue;
        };
        out.push_str(&format!(
            "impl{} {} for {}{} {{\n    type Output = {};\n    fn {}(self, rhs: {}) -> Self::Output {{\n        self.{}(rhs)\n    }}\n}}\n",
            class_generic_decl(class),
            trait_name,
            declared_name(&class.name, &class.rust_name),
            class_generic_use(class),
            function_type(&method.return_type, false, interfaces, with_spawn),
            rust_name,
            function_type(&parameter.ty, false, interfaces, with_spawn),
            method_name
        ));
    }
    if let Some(method) = class
        .methods
        .iter()
        .find(|method| method.name == "operator_eq")
    {
        if let Some(parameter) = method.params.first() {
            out.push_str(&format!(
                "impl{} PartialEq for {}{} {{\n    fn eq(&self, rhs: &Self) -> bool {{ self.operator_eq(rhs) }}\n}}\n",
                class_generic_decl(class),
                declared_name(&class.name, &class.rust_name),
                class_generic_use(class)
            ));
            let _ = parameter;
        }
    }
    if class
        .methods
        .iter()
        .any(|method| method.name == "operator_lt")
        || class
            .methods
            .iter()
            .any(|method| method.name == "operator_gt")
    {
        let less = if class
            .methods
            .iter()
            .any(|method| method.name == "operator_lt")
        {
            "if self.operator_lt(rhs) { Some(std::cmp::Ordering::Less) } else "
        } else {
            ""
        };
        let greater = if class
            .methods
            .iter()
            .any(|method| method.name == "operator_gt")
        {
            "if self.operator_gt(rhs) { Some(std::cmp::Ordering::Greater) } else "
        } else {
            ""
        };
        let equal = if class
            .methods
            .iter()
            .any(|method| method.name == "operator_eq")
        {
            "if self.operator_eq(rhs) { Some(std::cmp::Ordering::Equal) } else "
        } else {
            ""
        };
        out.push_str(&format!(
            "impl{} std::cmp::PartialOrd for {}{} {{\n    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {{ {}{}{}None }}\n}}\n",
            class_generic_decl(class),
            declared_name(&class.name, &class.rust_name),
            class_generic_use(class),
            less,
            greater,
            equal
        ));
    }
    if let Some(method) = class
        .methods
        .iter()
        .find(|method| method.name == "operator_index")
    {
        if let Some(parameter) = method.params.first() {
            let output_type = method
                .return_type
                .strip_prefix('&')
                .map(|ty| function_type(ty, false, interfaces, with_spawn))
                .unwrap_or_else(|| {
                    function_type(&method.return_type, false, interfaces, with_spawn)
                });
            out.push_str(&format!(
                "impl{} std::ops::Index<{}> for {}{} {{\n    type Output = {};\n    fn index(&self, index: {}) -> &Self::Output {{ self.operator_index(index) }}\n}}\n",
                class_generic_decl(class),
                function_type(&parameter.ty, false, interfaces, with_spawn),
                declared_name(&class.name, &class.rust_name),
                class_generic_use(class),
                output_type,
                function_type(&parameter.ty, false, interfaces, with_spawn)
            ));
        }
    }
}

fn emit_enum(
    out: &mut String,
    enum_decl: &orust_syntax::Enum,
    interfaces: &HashSet<String>,
    uses_spawn: bool,
) {
    out.push_str(&format!(
        "{}enum {}{} {{\n",
        if enum_decl.exported { "pub " } else { "" },
        declared_name(&enum_decl.name, &enum_decl.rust_name),
        generic_decl(&enum_decl.generics, &enum_decl.bounds)
    ));
    for variant in &enum_decl.variants {
        if variant.fields.is_empty() {
            out.push_str(&format!("    {},\n", variant.name));
        } else {
            out.push_str(&format!("    {}(", variant.name));
            out.push_str(
                &variant
                    .fields
                    .iter()
                    .map(|field| rust_type_for(field, interfaces, uses_spawn))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            out.push_str("),\n");
        }
    }
    out.push_str("}\n");
}

fn emit_extension(
    out: &mut String,
    extension: &orust_syntax::Extension,
    interfaces: &HashSet<String>,
    uses_spawn: bool,
    method_mutability: &HashMap<String, HashMap<String, bool>>,
) {
    let trait_name = format!("{}Extensions", extension.target);
    out.push_str(&format!("trait {trait_name} {{\n"));
    for method in &extension.methods {
        out.push_str(&format!("    fn {}(&self", method.name));
        for parameter in &method.params {
            out.push_str(&format!(
                ", {}: {}",
                parameter.name,
                function_type(&parameter.ty, false, interfaces, uses_spawn)
            ));
        }
        out.push_str(&format!(
            ") -> {};\n",
            function_type(&method.return_type, false, interfaces, uses_spawn)
        ));
    }
    out.push_str(&format!(
        "}}\nimpl {trait_name} for {} {{\n",
        extension.target
    ));
    let signatures = HashMap::new();
    for method in &extension.methods {
        emit_function(
            out,
            method,
            true,
            None,
            method_mutability,
            interfaces,
            uses_spawn,
            &signatures,
            false,
        );
    }
    out.push_str("}\n");
}

fn emit_shared_class(
    out: &mut String,
    class: &orust_syntax::Class,
    uses_spawn: bool,
    interfaces: &HashSet<String>,
) {
    if uses_spawn {
        out.push_str("use std::sync::Arc;\nuse orust_runtime::tokio::sync::Mutex;\n");
    } else {
        out.push_str("use std::cell::RefCell;\nuse std::rc::Rc;\n");
    }
    out.push_str(&format!(
        "{}struct {}Data {{\n",
        if class.exported { "pub " } else { "" },
        class.name
    ));
    for field in &class.fields {
        out.push_str(&format!(
            "    {}: {},\n",
            field.name,
            rust_type_for(&field.ty, interfaces, uses_spawn)
        ));
    }
    out.push_str("}\n");
    if uses_spawn {
        out.push_str(&format!(
            "{}struct {}(Arc<Mutex<{}Data>>);\n",
            if class.exported { "pub " } else { "" },
            class.name,
            class.name
        ));
        out.push_str(&format!(
            "impl {} {{\n    {}fn new() -> Self {{ Self(Arc::new(Mutex::new({}Data {{\n",
            class.name,
            if class.exported { "pub " } else { "" },
            class.name
        ));
    } else {
        out.push_str(&format!(
            "{}struct {}(Rc<RefCell<{}Data>>);\n",
            if class.exported { "pub " } else { "" },
            class.name,
            class.name
        ));
        out.push_str(&format!(
            "impl {} {{\n    {}fn new() -> Self {{ Self(Rc::new(RefCell::new({}Data {{\n",
            class.name,
            if class.exported { "pub " } else { "" },
            class.name
        ));
    }
    for field in &class.fields {
        let value = field
            .initializer
            .as_ref()
            .map(expr)
            .unwrap_or_else(|| "Default::default()".into());
        out.push_str(&format!("        {}: {},\n", field.name, value));
    }
    if uses_spawn {
        out.push_str("    }))) }\n    fn clone(&self) -> Self { Self(Arc::clone(&self.0)) }\n");
    } else {
        out.push_str("    }))) }\n    fn clone(&self) -> Self { Self(Rc::clone(&self.0)) }\n    fn borrow(&self) -> std::cell::Ref<'_, ");
        out.push_str(&format!("{}Data> {{ self.0.borrow() }}\n", class.name));
        out.push_str("    fn borrow_mut(&self) -> std::cell::RefMut<'_, ");
        out.push_str(&format!("{}Data> {{ self.0.borrow_mut() }}\n", class.name));
    }
    let field_names: Vec<String> = class
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect();
    for method in &class.methods {
        emit_shared_method(out, method, &field_names, uses_spawn, interfaces);
    }
    out.push_str("}\n");
}

fn emit_shared_method(
    out: &mut String,
    function: &Function,
    fields: &[String],
    uses_spawn: bool,
    interfaces: &HashSet<String>,
) {
    out.push_str(&format!(
        "    {}fn {}(&self",
        if function.is_async { "async " } else { "" },
        function.name
    ));
    for parameter in &function.params {
        out.push_str(&format!(
            ", {}: {}",
            parameter.name,
            rust_type_for(&parameter.ty, interfaces, uses_spawn)
        ));
    }
    out.push_str(&format!(
        ") -> {} {{\n",
        rust_type_for(&function.return_type, interfaces, uses_spawn)
    ));
    for statement in &function.body {
        match &statement.node {
            Stmt::Return(Some(value)) => out.push_str(&format!(
                "        return {};\n",
                render_shared_expr(value, fields, uses_spawn, function.is_async)
            )),
            Stmt::Return(None) => out.push_str("        return;\n"),
            Stmt::Expr(value) => out.push_str(&format!(
                "        {};\n",
                render_shared_expr(value, fields, uses_spawn, function.is_async)
            )),
            Stmt::Print(value) => out.push_str(&format!(
                "        println!(\"{{}}\", {});\n",
                render_shared_expr(value, fields, uses_spawn, function.is_async)
            )),
            Stmt::Var {
                name, initializer, ..
            } => out.push_str(&format!(
                "        let {} = {};\n",
                binding_source(name),
                render_shared_expr(initializer, fields, uses_spawn, function.is_async)
            )),
            _ => out.push_str("        // complex control flow is lowered in a later pass\n"),
        }
    }
    out.push_str("    }\n");
}

fn render_shared_expr(value: &Expr, fields: &[String], uses_spawn: bool, is_async: bool) -> String {
    let mut rendered = expr(value);
    for field in fields {
        let access = if uses_spawn {
            if is_async {
                "self.0.lock().await"
            } else {
                "self.0.blocking_lock()"
            }
        } else if rendered.contains(&format!("{field} =")) {
            "self.0.borrow_mut()"
        } else {
            "self.0.borrow()"
        };
        rendered = rendered.replace(field, &format!("{access}.{field}"));
    }
    rendered
}

fn program_uses_spawn(program: &Program) -> bool {
    program.items.iter().any(|item| match &item.node {
        Item::Class(class) => class
            .methods
            .iter()
            .any(|function| statements_use_spawn(&function.body)),
        Item::Function(function) => statements_use_spawn(&function.body),
        Item::Interface(_) | Item::Enum(_) | Item::Error(_) => false,
        Item::Extension(extension) => extension
            .methods
            .iter()
            .any(|function| statements_use_spawn(&function.body)),
        Item::TypeAlias(_) | Item::Newtype(_) => false,
    })
}

fn program_uses_copy(program: &Program) -> bool {
    fn expr_uses_copy(expression: &Expr) -> bool {
        match expression {
            Expr::Copy(_) => true,
            Expr::Call { callee, args } => {
                expr_uses_copy(callee) || args.iter().any(expr_uses_copy)
            }
            Expr::Member { object, .. }
            | Expr::OptionalMember { object, .. }
            | Expr::Index { object, .. }
            | Expr::Borrow { value: object, .. }
            | Expr::Await(object)
            | Expr::Spawn(object) => expr_uses_copy(object),
            Expr::Slice {
                object, start, end, ..
            } => {
                expr_uses_copy(object)
                    || start.as_deref().is_some_and(expr_uses_copy)
                    || end.as_deref().is_some_and(expr_uses_copy)
            }
            Expr::Binary { left, right, .. } | Expr::Coalesce { left, right } => {
                expr_uses_copy(left) || expr_uses_copy(right)
            }
            Expr::List(values) => values.iter().any(expr_uses_copy),
            Expr::NewArgs { args, .. } => args.iter().any(expr_uses_copy),
            Expr::NamedArg { value, .. } => expr_uses_copy(value),
            Expr::Closure { body, .. } => match body {
                orust_syntax::ClosureBody::Expr(value) => expr_uses_copy(value),
                orust_syntax::ClosureBody::Block(body) => statements_use_copy(body),
            },
            Expr::Int(_)
            | Expr::Float(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Name(_)
            | Expr::New(_)
            | Expr::Null
            | Expr::This => false,
        }
    }
    fn statements_use_copy(statements: &[orust_syntax::Spanned<Stmt>]) -> bool {
        statements.iter().any(|statement| match &statement.node {
            Stmt::Var { initializer, .. }
            | Stmt::PatternVar { initializer, .. }
            | Stmt::Print(initializer)
            | Stmt::Expr(initializer)
            | Stmt::Return(Some(initializer))
            | Stmt::Throw(initializer) => expr_uses_copy(initializer),
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                expr_uses_copy(condition)
                    || statements_use_copy(then_body)
                    || statements_use_copy(else_body)
            }
            Stmt::While { condition, body }
            | Stmt::WhileCase {
                value: condition,
                body,
                ..
            } => expr_uses_copy(condition) || statements_use_copy(body),
            Stmt::For {
                initializer,
                condition,
                step,
                body,
            } => {
                initializer.as_deref().is_some_and(|statement| {
                    statements_use_copy(std::slice::from_ref(&orust_syntax::Spanned {
                        node: statement.clone(),
                        span: Span { start: 0, end: 0 },
                    }))
                }) || condition.as_ref().is_some_and(expr_uses_copy)
                    || step.as_ref().is_some_and(expr_uses_copy)
                    || statements_use_copy(body)
            }
            Stmt::ForIn { iterable, body, .. }
            | Stmt::AwaitFor {
                stream: iterable,
                body,
                ..
            } => expr_uses_copy(iterable) || statements_use_copy(body),
            Stmt::TryCatch {
                body, catch_body, ..
            } => statements_use_copy(body) || statements_use_copy(catch_body),
            Stmt::Switch { value, cases } => {
                expr_uses_copy(value) || cases.iter().any(|case| statements_use_copy(&case.body))
            }
            Stmt::IfCase {
                value,
                then_body,
                else_body,
                ..
            } => {
                expr_uses_copy(value)
                    || statements_use_copy(then_body)
                    || statements_use_copy(else_body)
            }
            Stmt::Return(None) | Stmt::Rust(_) => false,
        })
    }
    program.items.iter().any(|item| match &item.node {
        Item::Class(class) => class
            .methods
            .iter()
            .any(|method| statements_use_copy(&method.body)),
        Item::Function(function) => statements_use_copy(&function.body),
        Item::Extension(extension) => extension
            .methods
            .iter()
            .any(|method| statements_use_copy(&method.body)),
        Item::Interface(_)
        | Item::Enum(_)
        | Item::Error(_)
        | Item::TypeAlias(_)
        | Item::Newtype(_) => false,
    })
}

fn cloneable_type(ty: &str, class_names: &HashSet<String>) -> bool {
    let ty = ty
        .trim()
        .trim_start_matches("lend ")
        .trim_start_matches("mut ");
    if let Some(inner) = ty.strip_suffix('?') {
        return cloneable_type(inner, class_names);
    }
    if let Some(inner) = ty.strip_prefix("List<").and_then(|v| v.strip_suffix('>')) {
        return cloneable_type(inner, class_names);
    }
    matches!(
        ty,
        "String"
            | "bool"
            | "int"
            | "double"
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
            | "f32"
            | "f64"
    ) || ty.starts_with('&')
        || class_names.contains(ty)
}

fn record_module_code(
    records: &HashMap<String, Vec<(String, String)>>,
    interfaces: &HashSet<String>,
    uses_spawn: bool,
) -> String {
    if records.is_empty() {
        return String::new();
    }
    let mut out = String::from("pub mod orust_records {\n");
    let mut declarations = records.values().collect::<Vec<_>>();
    declarations.sort_by_key(|fields| record_name(fields));
    for fields in declarations {
        let derives = if fields.iter().all(|(_, ty)| ty == "int" || ty == "bool") {
            "Debug, Clone, PartialEq, Eq, Hash, Copy"
        } else if fields.iter().all(|(_, ty)| ty != "double") {
            "Debug, Clone, PartialEq, Eq, Hash"
        } else {
            "Debug, Clone, PartialEq"
        };
        out.push_str(&format!(
            "    #[derive({derives})]\n    pub struct {} {{\n",
            record_name(fields)
        ));
        for (field, ty) in fields {
            out.push_str(&format!(
                "        pub {field}: {},\n",
                rust_type_for(ty, interfaces, uses_spawn)
            ));
        }
        out.push_str("    }\n");
    }
    out.push_str("}\n");
    out
}

fn collect_program_records(
    program: &Program,
    records: &mut HashMap<String, Vec<(String, String)>>,
) {
    for item in &program.items {
        match &item.node {
            Item::Class(class) => {
                for field in &class.fields {
                    collect_record_type(&field.ty, records);
                }
                for constructor in &class.constructors {
                    for parameter in &constructor.params {
                        collect_record_type(&parameter.ty, records);
                    }
                    collect_record_statements(&constructor.body, records);
                }
                for method in &class.methods {
                    collect_record_type(&method.return_type, records);
                    for parameter in &method.params {
                        collect_record_type(&parameter.ty, records);
                    }
                    collect_record_statements(&method.body, records);
                }
            }
            Item::Function(function) => {
                collect_record_type(&function.return_type, records);
                for parameter in &function.params {
                    collect_record_type(&parameter.ty, records);
                }
                collect_record_statements(&function.body, records);
            }
            Item::Interface(interface) => {
                for method in &interface.methods {
                    collect_record_type(&method.return_type, records);
                    for parameter in &method.params {
                        collect_record_type(&parameter.ty, records);
                    }
                }
            }
            Item::TypeAlias(alias) => collect_record_type(&alias.ty, records),
            Item::Newtype(newtype) => collect_record_type(&newtype.inner, records),
            Item::Enum(_) | Item::Error(_) | Item::Extension(_) => {}
        }
    }
}

/// Build the single record module used by a generated multi-file project.
pub fn emit_project_record_module(programs: &[&Program]) -> String {
    let mut records = HashMap::new();
    let mut interfaces = HashSet::new();
    let mut uses_spawn = false;
    for program in programs {
        collect_program_records(program, &mut records);
        uses_spawn |= program_uses_spawn(program);
        interfaces.extend(program.items.iter().filter_map(|item| match &item.node {
            Item::Interface(interface) => Some(interface.name.clone()),
            _ => None,
        }));
    }
    record_module_code(&records, &interfaces, uses_spawn)
}

fn statements_have_await_for(statements: &[orust_syntax::Spanned<Stmt>]) -> bool {
    statements.iter().any(|statement| match &statement.node {
        Stmt::AwaitFor { .. } => true,
        Stmt::PatternVar { else_body, .. } => statements_have_await_for(else_body),
        Stmt::If {
            then_body,
            else_body,
            ..
        }
        | Stmt::IfCase {
            then_body,
            else_body,
            ..
        } => statements_have_await_for(then_body) || statements_have_await_for(else_body),
        Stmt::While { body, .. }
        | Stmt::WhileCase { body, .. }
        | Stmt::For { body, .. }
        | Stmt::ForIn { body, .. } => statements_have_await_for(body),
        Stmt::TryCatch {
            body, catch_body, ..
        } => statements_have_await_for(body) || statements_have_await_for(catch_body),
        Stmt::Switch { cases, .. } => cases
            .iter()
            .any(|case| statements_have_await_for(&case.body)),
        _ => false,
    })
}

fn statements_use_spawn(statements: &[orust_syntax::Spanned<Stmt>]) -> bool {
    statements.iter().any(|statement| match &statement.node {
        Stmt::PatternVar {
            initializer,
            else_body,
            ..
        } => expr_uses_spawn(initializer) || statements_use_spawn(else_body),
        Stmt::Var { initializer, .. }
        | Stmt::Print(initializer)
        | Stmt::Expr(initializer)
        | Stmt::Return(Some(initializer)) => expr_uses_spawn(initializer),
        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_spawn(condition)
                || statements_use_spawn(then_body)
                || statements_use_spawn(else_body)
        }
        Stmt::While { condition, body } => expr_uses_spawn(condition) || statements_use_spawn(body),
        Stmt::WhileCase { value, body, .. } => expr_uses_spawn(value) || statements_use_spawn(body),
        Stmt::For { body, .. } => statements_use_spawn(body),
        Stmt::ForIn { iterable, body, .. } => {
            expr_uses_spawn(iterable) || statements_use_spawn(body)
        }
        Stmt::TryCatch {
            body, catch_body, ..
        } => statements_use_spawn(body) || statements_use_spawn(catch_body),
        Stmt::Switch { value, cases } => {
            expr_uses_spawn(value) || cases.iter().any(|case| statements_use_spawn(&case.body))
        }
        Stmt::IfCase {
            value,
            then_body,
            else_body,
            ..
        } => {
            expr_uses_spawn(value)
                || statements_use_spawn(then_body)
                || statements_use_spawn(else_body)
        }
        Stmt::AwaitFor { body, .. } => statements_use_spawn(body),
        Stmt::Return(None) | Stmt::Rust(_) => false,
        Stmt::Throw(value) => expr_uses_spawn(value),
    })
}

fn expr_uses_spawn(expression: &Expr) -> bool {
    match expression {
        Expr::Spawn(_) => true,
        Expr::Await(value)
        | Expr::Borrow { value, .. }
        | Expr::Copy(value)
        | Expr::OptionalMember { object: value, .. } => expr_uses_spawn(value),
        Expr::Call { callee, args } => expr_uses_spawn(callee) || args.iter().any(expr_uses_spawn),
        Expr::NamedArg { value, .. } => expr_uses_spawn(value),
        Expr::Member { object, .. } => expr_uses_spawn(object),
        Expr::Binary { left, right, .. } | Expr::Coalesce { left, right } => {
            expr_uses_spawn(left) || expr_uses_spawn(right)
        }
        Expr::List(values) => values.iter().any(expr_uses_spawn),
        Expr::Closure { body, .. } => match body {
            ClosureBody::Expr(value) => expr_uses_spawn(value),
            ClosureBody::Block(statements) => statements_use_spawn(statements),
        },
        Expr::NewArgs { args, .. } => args.iter().any(expr_uses_spawn),
        Expr::Index { object, index } => expr_uses_spawn(object) || expr_uses_spawn(index),
        Expr::Slice {
            object, start, end, ..
        } => {
            expr_uses_spawn(object)
                || start.as_deref().is_some_and(expr_uses_spawn)
                || end.as_deref().is_some_and(expr_uses_spawn)
        }
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Bool(_)
        | Expr::String(_)
        | Expr::Name(_)
        | Expr::New(_)
        | Expr::Null
        | Expr::This => false,
    }
}
fn rust_type(t: &str) -> &str {
    match t {
        "void" => "()",
        "int" => "i64",
        "double" => "f64",
        "String" => "String",
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" | "bool" => t,
        t if t.starts_with("Future<") && t.ends_with('>') => rust_type(&t[7..t.len() - 1]),
        t if t.ends_with('?') => {
            Box::leak(format!("Option<{}>", rust_type(&t[..t.len() - 1])).into_boxed_str())
        }
        t if t.starts_with('&') => t,
        other => other,
    }
}

fn generic_decl(names: &[String], bounds: &[String]) -> String {
    if names.is_empty() {
        return String::new();
    }
    let bound_names: HashSet<&str> = bounds
        .iter()
        .filter_map(|bound| bound.split_once(':').map(|(name, _)| name.trim()))
        .collect();
    let values = names
        .iter()
        .map(|name| {
            if bound_names.contains(name.as_str()) {
                bounds
                    .iter()
                    .find(|bound| bound.starts_with(&format!("{name}:")))
                    .cloned()
                    .unwrap_or_else(|| name.clone())
            } else {
                name.clone()
            }
        })
        .collect::<Vec<_>>();
    format!("<{}>", values.join(", "))
}

fn generic_use(names: &[String]) -> String {
    if names.is_empty() {
        String::new()
    } else {
        format!("<{}>", names.join(", "))
    }
}

fn rust_type_for(t: &str, interfaces: &HashSet<String>, with_spawn: bool) -> String {
    if let Some(args) = t
        .strip_prefix("function(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let rendered = args
            .split(',')
            .filter(|arg| !arg.is_empty())
            .map(|arg| rust_type_for(arg, interfaces, with_spawn))
            .collect::<Vec<_>>()
            .join(", ");
        return format!("Box<dyn Fn({rendered}) -> ()>");
    }
    if let Some(fields) = record_fields_from_type(t) {
        return format!("crate::orust_records::{}", record_name(&fields));
    }
    if let Some(inner) = t
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        let parts = inner
            .split(", ")
            .map(|part| rust_type_for(part, interfaces, with_spawn))
            .collect::<Vec<_>>();
        return format!("({})", parts.join(", "));
    }
    if let Some(inner) = t.strip_prefix('&') {
        let (lifetime, inner) = if inner.starts_with('\'') {
            inner
                .split_once(' ')
                .map(|(lifetime, inner)| (Some(format!("{lifetime} ")), inner))
                .unwrap_or((None, inner))
        } else {
            (None, inner)
        };
        let (mutable, inner) = if let Some(inner) = inner.strip_prefix("mut ") {
            (true, inner)
        } else {
            (false, inner)
        };
        if inner == "String" {
            return format!(
                "&{}{}str",
                lifetime.as_deref().unwrap_or(""),
                if mutable { "mut " } else { "" }
            );
        }
        if let Some(element) = inner
            .strip_prefix("List<")
            .and_then(|value| value.strip_suffix('>'))
        {
            return format!(
                "&{}{}[{}]",
                lifetime.as_deref().unwrap_or(""),
                if mutable { "mut " } else { "" },
                rust_type_for(element, interfaces, with_spawn)
            );
        }
        if interfaces.contains(inner) {
            return format!(
                "&{}{}dyn {}",
                lifetime.as_deref().unwrap_or(""),
                if mutable { "mut " } else { "" },
                inner
            );
        }
        return format!(
            "&{}{}{}",
            lifetime.as_deref().unwrap_or(""),
            if mutable { "mut " } else { "" },
            rust_type_for(inner, interfaces, with_spawn)
        );
    }
    if let Some(inner) = t.strip_prefix("Future<").and_then(|t| t.strip_suffix('>')) {
        return rust_type_for(inner, interfaces, with_spawn);
    }
    if let Some(inner) = t.strip_prefix("List<").and_then(|t| t.strip_suffix('>')) {
        return format!(
            "Vec<{}>",
            if interfaces.contains(inner) {
                interface_object_type(inner, with_spawn)
            } else {
                rust_type_for(inner, interfaces, with_spawn)
            }
        );
    }
    if let Some(inner) = t
        .strip_prefix("Iterator<")
        .and_then(|value| value.strip_suffix('>'))
    {
        return format!(
            "Box<dyn Iterator<Item = {}>>",
            rust_type_for(inner, interfaces, with_spawn)
        );
    }
    if let Some(inner) = t.strip_suffix('?') {
        return format!("Option<{}>", rust_type_for(inner, interfaces, with_spawn));
    }
    if let Some(open) = t.find('<') {
        if t.ends_with('>') {
            let base = &t[..open];
            let args = &t[open + 1..t.len() - 1];
            let rendered = args
                .split(", ")
                .map(|arg| rust_type_for(arg, interfaces, with_spawn))
                .collect::<Vec<_>>()
                .join(", ");
            let rust_base = match base {
                "Stream" => "orust_runtime::Stream",
                "Channel" => "orust_runtime::Channel",
                "Map" => "std::collections::HashMap",
                "Set" => "std::collections::HashSet",
                "Fn" => "Fn",
                "FnMut" => "FnMut",
                "FnOnce" => "FnOnce",
                other => other,
            };
            if matches!(base, "Fn" | "FnMut" | "FnOnce") {
                let mut parts = rendered.splitn(2, ',');
                let argument = parts.next().unwrap_or("()").trim();
                let result = parts.next().unwrap_or("()").trim();
                return format!("Box<dyn {}({}) -> {}>", rust_base, argument, result);
            }
            return format!("{rust_base}<{rendered}>");
        }
    }
    if interfaces.contains(t) {
        return interface_object_type(t, with_spawn);
    }
    rust_type(t).to_string()
}

fn interface_object_type(name: &str, with_spawn: bool) -> String {
    if with_spawn {
        format!("Box<dyn {name} + Send + Sync>")
    } else {
        format!("Box<dyn {name}>")
    }
}

fn emit_constructor(
    out: &mut String,
    class: &orust_syntax::Class,
    constructor: &orust_syntax::Constructor,
    interfaces: &HashSet<String>,
    with_spawn: bool,
    call_signatures: &HashMap<String, Vec<Param>>,
) {
    let visibility = if class.exported && constructor.exported {
        "pub "
    } else {
        ""
    };
    let has_defaults = constructor
        .params
        .iter()
        .any(|parameter| parameter.default.is_some());
    let args_name = constructor_args_name(&class.name, &constructor.name);
    if has_defaults {
        out.push_str(&format!(
            "    {visibility}fn {}(args: {args_name})",
            constructor.name
        ));
    } else {
        out.push_str(&format!("    {visibility}fn {}(", constructor.name));
        for (index, parameter) in constructor.params.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!(
                "{}: {}",
                parameter.name,
                constructor_param_type(&parameter.ty, interfaces, with_spawn)
            ));
        }
        out.push(')');
    }
    out.push_str(" -> Self {\n");
    if has_defaults {
        for parameter in &constructor.params {
            let value = parameter
                .default
                .as_ref()
                .map(expr)
                .map(|default| format!("unwrap_or({default})"))
                .unwrap_or_else(|| {
                    format!("expect(\"missing required parameter {}\")", parameter.name)
                });
            out.push_str(&format!(
                "        let {} = args.{}.{};\n",
                parameter.name, parameter.name, value
            ));
        }
    }
    let has_body = !constructor.body.is_empty();
    if has_body {
        out.push_str("        let mut value = Self {\n");
    } else {
        out.push_str("        Self {\n");
    }
    for field in &class.fields {
        let initializes = constructor.initializing_fields.contains(&field.name);
        let value = if initializes {
            field.name.clone()
        } else {
            field
                .initializer
                .as_ref()
                .map(expr)
                .unwrap_or_else(|| "Default::default()".into())
        };
        let value = if initializes && field.ty.starts_with("Box<") && field.ty.ends_with('?') {
            format!("{value}.map(Box::new)")
        } else {
            box_field_value(&field.ty, value)
        };
        out.push_str(&format!("            {}: {},\n", field.name, value));
    }
    if has_body {
        out.push_str("        };\n");
    } else {
        out.push_str("        }\n");
    }
    if has_body {
        for statement in &constructor.body {
            match &statement.node {
                Stmt::Var {
                    name, initializer, ..
                } => out.push_str(&format!(
                    "        let mut {} = {};\n",
                    name,
                    render_function_expr(initializer, None, false, call_signatures, interfaces)
                        .replace("self.", "value.")
                )),
                Stmt::Print(expression) => out.push_str(&format!(
                    "        println!(\"{{}}\", {});\n",
                    render_function_expr(expression, None, false, call_signatures, interfaces)
                        .replace("self.", "value.")
                )),
                Stmt::Expr(expression) => out.push_str(&format!(
                    "        {};\n",
                    render_function_expr(expression, None, false, call_signatures, interfaces)
                        .replace("self.", "value.")
                )),
                Stmt::Return(Some(expression)) => out.push_str(&format!(
                    "        return {};\n",
                    render_function_expr(expression, None, false, call_signatures, interfaces)
                        .replace("self.", "value.")
                )),
                Stmt::Return(None) => out.push_str("        return value;\n"),
                Stmt::Rust(raw) => out.push_str(&format!("        {raw}\n")),
                other => emit_simple(out, other, 2),
            }
        }
        out.push_str("        value\n");
    }
    out.push_str("    }\n");
}

fn emit_constructor_args(
    out: &mut String,
    class: &orust_syntax::Class,
    constructor: &orust_syntax::Constructor,
    interfaces: &HashSet<String>,
    with_spawn: bool,
) {
    let name = constructor_args_name(&class.name, &constructor.name);
    out.push_str(&format!("#[derive(Default)]\nstruct {name} {{\n"));
    for parameter in &constructor.params {
        out.push_str(&format!(
            "    {}: Option<{}>,\n",
            parameter.name,
            constructor_param_type(&parameter.ty, interfaces, with_spawn)
        ));
    }
    out.push_str("}\n");
}

#[allow(clippy::too_many_arguments)]
fn emit_function(
    out: &mut String,
    f: &Function,
    method: bool,
    fields: Option<&[String]>,
    method_mutability: &HashMap<String, HashMap<String, bool>>,
    interfaces: &HashSet<String>,
    with_spawn: bool,
    call_signatures: &HashMap<String, Vec<Param>>,
    force_mut_receiver: bool,
) {
    let inferred_lifetime = !method
        && f.return_type.starts_with('&')
        && !f.return_type.contains('\'')
        && f.params
            .iter()
            .any(|parameter| parameter.ty.starts_with('&'))
        && !f.params.iter().any(|parameter| parameter.ty.contains('\''));
    let function_generics = if inferred_lifetime {
        let declared = generic_decl(&f.generics, &inferred_generic_bounds(f));
        if declared.is_empty() {
            "<'a>".into()
        } else {
            format!("{}, 'a>", &declared[..declared.len() - 1])
        }
    } else {
        generic_decl(&f.generics, &inferred_generic_bounds(f))
    };
    out.push_str(&format!(
        "{}{}fn {}{}{}(",
        visibility_prefix(f.visibility, f.exported),
        if f.is_async { "async " } else { "" },
        declared_name(&f.name, &f.rust_name),
        function_generics,
        ""
    ));
    let uses_parameter_struct =
        !method && f.params.iter().any(|parameter| parameter.default.is_some());
    if uses_parameter_struct {
        out.push_str(&format!(
            "args: {}Args",
            declared_name(&f.name, &f.rust_name)
        ));
    } else if method {
        out.push_str(if force_mut_receiver || function_needs_mut(f) {
            "&mut self"
        } else {
            "&self"
        });
        if !f.params.is_empty() {
            out.push_str(", ");
        }
    }
    for (i, p) in f.params.iter().enumerate() {
        if uses_parameter_struct {
            break;
        }
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{}: {}",
            p.name,
            function_parameter_type(p, &f.body, inferred_lifetime, interfaces, with_spawn,)
        ));
    }
    let return_type =
        function_return_type(&f.return_type, inferred_lifetime, interfaces, with_spawn);
    let return_type = if f.throws {
        let error_type = f.throws_type.as_deref().unwrap_or("orust_runtime::Error");
        format!("Result<{}, {}>", return_type, error_type)
    } else {
        return_type
    };
    out.push_str(&format!(") -> {} {{\n", return_type));
    if let Some(path) = &f.rust_import {
        let arguments = f
            .params
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let call = if f.is_async {
            format!("{path}({arguments}).await")
        } else {
            format!("{path}({arguments})")
        };
        if f.throws {
            out.push_str(&format!("    return {call};\n"));
        } else if return_type == "()" {
            out.push_str(&format!("    {call};\n"));
        } else {
            out.push_str(&format!("    return {call};\n"));
        }
        out.push_str("}\n");
        return;
    }
    if !method && f.name == "main" {
        out.push_str("    orust_runtime::install_panic_hook();\n");
    }
    if uses_parameter_struct {
        for parameter in &f.params {
            let value = parameter
                .default
                .as_ref()
                .map(expr)
                .map(|default| format!("unwrap_or({default})"))
                .unwrap_or_else(|| {
                    format!("expect(\"missing required parameter {}\")", parameter.name)
                });
            out.push_str(&format!(
                "    let {} = args.{}.{};\n",
                parameter.name, parameter.name, value
            ));
        }
    }
    let nullable_locals: HashSet<String> = f
        .body
        .iter()
        .filter_map(|statement| match &statement.node {
            Stmt::Var {
                name,
                declared_type: Some(ty),
                ..
            } if ty.ends_with('?') => Some(name.clone()),
            _ => None,
        })
        .collect();
    for s in &f.body {
        match &s.node {
            Stmt::Var {
                name,
                initializer,
                declared_type,
            } => out.push_str(&format!(
                "    let{} {}{} = {};\n",
                if local_needs_mut(name, initializer, &f.body, method_mutability) {
                    " mut"
                } else {
                    ""
                },
                binding_source(name),
                declared_type
                    .as_ref()
                    .map(|ty| format!(": {}", rust_type_for(ty, interfaces, with_spawn)))
                    .unwrap_or_default(),
                widen_integer_literal(
                    initializer,
                    declared_type.as_deref(),
                    render_function_expr(
                        initializer,
                        fields,
                        f.throws,
                        call_signatures,
                        interfaces
                    ),
                )
            )),
            Stmt::PatternVar {
                pattern,
                initializer,
                else_body,
            } => {
                out.push_str(&format!(
                    "    let {} = {} else {{\n",
                    pattern,
                    render_function_expr(
                        initializer,
                        fields,
                        f.throws,
                        call_signatures,
                        interfaces
                    )
                ));
                emit_block(out, else_body, 2);
                out.push_str("    };\n");
            }
            Stmt::Print(e) => {
                let rendered =
                    render_function_expr(e, fields, f.throws, call_signatures, interfaces);
                let rendered = if let Expr::Name(name) = e {
                    let source_name = name.strip_prefix("__orust_nullable__").unwrap_or(name);
                    if nullable_locals.contains(source_name) {
                        format!("orust_runtime::display_option(&{rendered})")
                    } else {
                        rendered
                    }
                } else {
                    rendered
                };
                out.push_str(&format!("    println!(\"{{}}\", {rendered});\n"));
            }
            Stmt::Return(Some(e)) => {
                let rendered =
                    render_function_expr(e, fields, f.throws, call_signatures, interfaces);
                let rendered = if f.return_type.ends_with('?')
                    && !matches!(e, Expr::Null | Expr::Coalesce { .. })
                    && !rendered.starts_with("Some(")
                    && !rendered.starts_with("None")
                {
                    format!("Some({rendered})")
                } else {
                    rendered
                };
                if f.throws {
                    out.push_str(&format!("    return Ok({rendered});\n"));
                } else {
                    out.push_str(&format!("    return {rendered};\n"));
                }
            }
            Stmt::Return(None) => out.push_str("    return;\n"),
            Stmt::Throw(e) => out.push_str(&format!(
                "    return Err({}.into());\n",
                render_function_expr(e, fields, false, call_signatures, interfaces)
            )),
            Stmt::Rust(raw) => out.push_str(&format!("    {}\n", raw)),
            Stmt::TryCatch {
                body,
                error,
                catch_body,
            } => {
                emit_try_catch(out, body, error, catch_body, 1);
            }
            Stmt::Expr(e) => out.push_str(&format!(
                "    {};\n",
                render_function_expr(e, fields, f.throws, call_signatures, interfaces)
            )),
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                out.push_str(&format!(
                    "    if {} {{\n",
                    render_function_expr(condition, fields, f.throws, call_signatures, interfaces)
                ));
                emit_block(out, then_body, 2);
                out.push_str("    }");
                if !else_body.is_empty() {
                    out.push_str(" else {\n");
                    emit_block(out, else_body, 2);
                    out.push_str("    }");
                }
                out.push('\n');
            }
            Stmt::While { condition, body } => {
                out.push_str(&format!("    while {} {{\n", expr(condition)));
                emit_block(out, body, 2);
                out.push_str("    }\n");
            }
            Stmt::WhileCase {
                value,
                pattern,
                body,
            } => {
                out.push_str(&format!(
                    "    while let {} = {} {{\n",
                    pattern,
                    render_function_expr(value, fields, f.throws, call_signatures, interfaces)
                ));
                emit_block(out, body, 2);
                out.push_str("    }\n");
            }
            Stmt::For {
                initializer,
                condition,
                step,
                body,
            } => {
                if let Some(init) = initializer.as_ref() {
                    emit_simple(out, init, 1);
                }
                out.push_str(&format!(
                    "    while {} {{\n",
                    condition
                        .as_ref()
                        .map(expr)
                        .unwrap_or_else(|| "true".into())
                ));
                emit_block(out, body, 2);
                if let Some(step) = step {
                    out.push_str(&format!("        {};\n", expr(step)));
                }
                out.push_str("    }\n");
            }
            Stmt::ForIn {
                name,
                iterable,
                body,
            } => {
                out.push_str(&format!(
                    "    for {name} in {} {{\n",
                    for_in_source(iterable)
                ));
                emit_block(out, body, 2);
                out.push_str("    }\n");
            }
            Stmt::Switch { .. } | Stmt::IfCase { .. } => emit_simple(out, &s.node, 1),
            Stmt::AwaitFor { name, stream, body } => {
                out.push_str(&format!(
                    "    while let Some({name}) = {}.next().await {{\n",
                    expr(stream)
                ));
                emit_block(out, body, 2);
                out.push_str("    }\n");
            }
        }
    }
    out.push_str("}\n");
}

fn inferred_generic_bounds(function: &Function) -> Vec<String> {
    let mut bounds = function.bounds.clone();
    for generic in &function.generics {
        let generic_parameters: HashSet<&str> = function
            .params
            .iter()
            .filter(|parameter| parameter.ty == *generic)
            .map(|parameter| parameter.name.as_str())
            .collect();
        if !bounds
            .iter()
            .any(|bound| bound.starts_with(&format!("{generic}:")))
            && statements_copy_generic(&function.body, generic, &generic_parameters)
        {
            bounds.push(format!("{generic}: Clone"));
        }
    }
    bounds
}

fn statements_copy_generic(
    statements: &[orust_syntax::Spanned<Stmt>],
    generic: &str,
    parameters: &HashSet<&str>,
) -> bool {
    statements.iter().any(|statement| match &statement.node {
        Stmt::PatternVar {
            initializer,
            else_body,
            ..
        } => {
            expr_copies_generic(initializer, generic, parameters)
                || statements_copy_generic(else_body, generic, parameters)
        }
        Stmt::Var { initializer, .. }
        | Stmt::Print(initializer)
        | Stmt::Expr(initializer)
        | Stmt::Return(Some(initializer)) => expr_copies_generic(initializer, generic, parameters),
        Stmt::If {
            then_body,
            else_body,
            ..
        } => {
            statements_copy_generic(then_body, generic, parameters)
                || statements_copy_generic(else_body, generic, parameters)
        }
        Stmt::While { body, .. } | Stmt::WhileCase { body, .. } | Stmt::For { body, .. } => {
            statements_copy_generic(body, generic, parameters)
        }
        Stmt::ForIn { body, .. } => statements_copy_generic(body, generic, parameters),
        Stmt::TryCatch {
            body, catch_body, ..
        } => {
            statements_copy_generic(body, generic, parameters)
                || statements_copy_generic(catch_body, generic, parameters)
        }
        Stmt::Switch { cases, .. } => cases
            .iter()
            .any(|case| statements_copy_generic(&case.body, generic, parameters)),
        Stmt::IfCase {
            then_body,
            else_body,
            ..
        } => {
            statements_copy_generic(then_body, generic, parameters)
                || statements_copy_generic(else_body, generic, parameters)
        }
        Stmt::AwaitFor { body, .. } => statements_copy_generic(body, generic, parameters),
        Stmt::Return(None) | Stmt::Rust(_) => false,
        Stmt::Throw(value) => expr_uses_spawn(value),
    })
}

fn expr_copies_generic(expression: &Expr, generic: &str, parameters: &HashSet<&str>) -> bool {
    match expression {
        Expr::Copy(value) => {
            matches!(value.as_ref(), Expr::Name(name) if name == generic || parameters.contains(name.as_str()))
        }
        Expr::Call { callee, args } => {
            expr_copies_generic(callee, generic, parameters)
                || args
                    .iter()
                    .any(|argument| expr_copies_generic(argument, generic, parameters))
        }
        Expr::Member { object, .. }
        | Expr::Borrow { value: object, .. }
        | Expr::Await(object)
        | Expr::Spawn(object)
        | Expr::OptionalMember { object, .. } => expr_copies_generic(object, generic, parameters),
        Expr::Binary { left, right, .. } | Expr::Coalesce { left, right } => {
            expr_copies_generic(left, generic, parameters)
                || expr_copies_generic(right, generic, parameters)
        }
        Expr::List(values) => values
            .iter()
            .any(|value| expr_copies_generic(value, generic, parameters)),
        _ => false,
    }
}

fn function_type(
    ty: &str,
    inferred_lifetime: bool,
    interfaces: &HashSet<String>,
    with_spawn: bool,
) -> String {
    if inferred_lifetime && ty.starts_with("&mut ") {
        format!(
            "&'a mut {}",
            rust_type_for(&ty[5..], interfaces, with_spawn)
        )
    } else if inferred_lifetime && ty.starts_with('&') {
        format!("&'a {}", rust_type_for(&ty[1..], interfaces, with_spawn))
    } else {
        rust_type_for(ty, interfaces, with_spawn)
    }
}

fn function_return_type(
    ty: &str,
    inferred_lifetime: bool,
    interfaces: &HashSet<String>,
    with_spawn: bool,
) -> String {
    if let Some(inner) = ty
        .strip_prefix("Iterator<")
        .and_then(|value| value.strip_suffix('>'))
    {
        return format!(
            "impl Iterator<Item = {}>",
            rust_type_for(inner, interfaces, with_spawn)
        );
    }
    function_type(ty, inferred_lifetime, interfaces, with_spawn)
}

fn function_parameter_type(
    parameter: &Param,
    body: &[orust_syntax::Spanned<Stmt>],
    inferred_lifetime: bool,
    interfaces: &HashSet<String>,
    with_spawn: bool,
) -> String {
    if let Some(element) = parameter
        .ty
        .strip_prefix("&mut List<")
        .and_then(|value| value.strip_suffix('>'))
    {
        if statements_grow_collection(body, &parameter.name) {
            return format!(
                "{}mut Vec<{}>",
                if inferred_lifetime { "&'a " } else { "&" },
                rust_type_for(element, interfaces, with_spawn)
            );
        }
    }
    function_type(&parameter.ty, inferred_lifetime, interfaces, with_spawn)
}

fn statements_grow_collection(statements: &[orust_syntax::Spanned<Stmt>], name: &str) -> bool {
    fn expr_grows_collection(expression: &Expr, name: &str) -> bool {
        match expression {
            Expr::Call { callee, args } => {
                let direct = matches!(callee.as_ref(), Expr::Member { object, name: method }
                    if matches!(object.as_ref(), Expr::Name(value) if value == name)
                        && matches!(method.as_str(), "add" | "remove"));
                direct
                    || expr_grows_collection(callee, name)
                    || args.iter().any(|arg| expr_grows_collection(arg, name))
            }
            Expr::Member { object, .. }
            | Expr::OptionalMember { object, .. }
            | Expr::Index { object, .. }
            | Expr::Borrow { value: object, .. }
            | Expr::Copy(object)
            | Expr::Await(object)
            | Expr::Spawn(object) => expr_grows_collection(object, name),
            Expr::Slice {
                object, start, end, ..
            } => {
                expr_grows_collection(object, name)
                    || start
                        .as_deref()
                        .is_some_and(|value| expr_grows_collection(value, name))
                    || end
                        .as_deref()
                        .is_some_and(|value| expr_grows_collection(value, name))
            }
            Expr::Binary { left, right, .. } | Expr::Coalesce { left, right } => {
                expr_grows_collection(left, name) || expr_grows_collection(right, name)
            }
            Expr::List(values) => values
                .iter()
                .any(|value| expr_grows_collection(value, name)),
            Expr::NewArgs { args, .. } => {
                args.iter().any(|value| expr_grows_collection(value, name))
            }
            Expr::NamedArg { value, .. } => expr_grows_collection(value, name),
            Expr::Closure { body, .. } => match body {
                orust_syntax::ClosureBody::Expr(value) => expr_grows_collection(value, name),
                orust_syntax::ClosureBody::Block(body) => statements_grow_collection(body, name),
            },
            _ => false,
        }
    }
    statements.iter().any(|statement| match &statement.node {
        Stmt::Var { initializer, .. }
        | Stmt::PatternVar { initializer, .. }
        | Stmt::Print(initializer)
        | Stmt::Expr(initializer)
        | Stmt::Return(Some(initializer))
        | Stmt::Throw(initializer) => expr_grows_collection(initializer, name),
        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_grows_collection(condition, name)
                || statements_grow_collection(then_body, name)
                || statements_grow_collection(else_body, name)
        }
        Stmt::While { condition, body }
        | Stmt::WhileCase {
            value: condition,
            body,
            ..
        } => expr_grows_collection(condition, name) || statements_grow_collection(body, name),
        Stmt::For { body, .. } | Stmt::ForIn { body, .. } | Stmt::AwaitFor { body, .. } => {
            statements_grow_collection(body, name)
        }
        Stmt::TryCatch {
            body, catch_body, ..
        } => statements_grow_collection(body, name) || statements_grow_collection(catch_body, name),
        Stmt::Switch { value, cases } => {
            expr_grows_collection(value, name)
                || cases
                    .iter()
                    .any(|case| statements_grow_collection(&case.body, name))
        }
        Stmt::IfCase {
            value,
            then_body,
            else_body,
            ..
        } => {
            expr_grows_collection(value, name)
                || statements_grow_collection(then_body, name)
                || statements_grow_collection(else_body, name)
        }
        Stmt::Return(None) | Stmt::Rust(_) => false,
    })
}

fn local_needs_mut(
    name: &str,
    initializer: &Expr,
    body: &[orust_syntax::Spanned<Stmt>],
    methods: &HashMap<String, HashMap<String, bool>>,
) -> bool {
    let class_name = match initializer {
        Expr::New(class_name) => class_name,
        Expr::NewArgs {
            name: class_name, ..
        } => class_name,
        _ => "",
    };
    fn used_mutably(
        name: &str,
        statements: &[orust_syntax::Spanned<Stmt>],
        class_name: &str,
        methods: &HashMap<String, HashMap<String, bool>>,
    ) -> bool {
        for statement in statements {
            let expressions = match &statement.node {
                Stmt::Var { initializer, .. } => vec![initializer],
                Stmt::Expr(e) | Stmt::Print(e) | Stmt::Return(Some(e)) => vec![e],
                _ => Vec::new(),
            };
            for expression in expressions {
                if let Expr::Binary { left, op, .. } = expression {
                    if op == "="
                        && matches!(left.as_ref(), Expr::Name(value) if value.strip_prefix("__orust_nullable__").unwrap_or(value) == name)
                    {
                        return true;
                    }
                }
                if let Expr::Call { callee, .. } = expression {
                    if let Expr::Member {
                        object,
                        name: method,
                    } = callee.as_ref()
                    {
                        if matches!(object.as_ref(), Expr::Name(object_name) if object_name == name)
                            && methods
                                .get(class_name)
                                .and_then(|class| class.get(method))
                                .copied()
                                .unwrap_or(false)
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    used_mutably(name, body, class_name, methods)
}

fn function_needs_mut(function: &Function) -> bool {
    fn expr_needs_mut(expression: &Expr) -> bool {
        match expression {
            Expr::Binary { left, op, right } => {
                op == "=" || expr_needs_mut(left) || expr_needs_mut(right)
            }
            Expr::Call { callee, args } => {
                expr_needs_mut(callee) || args.iter().any(expr_needs_mut)
            }
            Expr::Member { object, .. }
            | Expr::Borrow { value: object, .. }
            | Expr::Copy(object) => expr_needs_mut(object),
            _ => false,
        }
    }
    fn statement_needs_mut(statement: &Stmt) -> bool {
        match statement {
            Stmt::Expr(expression) => expr_needs_mut(expression),
            Stmt::If {
                then_body,
                else_body,
                ..
            } => {
                then_body
                    .iter()
                    .any(|statement| statement_needs_mut(&statement.node))
                    || else_body
                        .iter()
                        .any(|statement| statement_needs_mut(&statement.node))
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } => body
                .iter()
                .any(|statement| statement_needs_mut(&statement.node)),
            Stmt::IfCase {
                then_body,
                else_body,
                ..
            } => then_body
                .iter()
                .chain(else_body.iter())
                .any(|statement| statement_needs_mut(&statement.node)),
            Stmt::Switch { cases, .. } => cases.iter().any(|case| {
                case.body
                    .iter()
                    .any(|statement| statement_needs_mut(&statement.node))
            }),
            Stmt::AwaitFor { body, .. } => body
                .iter()
                .any(|statement| statement_needs_mut(&statement.node)),
            _ => false,
        }
    }
    function
        .body
        .iter()
        .any(|statement| statement_needs_mut(&statement.node))
}
fn for_in_source(iterable: &Expr) -> String {
    if let Expr::Call { callee, args } = iterable {
        if args.is_empty() {
            if let Expr::Member { object, name } = callee.as_ref() {
                if name == "consume" {
                    return format!("{}.into_iter()", expr(object));
                }
            }
        }
    }
    format!("&{}", expr(iterable))
}

fn binding_source(name: &str) -> String {
    name.strip_prefix("__orust_tuple_pattern__")
        .map(|bindings| format!("({bindings})"))
        .unwrap_or_else(|| name.to_string())
}

fn emit_block(out: &mut String, body: &[orust_syntax::Spanned<Stmt>], indent: usize) {
    for statement in body {
        emit_simple(out, &statement.node, indent);
    }
}

fn emit_boxed_pattern_bindings(out: &mut String, bindings: &[String], indent: usize) {
    let pad = "    ".repeat(indent);
    for binding in bindings {
        if let Some(name) = binding.strip_prefix("__orust_box_binding__") {
            out.push_str(&format!("{pad}let {name} = *{name};\n"));
        }
    }
}

fn emit_simple(out: &mut String, statement: &Stmt, indent: usize) {
    let pad = "    ".repeat(indent);
    match statement {
        Stmt::Var {
            name, initializer, ..
        } => out.push_str(&format!(
            "{}let mut {} = {};\n",
            pad,
            binding_source(name),
            expr(initializer)
        )),
        Stmt::PatternVar {
            pattern,
            initializer,
            else_body,
        } => {
            out.push_str(&format!(
                "{}let {} = {} else {{\n",
                pad,
                pattern,
                expr(initializer)
            ));
            emit_block(out, else_body, indent + 1);
            out.push_str(&format!("{}}};\n", pad));
        }
        Stmt::Print(e) => out.push_str(&format!("{}println!(\"{{}}\", {});\n", pad, expr(e))),
        Stmt::Return(Some(e)) => out.push_str(&format!("{}return {};\n", pad, expr(e))),
        Stmt::Return(None) => out.push_str(&format!("{}return;\n", pad)),
        Stmt::Throw(e) => out.push_str(&format!("{}return Err({}.into());\n", pad, expr(e))),
        Stmt::Rust(raw) => out.push_str(&format!("{}{}\n", pad, raw)),
        Stmt::Expr(e) => out.push_str(&format!("{}{};\n", pad, expr(e))),
        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            out.push_str(&format!("{}if {} {{\n", pad, expr(condition)));
            emit_block(out, then_body, indent + 1);
            out.push_str(&format!("{}}}", pad));
            if !else_body.is_empty() {
                out.push_str(" else {\n");
                emit_block(out, else_body, indent + 1);
                out.push_str(&format!("{}}}", pad));
            }
            out.push('\n');
        }
        Stmt::While { condition, body } => {
            out.push_str(&format!("{}while {} {{\n", pad, expr(condition)));
            emit_block(out, body, indent + 1);
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::WhileCase {
            value,
            pattern,
            body,
        } => {
            out.push_str(&format!(
                "{}while let {} = {} {{\n",
                pad,
                pattern,
                expr(value)
            ));
            emit_block(out, body, indent + 1);
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::For { .. } => out.push_str(&format!("{}/* nested for */\n", pad)),
        Stmt::ForIn {
            name,
            iterable,
            body,
        } => {
            out.push_str(&format!(
                "{}for {name} in {} {{\n",
                pad,
                for_in_source(iterable)
            ));
            emit_block(out, body, indent + 1);
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::TryCatch {
            body,
            error,
            catch_body,
        } => emit_try_catch(out, body, error, catch_body, indent),
        Stmt::IfCase {
            value,
            pattern,
            bindings,
            then_body,
            else_body,
            ..
        } => {
            out.push_str(&format!("{}if let {} = {} {{\n", pad, pattern, expr(value)));
            emit_boxed_pattern_bindings(out, bindings, indent + 1);
            emit_block(out, then_body, indent + 1);
            out.push_str(&format!("{}}}", pad));
            if !else_body.is_empty() {
                out.push_str(" else {\n");
                emit_block(out, else_body, indent + 1);
                out.push_str(&format!("{}}}", pad));
            }
            out.push('\n');
        }
        Stmt::Switch { value, cases } => {
            out.push_str(&format!("{}match {} {{\n", pad, expr(value)));
            for case in cases {
                let pattern = case
                    .guard
                    .as_ref()
                    .map(|guard| format!("{} if {}", case.pattern, guard))
                    .unwrap_or_else(|| case.pattern.clone());
                out.push_str(&format!("{}    {} => {{\n", pad, pattern));
                emit_boxed_pattern_bindings(out, &case.bindings, indent + 2);
                emit_block(out, &case.body, indent + 2);
                out.push_str(&format!("{}    }},\n", pad));
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::AwaitFor { name, stream, body } => {
            out.push_str(&format!(
                "{}while let Some({name}) = {}.next().await {{\n",
                pad,
                expr(stream)
            ));
            emit_block(out, body, indent + 1);
            out.push_str(&format!("{}}}\n", pad));
        }
    }
}

fn widen_integer_literal(
    initializer: &Expr,
    declared_type: Option<&str>,
    rendered: String,
) -> String {
    if declared_type == Some("double") && matches!(initializer, Expr::Int(_)) {
        format!("({rendered} as f64)")
    } else {
        rendered
    }
}

fn expr(e: &Expr) -> String {
    match e {
        Expr::Int(n) => n.to_string(),
        Expr::Float(n) if n.fract() == 0.0 => format!("{n:.1}"),
        Expr::Float(n) => n.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::String(s) => format!("{}.to_string()", rust_string_literal(s)),
        Expr::Name(n) => n
            .strip_prefix("__orust_rust_expr__")
            .or_else(|| n.strip_prefix("__orust_nullable__"))
            .unwrap_or(n)
            .to_string(),
        Expr::New(n) if n.starts_with("__orust_newtype__") => {
            format!("{}()", n.trim_start_matches("__orust_newtype__"))
        }
        Expr::New(n) => format!("{}::new()", n),
        Expr::NewArgs { name, args } => {
            if name == "__orust_tuple__" {
                return format!("({})", args.iter().map(expr).collect::<Vec<_>>().join(", "));
            }
            if let Some(type_name) = name.strip_prefix("__orust_newtype__") {
                return format!(
                    "{}({})",
                    type_name,
                    args.iter().map(expr).collect::<Vec<_>>().join(", ")
                );
            }
            if let Some(encoded) = name.strip_prefix("__orust_enum__") {
                let mut parts = encoded.splitn(3, "__");
                let enum_name = parts.next().unwrap_or_default();
                let variant_name = parts.next().unwrap_or_default();
                let field_types = parts
                    .next()
                    .unwrap_or_default()
                    .split(',')
                    .collect::<Vec<_>>();
                let values = args
                    .iter()
                    .enumerate()
                    .map(|(index, argument)| {
                        let rendered = expr(argument);
                        if field_types
                            .get(index)
                            .is_some_and(|field| field.starts_with("Box<"))
                        {
                            format!("Box::new({rendered})")
                        } else {
                            rendered
                        }
                    })
                    .collect::<Vec<_>>();
                return if values.is_empty() {
                    format!("{enum_name}::{variant_name}")
                } else {
                    format!("{enum_name}::{variant_name}({})", values.join(", "))
                };
            }
            if name == "__orust_record__" {
                let fields = args
                    .iter()
                    .filter_map(|arg| match arg {
                        Expr::NamedArg { name, value } => {
                            Some((name.clone(), inferred_expr_type(value)))
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let values = args
                    .iter()
                    .filter_map(|arg| match arg {
                        Expr::NamedArg { name, value } => Some(format!("{name}: {}", expr(value))),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                return format!(
                    "crate::orust_records::{} {{ {} }}",
                    record_name(&fields),
                    values.join(", ")
                );
            }
            if let Some(encoded) = name.strip_prefix("__orust_error__") {
                let mut parts = encoded.splitn(3, "__");
                let error_name = parts.next().unwrap_or_default();
                let variant = parts.next().unwrap_or("new");
                let field_names = parts
                    .next()
                    .unwrap_or_default()
                    .split(',')
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>();
                let values = field_names
                    .iter()
                    .enumerate()
                    .filter_map(|(index, field)| {
                        let value = args.iter().find_map(|argument| match argument {
                            Expr::NamedArg { name, value } if name == field => Some(value.as_ref()),
                            _ if !args
                                .iter()
                                .any(|argument| matches!(argument, Expr::NamedArg { .. }))
                                && index < args.len() =>
                            {
                                Some(&args[index])
                            }
                            _ => None,
                        })?;
                        Some(format!("{field}: {}", expr(value)))
                    })
                    .collect::<Vec<_>>();
                if variant == "new" {
                    return format!("{error_name} {{ {} }}", values.join(", "));
                }
                if values.is_empty() {
                    return format!("{error_name}::{variant}");
                }
                return format!("{error_name}::{variant} {{ {} }}", values.join(", "));
            }
            if let Some(encoded) = name.strip_prefix("__orust_ctor__") {
                let mut parts = encoded.splitn(5, "__");
                let class_name = parts.next().unwrap_or_default();
                let constructor_name = parts.next().unwrap_or("new");
                let parameter_names = parts
                    .next()
                    .unwrap_or_default()
                    .split(',')
                    .filter(|value| !value.is_empty())
                    .collect::<Vec<_>>();
                let has_defaults = parts.next() == Some("d");
                let parameter_types = parts
                    .next()
                    .unwrap_or_default()
                    .split(',')
                    .collect::<Vec<_>>();
                let mut fields = Vec::new();
                for (index, parameter) in parameter_names.iter().enumerate() {
                    let value = args.iter().find_map(|argument| match argument {
                        Expr::NamedArg { name, value } if name == parameter => Some(value.as_ref()),
                        _ if !args
                            .iter()
                            .any(|argument| matches!(argument, Expr::NamedArg { .. }))
                            && index < args.len() =>
                        {
                            Some(&args[index])
                        }
                        _ => None,
                    });
                    if let Some(value) = value {
                        let rendered = expr(value);
                        let rendered = if parameter_types
                            .get(index)
                            .is_some_and(|ty| ty.ends_with('?'))
                            && !matches!(value, Expr::Null)
                            && !matches!(value, Expr::Name(name) if name.starts_with("__orust_nullable__"))
                        {
                            format!("Some({rendered})")
                        } else {
                            rendered
                        };
                        fields.push(format!("{parameter}: {rendered}"));
                    }
                }
                let args_name = constructor_args_name(class_name, constructor_name);
                if has_defaults {
                    let fields = fields
                        .iter()
                        .map(|field| {
                            let (name, value) = field.split_once(": ").unwrap_or((field, field));
                            format!("{name}: Some({value})")
                        })
                        .collect::<Vec<_>>();
                    let fields = if fields.is_empty() {
                        "..Default::default()".into()
                    } else {
                        format!("{}, ..Default::default()", fields.join(", "))
                    };
                    return format!(
                        "{}::{}({args_name} {{ {fields} }})",
                        class_name, constructor_name,
                    );
                }
                return format!(
                    "{}::{}({})",
                    class_name,
                    constructor_name,
                    fields
                        .iter()
                        .map(|field| field
                            .split_once(": ")
                            .map(|(_, value)| value)
                            .unwrap_or(field))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            format!(
                "{}::new({})",
                name,
                args.iter().map(expr).collect::<Vec<_>>().join(", ")
            )
        }
        Expr::Index { object, index } => format!(
            "{}[orust_runtime::checked_index({}, {}.len())]",
            expr(object),
            expr(index),
            expr(object)
        ),
        Expr::Slice {
            object,
            start,
            end,
            inclusive,
        } => {
            let object_code = expr(object);
            let bound = |value: &Expr| {
                format!(
                    "orust_runtime::checked_index({}, {}.len())",
                    expr(value),
                    object_code
                )
            };
            let range = match (start, end, inclusive) {
                (Some(start), Some(end), true) => format!("{}..={}", bound(start), bound(end)),
                (Some(start), Some(end), false) => format!("{}..{}", bound(start), bound(end)),
                (Some(start), None, _) => format!("{}..", bound(start)),
                (None, Some(end), true) => format!("..={}", bound(end)),
                (None, Some(end), false) => format!("..{}", bound(end)),
                (None, None, _) => "..".into(),
            };
            format!("&{}[{}]", object_code, range)
        }
        Expr::This => "self".into(),
        Expr::Borrow { mutable, value } => {
            format!("&{}{}", if *mutable { "mut " } else { "" }, expr(value))
        }
        Expr::Copy(value) => format!("{}.clone()", expr(value)),
        Expr::Null => "None".into(),
        Expr::OptionalMember { object, name } => {
            format!("{}.as_ref().map(|value| value.{name}())", expr(object))
        }
        Expr::Coalesce { left, right } => format!("{}.unwrap_or({})", expr(left), expr(right)),
        Expr::Await(value) => match value.as_ref() {
            Expr::Name(_) => format!("{}.await_task().await", expr(value)),
            _ => format!("{}.await", expr(value)),
        },
        Expr::List(values) => format!(
            "vec![{}]",
            values.iter().map(expr).collect::<Vec<_>>().join(", ")
        ),
        Expr::Spawn(value) => format!("orust_runtime::spawn({})", expr_spawn(value)),
        Expr::Member { object, name } if name == "millis" || name == "seconds" => {
            format!("orust_runtime::{}({})", name, expr(object))
        }
        Expr::Member { object, name } if name == "length" => {
            format!("{}.len() as i64", expr(object))
        }
        Expr::Member { object, name } if name == "isEmpty" => {
            format!("{}.is_empty()", expr(object))
        }
        Expr::Member { object, name } if name == "charCount" => {
            format!("{}.chars().count() as i64", expr(object))
        }
        Expr::Member { object, name } if name.starts_with("__orust_box_optional__") => {
            let field = name.trim_start_matches("__orust_box_optional__");
            format!("{}.{}.map(|value| *value)", expr(object), field)
        }
        Expr::Member { object, name } if name.starts_with("__orust_box_value__") => {
            let field = name.trim_start_matches("__orust_box_value__");
            format!("*{}.{}", expr(object), field)
        }
        Expr::Member { object, name }
            if matches!(
                name.as_str(),
                "map"
                    | "expand"
                    | "zip"
                    | "take"
                    | "skip"
                    | "enumerate"
                    | "chain"
                    | "takeWhile"
                    | "skipWhile"
                    | "count"
                    | "any"
                    | "every"
                    | "sum"
                    | "reduce"
                    | "fold"
                    | "toList"
                    | "toSet"
                    | "toMap"
                    | "first"
                    | "last"
                    | "join"
                    | "forEach"
                    | "min"
                    | "max"
            ) =>
        {
            let method = match name.as_str() {
                "toList" => "collect::<Vec<_>>",
                "toSet" => "collect::<std::collections::HashSet<_>>",
                "toMap" => "collect::<std::collections::HashMap<_, _>>",
                "first" => "next",
                "join" => "map(|value| value.to_string()).collect::<Vec<_>>().join",
                "every" => "all",
                "takeWhile" => "take_while",
                "skipWhile" => "skip_while",
                "forEach" => "for_each",
                other => other,
            };
            let receiver = if matches!(object.as_ref(), Expr::Call { .. }) {
                expr(object)
            } else {
                format!("{}.iter()", expr(object))
            };
            if matches!(name.as_str(), "min" | "max") {
                return format!("{receiver}.{method}()");
            }
            format!("{receiver}.{method}")
        }
        Expr::Member { object, name } if name == "where" => {
            let receiver = if matches!(object.as_ref(), Expr::Call { .. }) {
                expr(object)
            } else {
                format!("{}.iter()", expr(object))
            };
            format!("{receiver}.filter")
        }
        Expr::Member { object, name } if matches!(object.as_ref(), Expr::Name(value) if value == "File" || value == "Path") =>
        {
            let type_name = match object.as_ref() {
                Expr::Name(value) if value == "File" => "File",
                _ => "Path",
            };
            let method = match name.as_str() {
                "readString" => "read_string",
                "writeString" => "write_string",
                "appendString" => "append_string",
                "fileName" => "file_name",
                other => other,
            };
            format!("orust_runtime::std_io::{type_name}::{method}")
        }
        Expr::Member { object, name } => {
            let object = expr(object);
            if name == "__orust_newtype_value__" {
                return format!("{object}.value()");
            }
            if let Some(index) = name
                .strip_prefix('$')
                .and_then(|value| value.parse::<usize>().ok())
            {
                return format!("{object}.{}", index.saturating_sub(1));
            }
            if let Some(namespace) = object.strip_prefix("__orust_ns__") {
                format!("{namespace}::{name}")
            } else {
                format!("{object}.{name}")
            }
        }
        Expr::Call { callee, args } => {
            if matches!(callee.as_ref(), Expr::Name(name) if name == "expect") {
                return format!(
                    "assert!({})",
                    args.first().map(expr).unwrap_or_else(|| "false".into())
                );
            }
            if let Expr::Member { object, name } = callee.as_ref() {
                let rendered_object = expr(object);
                if args.is_empty() {
                    if let Some(target) = name
                        .strip_prefix("toChecked<")
                        .and_then(|value| value.strip_suffix('>'))
                    {
                        return format!("{target}::try_from({rendered_object}).ok()");
                    }
                }
                match name.as_str() {
                    "checkedAdd" | "checkedSub" | "checkedMul" | "checkedDiv" | "checkedPow"
                    | "wrappingAdd" | "wrappingSub" | "wrappingMul" | "saturatingAdd"
                    | "saturatingSub" | "saturatingMul" => {
                        let method = match name.as_str() {
                            "checkedAdd" => "checked_add",
                            "checkedSub" => "checked_sub",
                            "checkedMul" => "checked_mul",
                            "checkedDiv" => "checked_div",
                            "checkedPow" => "checked_pow",
                            "wrappingAdd" => "wrapping_add",
                            "wrappingSub" => "wrapping_sub",
                            "wrappingMul" => "wrapping_mul",
                            "saturatingAdd" => "saturating_add",
                            "saturatingSub" => "saturating_sub",
                            "saturatingMul" => "saturating_mul",
                            _ => unreachable!(),
                        };
                        return format!(
                            "{rendered_object}.{method}({})",
                            args.iter().map(expr).collect::<Vec<_>>().join(", ")
                        );
                    }
                    "abs" | "pow" | "min" | "max" | "clamp" | "signum" => {
                        return format!(
                            "{rendered_object}.{}({})",
                            match name.as_str() {
                                "abs" => "abs",
                                "pow" => "pow",
                                "min" => "min",
                                "max" => "max",
                                "clamp" => "clamp",
                                _ => "signum",
                            },
                            args.iter().map(expr).collect::<Vec<_>>().join(", ")
                        );
                    }
                    "isEven" if args.is_empty() => {
                        return format!("{rendered_object} % 2 == 0");
                    }
                    "isOdd" if args.is_empty() => {
                        return format!("{rendered_object} % 2 != 0");
                    }
                    "toDouble" if args.is_empty() => {
                        return format!("{rendered_object} as f64");
                    }
                    "toInt" if args.is_empty() => {
                        return format!("{rendered_object} as i64");
                    }
                    "round" | "floor" | "ceil" if args.is_empty() => {
                        return format!("{rendered_object}.{}() as i64", name);
                    }
                    "chars" | "bytes" | "lines" | "trim" if args.is_empty() => {
                        return format!("{rendered_object}.{name}()");
                    }
                    "contains" if args.len() == 1 => {
                        return format!("{rendered_object}.contains(&{})", expr(&args[0]));
                    }
                    "split" if args.len() == 1 => {
                        return format!("{rendered_object}.split(&{})", expr(&args[0]));
                    }
                    "charCount" if args.is_empty() => {
                        return format!("{}.chars().count() as i64", rendered_object);
                    }
                    "startsWith" | "endsWith" if args.len() == 1 => {
                        let method = if name == "startsWith" {
                            "starts_with"
                        } else {
                            "ends_with"
                        };
                        return format!("{rendered_object}.{method}(&{})", expr(&args[0]));
                    }
                    "replaceAll" if args.len() == 2 => {
                        return format!(
                            "{rendered_object}.replace(&{}, &{})",
                            expr(&args[0]),
                            expr(&args[1])
                        );
                    }
                    "toUpperCase" if args.is_empty() => {
                        return format!("{rendered_object}.to_uppercase()");
                    }
                    "toLowerCase" if args.is_empty() => {
                        return format!("{rendered_object}.to_lowercase()");
                    }
                    "indexOf" if args.len() == 1 => {
                        return format!(
                            "{rendered_object}.find(&{}).map(|index| index as i64)",
                            expr(&args[0])
                        );
                    }
                    "substring" if (1..=2).contains(&args.len()) => {
                        let start = expr(&args[0]);
                        let end = args
                            .get(1)
                            .map(expr)
                            .unwrap_or_else(|| format!("{rendered_object}.chars().count() as i64"));
                        return format!(
                            "{rendered_object}.chars().skip({start} as usize).take(({end} - {start}) as usize).collect::<String>()"
                        );
                    }
                    _ => {}
                }
                if name == "add" {
                    return format!(
                        "{}.push({})",
                        expr(object),
                        args.first()
                            .map(expr)
                            .unwrap_or_else(|| "Default::default()".into())
                    );
                }
                if name == "put" {
                    return format!(
                        "{}.insert({})",
                        expr(object),
                        args.iter().map(expr).collect::<Vec<_>>().join(", ")
                    );
                }
                if name == "remove" {
                    return format!(
                        "{}.remove({})",
                        expr(object),
                        args.first()
                            .map(expr)
                            .unwrap_or_else(|| "Default::default()".into())
                    );
                }
                if matches!(object.as_ref(), Expr::Name(value) if value == "Channel")
                    && name == "create"
                {
                    return format!(
                        "orust_runtime::channel({})",
                        args.first().map(expr).unwrap_or_else(|| "16".into())
                    );
                }
                if matches!(object.as_ref(), Expr::Name(value) if value == "Future")
                    && name == "delayed"
                {
                    return format!(
                        "orust_runtime::sleep({})",
                        args.first()
                            .map(expr)
                            .unwrap_or_else(|| "orust_runtime::seconds(0)".into())
                    );
                }
                if matches!(object.as_ref(), Expr::Name(value) if value == "Future")
                    && (name == "wait" || name == "any")
                {
                    let values = args.first().map(expr).unwrap_or_else(|| "vec![]".into());
                    let helper = if name == "wait" {
                        "join_all"
                    } else {
                        "select_all"
                    };
                    return format!("orust_runtime::futures::future::{}({})", helper, values);
                }
            }
            format!(
                "{}({})",
                expr(callee),
                args.iter().map(expr).collect::<Vec<_>>().join(", ")
            )
        }
        Expr::NamedArg { value, .. } => expr(value),
        Expr::Binary { left, op, right } => format!("{} {} {}", expr(left), op, expr(right)),
        Expr::Closure { params, body } => render_closure(params, body, false),
    }
}

fn expr_spawn(value: &Expr) -> String {
    match value {
        Expr::Closure { params, body } => render_closure(params, body, true),
        _ => expr(value),
    }
}

fn render_closure(params: &[String], body: &ClosureBody, move_capture: bool) -> String {
    let prefix = if move_capture { "move " } else { "" };
    match body {
        ClosureBody::Expr(value) => {
            format!("{prefix}|{}| {}", params.join(", "), expr(value))
        }
        ClosureBody::Block(statements) => {
            let mut output = format!("{prefix}|{}| {{\n", params.join(", "));
            emit_block(&mut output, statements, 1);
            output.push('}');
            output
        }
    }
}

fn rust_string_literal(value: &str) -> String {
    let mut escaped = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    escaped.push('"');
    escaped
}

fn expr_try(e: &Expr) -> String {
    match e {
        Expr::Await(value) => match value.as_ref() {
            Expr::Name(_) => format!("{}.await_task().await?", expr(value)),
            _ => format!("{}.await?", expr(value)),
        },
        Expr::Call { callee, args } => format!(
            "{}({})",
            expr_try(callee),
            args.iter().map(expr_try).collect::<Vec<_>>().join(", ")
        ),
        Expr::Member { object, name } => format!("{}.{}", expr_try(object), name),
        Expr::Binary { left, op, right } => {
            format!("{} {} {}", expr_try(left), op, expr_try(right))
        }
        Expr::Borrow { mutable, value } => {
            format!("&{}{}", if *mutable { "mut " } else { "" }, expr_try(value))
        }
        Expr::Copy(value) => format!("{}.clone()", expr_try(value)),
        _ => expr(e),
    }
}

fn emit_try_statement(out: &mut String, statement: &Stmt, indent: usize) {
    let pad = "    ".repeat(indent);
    match statement {
        Stmt::Var {
            name, initializer, ..
        } => out.push_str(&format!(
            "{}let {} = {};\n",
            pad,
            binding_source(name),
            expr_try(initializer)
        )),
        Stmt::PatternVar {
            pattern,
            initializer,
            else_body,
        } => {
            out.push_str(&format!(
                "{}let {} = {} else {{\n",
                pad,
                pattern,
                expr_try(initializer)
            ));
            for nested in else_body {
                emit_try_statement(out, &nested.node, indent + 1);
            }
            out.push_str(&format!("{}}};\n", pad));
        }
        Stmt::Print(value) => out.push_str(&format!(
            "{}println!(\"{{}}\", {});\n",
            pad,
            expr_try(value)
        )),
        Stmt::Return(Some(value)) => out.push_str(&format!("{}return {};\n", pad, expr_try(value))),
        Stmt::Return(None) => out.push_str(&format!("{}return;\n", pad)),
        Stmt::Throw(value) => {
            out.push_str(&format!("{}return Err({}.into());\n", pad, expr_try(value)))
        }
        Stmt::Expr(value) => out.push_str(&format!("{}{};\n", pad, expr_try(value))),
        Stmt::Rust(raw) => out.push_str(&format!("{}{}\n", pad, raw)),
        Stmt::If {
            condition,
            then_body,
            else_body,
        } => {
            out.push_str(&format!("{}if {} {{\n", pad, expr_try(condition)));
            for nested in then_body {
                emit_try_statement(out, &nested.node, indent + 1);
            }
            out.push_str(&format!("{}}}", pad));
            if !else_body.is_empty() {
                out.push_str(" else {\n");
                for nested in else_body {
                    emit_try_statement(out, &nested.node, indent + 1);
                }
                out.push_str(&format!("{}}}", pad));
            }
            out.push('\n');
        }
        Stmt::TryCatch {
            body,
            error,
            catch_body,
        } => emit_try_catch(out, body, error, catch_body, indent),
        Stmt::While { condition, body } => {
            out.push_str(&format!("{}while {} {{\n", pad, expr_try(condition)));
            for nested in body {
                emit_try_statement(out, &nested.node, indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::WhileCase {
            value,
            pattern,
            body,
        } => {
            out.push_str(&format!(
                "{}while let {} = {} {{\n",
                pad,
                pattern,
                expr_try(value)
            ));
            for nested in body {
                emit_try_statement(out, &nested.node, indent + 1);
            }
            out.push_str(&format!("{}}}\n", pad));
        }
        Stmt::For { .. } | Stmt::ForIn { .. } => {
            out.push_str(&format!("{}/* for in try */\n", pad))
        }
        Stmt::Switch { .. } | Stmt::IfCase { .. } => {
            out.push_str(&format!("{}/* complex pattern in try */\n", pad))
        }
        Stmt::AwaitFor { .. } => out.push_str(&format!("{}/* await for in try */\n", pad)),
    }
}

fn emit_try_catch(
    out: &mut String,
    body: &[orust_syntax::Spanned<Stmt>],
    error: &str,
    catch_body: &[orust_syntax::Spanned<Stmt>],
    indent: usize,
) {
    let pad = "    ".repeat(indent);
    out.push_str(&format!("{}match (async {{\n", pad));
    for statement in body {
        emit_try_statement(out, &statement.node, indent + 1);
    }
    out.push_str(&format!(
        "{}    Ok::<(), orust_runtime::Error>(())\n{} }}).await {{\n",
        pad, pad
    ));
    out.push_str(&format!("{}    Ok(_) => {{}},\n", pad));
    out.push_str(&format!("{}    Err({}) => {{\n", pad, error));
    for statement in catch_body {
        emit_simple(out, &statement.node, indent + 2);
    }
    out.push_str(&format!("{}    }}\n{}}}\n", pad, pad));
}

fn render_expr(e: &Expr, fields: Option<&[String]>) -> String {
    let mut rendered = expr(e);
    if let Some(fields) = fields {
        for field in fields {
            rendered = rendered.replace(field, &format!("self.{field}"));
        }
        rendered = rendered.replace("self.self.", "self.");
    }
    rendered
}

fn render_function_expr(
    e: &Expr,
    fields: Option<&[String]>,
    throws: bool,
    call_signatures: &HashMap<String, Vec<Param>>,
    interfaces: &HashSet<String>,
) -> String {
    let rendered = render_expr_with_interfaces(e, fields, call_signatures, interfaces);
    if throws {
        rendered.replace(".await", ".await?")
    } else {
        rendered
    }
}

fn render_expr_with_interfaces(
    e: &Expr,
    fields: Option<&[String]>,
    call_signatures: &HashMap<String, Vec<Param>>,
    interfaces: &HashSet<String>,
) -> String {
    let Expr::Call { callee, args } = e else {
        return render_expr(e, fields);
    };
    let Expr::Name(name) = callee.as_ref() else {
        return render_expr(e, fields);
    };
    let Some(parameters) = call_signatures.get(name) else {
        if fields.is_some() {
            let rendered_args = args
                .iter()
                .map(|argument| render_expr(argument, fields))
                .collect::<Vec<_>>()
                .join(", ");
            return format!("self.{name}({rendered_args})");
        }
        return render_expr(e, fields);
    };
    let rendered_callee = if fields.is_some() && !call_signatures.contains_key(name) {
        format!("self.{name}")
    } else {
        render_expr(callee, fields)
    };
    if parameters
        .iter()
        .any(|parameter| parameter.default.is_some())
    {
        let mut rendered_fields = Vec::new();
        for (index, parameter) in parameters.iter().enumerate() {
            let value = args.iter().find_map(|argument| match argument {
                Expr::NamedArg { name, value } if name == &parameter.name => Some(value.as_ref()),
                _ if !args
                    .iter()
                    .any(|argument| matches!(argument, Expr::NamedArg { .. }))
                    && index < args.len() =>
                {
                    match &args[index] {
                        Expr::NamedArg { .. } => None,
                        value => Some(value),
                    }
                }
                _ => None,
            });
            let rendered = value
                .map(|value| format!("Some({})", render_expr(value, fields)))
                .unwrap_or_else(|| "None".into());
            rendered_fields.push(format!("{}: {}", parameter.name, rendered));
        }
        return format!(
            "{}({}Args {{ {}, ..Default::default() }})",
            rendered_callee,
            name,
            rendered_fields.join(", ")
        );
    }
    let rendered_args = args
        .iter()
        .enumerate()
        .map(|(index, argument)| {
            let rendered = render_expr(argument, fields);
            let Some(expected) = parameters.get(index).map(|parameter| &parameter.ty) else {
                return rendered;
            };
            if interfaces.contains(expected) {
                if matches!(argument, Expr::New(_) | Expr::NewArgs { .. }) {
                    return format!("Box::new({rendered})");
                }
            } else if let Some(inner) = expected
                .strip_prefix("List<")
                .and_then(|value| value.strip_suffix('>'))
            {
                if interfaces.contains(inner) {
                    if let Expr::List(values) = argument {
                        return format!(
                            "vec![{}]",
                            values
                                .iter()
                                .map(|value| {
                                    let rendered = render_expr(value, fields);
                                    if matches!(value, Expr::New(_) | Expr::NewArgs { .. }) {
                                        format!("Box::new({rendered})")
                                    } else {
                                        rendered
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                    }
                }
            }
            rendered
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{rendered_callee}({rendered_args})")
}
