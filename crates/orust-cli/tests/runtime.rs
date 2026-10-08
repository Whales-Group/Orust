use std::{
    process::Command,
    sync::{Mutex, OnceLock},
};

static RUN_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn run_case(case: &str) -> String {
    let _guard = RUN_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_orust"))
        .current_dir(env!("CARGO_MANIFEST_DIR").to_string() + "/../..")
        .args(["run", case])
        .output()
        .expect("run ORust case");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 stdout")
}

#[test]
fn counter_stdout_is_deterministic() {
    assert_eq!(run_case("tests/cases/counter.or"), "1\n1\n");
}

#[test]
fn async_stdout_preserves_await_order() {
    assert_eq!(run_case("tests/cases/async-deterministic.or"), "1\n2\n");
}

#[test]
fn rust_scope_values_can_be_used_and_mutated_by_orust() {
    assert_eq!(run_case("tests/cases/rust-scope-interop.or"), "42\n");
}

#[test]
fn phase4_acceptance_program_runs() {
    assert_eq!(
        run_case("tests/cases/phase4-acceptance.or"),
        "phase4\n5\nnull\n2\n3\n"
    );
}

#[test]
fn phase5_recursive_enum_acceptance_program_runs() {
    assert_eq!(
        run_case("tests/cases/phase5-recursive-acceptance.or"),
        "14\n"
    );
}

#[test]
fn phase5_linked_list_acceptance_program_runs() {
    assert_eq!(run_case("tests/cases/phase5-linked-list.or"), "3\n2\n");
}

#[test]
fn phase5_bst_acceptance_program_runs() {
    assert_eq!(
        run_case("tests/cases/phase5-bst-acceptance.or"),
        "2\n1\n3\n"
    );
}

#[test]
fn phase5_iterator_acceptance_program_runs() {
    assert_eq!(
        run_case("tests/cases/phase5-iterators-acceptance.or"),
        "3\n2\n1\nnull\n3\n"
    );
}

#[test]
fn phase5_records_acceptance_program_runs() {
    assert_eq!(run_case("tests/cases/phase5-records.or"), "7\nrecord\n");
}

#[test]
fn phase5_newtype_acceptance_program_runs() {
    assert_eq!(run_case("tests/cases/phase5-newtype.or"), "7\n");
}
