#[test]
fn resource_macro_reports_definition_integrity_errors() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/legacy_contracts_removed.rs");
    cases.compile_fail("tests/ui/legacy_whole_realization_removed.rs");
}
