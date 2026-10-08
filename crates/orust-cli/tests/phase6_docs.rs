use std::process::Command;

fn fixture(name: &str) -> String {
    format!("{}/../../tests/cases/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn todo_authors_and_doc_outputs_are_golden() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let source = fixture("phase6-docs.or");
    let todo = Command::new(binary)
        .args(["todo", &source, "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&todo.stdout),
        "[\"6:6: add persistence\"]\n"
    );
    let authors = Command::new(binary)
        .args(["authors", &source, "--format", "markdown"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&authors.stdout),
        "- Ada Lovelace <ada@example.com>\n- Grace Hopper\n"
    );
    let docs = Command::new(binary)
        .args(["doc", &source])
        .output()
        .unwrap();
    let expected_docs = format!(
        concat!(
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>{}</title></head><body><main>",
            "<article><h2>Phase 6 documentation fixture.</h2><p>Phase 6 documentation fixture.</p>",
            "<p class=\"author\">Author: Ada Lovelace</p></article>",
            "<article><h2>A documented invoice.</h2><p>A documented invoice.</p>",
            "<p class=\"author\">Author: Grace Hopper</p></article></main></body></html>\n"
        ),
        source
    );
    assert_eq!(String::from_utf8_lossy(&docs.stdout), expected_docs);
}

#[test]
fn doctest_errors_map_to_doc_comment_lines() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let source = fixture("phase6-doc-error.or");
    let output = Command::new(binary)
        .args(["test", &source, "--doc"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("doc: "));
    assert!(stderr.contains("phase6-doc-error.or:1:"), "{stderr}");
}

#[test]
fn one_hundred_comments_preserve_doctest_error_line_mapping() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let source = fixture("phase6-doc-error.or");
    let shifted = format!(
        "{}/../../target/phase6-doc-error-shifted.or",
        env!("CARGO_MANIFEST_DIR")
    );
    let original = std::fs::read_to_string(&source).unwrap();
    let prefix = (0..100)
        .map(|index| format!("// generated comment {index}\n"))
        .collect::<String>();
    std::fs::write(&shifted, prefix + &original).unwrap();
    let output = Command::new(binary)
        .args(["test", &shifted, "--doc"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("phase6-doc-error-shifted.or:101:"),
        "{stderr}"
    );
}

#[test]
fn doctest_examples_compile_through_cargo() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let source = fixture("phase6-docs.or");
    let output = Command::new(binary)
        .args(["test", &source, "--doc"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("ok: 1 documentation test(s) in {source}\n")
    );
}

#[test]
fn emit_preserves_plain_comments_by_default() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let source = std::fs::canonicalize(fixture("phase6-comments.or"))
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let output = Command::new(binary)
        .args(["emit", &source])
        .output()
        .unwrap();
    assert!(output.status.success());
    let generated = String::from_utf8_lossy(&output.stdout);
    assert!(
        generated.contains("// Keep this note beside the generated function."),
        "{generated}"
    );
    assert!(
        generated.contains("// Keep this note beside the generated field."),
        "{generated}"
    );
    let semantic = Command::new(binary)
        .args(["emit", &source, "--no-comments"])
        .output()
        .unwrap();
    assert!(semantic.status.success());
    assert!(!String::from_utf8_lossy(&semantic.stdout)
        .contains("// Keep this note beside the generated function."));
}

#[test]
fn phase6_acceptance_program_and_doctest_compile() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let source = std::fs::canonicalize(fixture("phase6-acceptance.or"))
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let workspace = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let unit = Command::new(binary)
        .current_dir(&workspace)
        .args(["test", &source])
        .output()
        .unwrap();
    assert!(
        unit.status.success(),
        "{}",
        String::from_utf8_lossy(&unit.stderr)
    );
    let docs = Command::new(binary)
        .current_dir(&workspace)
        .args(["test", &source, "--doc"])
        .output()
        .unwrap();
    assert!(
        docs.status.success(),
        "{}",
        String::from_utf8_lossy(&docs.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&docs.stdout),
        format!("ok: 1 documentation test(s) in {source}\n")
    );
}

#[test]
fn rustdoc_command_builds_the_generated_project() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let source = std::fs::canonicalize(fixture("phase6-acceptance.or"))
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let workspace = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(binary)
        .current_dir(&workspace)
        .args(["doc", &source, "--rustdoc"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("warning:"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("target/rust/target/doc"));
}

#[test]
fn eject_output_is_a_checkable_cargo_project() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let source = fixture("phase6-docs.or");
    let output_dir = format!(
        "{}/../../target/phase6-ejected-test",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(binary)
        .args(["eject", &source, "--out", &output_dir])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(std::path::Path::new(&output_dir)
        .join("Cargo.toml")
        .exists());
    let manifest =
        std::fs::read_to_string(std::path::Path::new(&output_dir).join("Cargo.toml")).unwrap();
    assert!(manifest.starts_with(
        "[workspace]\n\n[package]\nname = \"orust-ejected\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n\n[dependencies]\n"
    ));
    assert!(manifest.contains("orust-runtime = { path = \""));
    assert_eq!(
        std::fs::read_to_string(std::path::Path::new(&output_dir).join("src/lib.rs")).unwrap(),
        "/// A documented invoice.\n///\n/// # Arguments\n/// - `amount`: cents\n///\n/// # Returns\n/// the invoice\n///\n/// # Examples\n/// smoke\n/// ```orust\n/// print(1);\n/// ```\n///\n/// # Authors\n/// - Grace Hopper\npub fn invoice(amount: i64) -> () {\n    println!(\"{}\", amount);\n}\n"
    );
    assert_eq!(
        std::fs::read_to_string(std::path::Path::new(&output_dir).join("EJECT_NOTES.md")).unwrap(),
        format!(
            "# Eject notes\n\nGenerated from `{source}`.\n\nORust documentation and comments were preserved where Rustdoc has an equivalent.\n"
        )
    );
}

#[test]
fn formatter_is_idempotent_with_comments() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let source = fixture("phase6-docs.or");
    let first = Command::new(binary)
        .args(["fmt", &source, "--check"])
        .output()
        .unwrap();
    let second = Command::new(binary)
        .args(["fmt", &source, "--check"])
        .output()
        .unwrap();
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn formatter_removes_trailing_whitespace_without_moving_comments() {
    let binary = env!("CARGO_BIN_EXE_orust");
    let path = format!(
        "{}/../../target/phase6-format-comments.or",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::write(
        &path,
        "// keep this comment  \r\nvoid main() {  \r\n  print(1); // keep this too  \r\n}\r\n",
    )
    .unwrap();
    let output = Command::new(binary).args(["fmt", &path]).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "// keep this comment\r\nvoid main() {\r\n  print(1); // keep this too\r\n}\r\n"
    );
    let check = Command::new(binary)
        .args(["fmt", &path, "--check"])
        .output()
        .unwrap();
    assert!(check.status.success());
}
