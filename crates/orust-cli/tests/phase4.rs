use std::process::Command;

#[test]
fn phase4_negative_constructor_case_is_rejected() {
    let output = Command::new(env!("CARGO_BIN_EXE_orust"))
        .current_dir(env!("CARGO_MANIFEST_DIR").to_string() + "/../..")
        .args(["check", "tests/cases/phase4-invalid-constructor.or"])
        .output()
        .expect("run orust");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("required parameter after a default"));
}
