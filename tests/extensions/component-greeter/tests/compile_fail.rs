#[test]
fn component_macro_reports_canonical_definition_and_removed_frontend_errors() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/component_api_handler_is_not_declaration.rs");
    cases.compile_fail("tests/ui/duplicate_component_api_method.rs");
    cases.compile_fail("tests/ui/duplicate_component_relation_role.rs");
    cases.compile_fail("tests/ui/removed_operations.rs");
    cases.compile_fail("tests/ui/removed_requires.rs");
    cases.compile_fail("tests/ui/removed_system.rs");
    cases.compile_fail("tests/ui/removed_teardown.rs");
}
