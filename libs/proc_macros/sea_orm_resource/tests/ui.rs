//! UI tests for compile-time error checking

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/missing_table_name.rs");
    t.pass("tests/ui/basic.rs");
}
