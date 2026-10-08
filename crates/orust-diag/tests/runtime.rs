#[test]
fn rewrites_shared_borrow_panics() {
    assert!(
        orust_diag::rewrite_runtime_panic("already borrowed: BorrowMutError")
            .contains("shared class")
    );
    assert_eq!(
        orust_diag::rewrite_runtime_panic("ordinary error"),
        "ordinary error"
    );
}

#[test]
fn rewrites_string_boundary_panics() {
    assert_eq!(
        orust_diag::rewrite_runtime_panic("byte index 3 is not a char boundary"),
        "byte 3 is in the middle of a character"
    );
}
