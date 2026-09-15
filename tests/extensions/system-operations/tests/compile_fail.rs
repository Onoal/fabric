#[test]
fn system_macro_reports_definition_integrity_errors() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/*.rs");
}
