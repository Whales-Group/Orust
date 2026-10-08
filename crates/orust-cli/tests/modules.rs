use std::sync::Mutex;
use std::{fs, process::Command};

static PROJECT_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn runs_a_multi_file_project_with_explicit_imports() {
    let _lock = PROJECT_LOCK.lock().unwrap();
    let binary = env!("CARGO_BIN_EXE_orust");
    let project = format!(
        "{}/../../tests/projects/module-basic/main.or",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(binary)
        .args(["run", &project])
        .output()
        .expect("run orust");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "0\n");
}

#[test]
fn preserves_orust_names_for_cross_file_rust_name_imports() {
    let _lock = PROJECT_LOCK.lock().unwrap();
    let binary = env!("CARGO_BIN_EXE_orust");
    let project = format!(
        "{}/../../tests/projects/module-rust-name/main.or",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(binary)
        .args(["run", &project])
        .output()
        .expect("run ORust rust-name project");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "7\n");
}

#[test]
fn runs_a_mixed_or_and_rust_project() {
    let _lock = PROJECT_LOCK.lock().unwrap();
    let binary = env!("CARGO_BIN_EXE_orust");
    let project = format!(
        "{}/../../tests/projects/mixed-rust/main.or",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(binary)
        .args(["run", &project])
        .output()
        .expect("run orust");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "HELLO\n");
}

#[test]
fn runs_a_cargo_manifest_project_without_orust_toml() {
    let _lock = PROJECT_LOCK.lock().unwrap();
    let binary = env!("CARGO_BIN_EXE_orust");
    let project = format!(
        "{}/../../tests/projects/cargo-basic/src/main.or",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(binary)
        .args(["run", &project])
        .output()
        .expect("run orust");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "\"Cargo.toml\"\n");
}

#[test]
fn generated_library_preserves_the_cargo_package_identity() {
    let _lock = PROJECT_LOCK.lock().unwrap();
    let binary = env!("CARGO_BIN_EXE_orust");
    let project = format!(
        "{}/../../tests/projects/cargo-basic/src/main.or",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(binary)
        .args(["build", "--lib", &project])
        .output()
        .expect("build ORust library");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest = fs::read_to_string("target/orust/Cargo.toml").expect("generated manifest");
    assert!(manifest.contains("name = \"cargo-basic\""));
}

#[test]
fn rust_cargo_crate_can_consume_a_generated_orust_library() {
    let _lock = PROJECT_LOCK.lock().unwrap();
    let binary = env!("CARGO_BIN_EXE_orust");
    let library = format!(
        "{}/../../tests/projects/orust-library/src/lib.or",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(binary)
        .current_dir(format!("{}/../..", env!("CARGO_MANIFEST_DIR")))
        .args(["build", "--lib", &library, "--features", "interop"])
        .output()
        .expect("build ORust library");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated_manifest = fs::read_to_string(format!(
        "{}/../../target/orust/Cargo.toml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("generated manifest");
    assert!(generated_manifest.contains("default = [\"interop\"]"));
    let consumer_manifest = format!(
        "{}/../../tests/projects/rust-consumer/Cargo.toml",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--offline",
            "--manifest-path",
            &consumer_manifest,
        ])
        .output()
        .expect("run Rust consumer");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "hello Rust\nlabel ORust\n"
    );
}

#[test]
fn shares_named_record_shapes_across_orust_modules() {
    let _lock = PROJECT_LOCK.lock().unwrap();
    let binary = env!("CARGO_BIN_EXE_orust");
    let project = format!(
        "{}/../../tests/projects/records-cross-file/main.or",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(binary)
        .args(["run", &project])
        .output()
        .expect("run orust");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "9\nshared\n");
}
