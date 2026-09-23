#[test]
fn wrong_resource_adapter_binding_fails_to_compile() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/wrong_resource_adapter.rs");
}

#[test]
fn hostile_component_runtime_definition_override_fails_to_compile() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/hostile_component_runtime_definition_override.rs");
}

#[test]
fn direct_only_resource_cannot_use_adapter_authoring_path() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/direct_only_resource_using_adapter.rs");
}

#[test]
fn resource_adapter_cannot_target_systems() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/resource_adapter_cannot_target_system.rs");
}

#[test]
fn system_adapter_cannot_target_resources() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/system_adapter_cannot_target_resource.rs");
}

#[test]
fn resource_target_adapters_cannot_use_system_schema_support() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/resource_target_adapter_with_system_schema_support.rs");
}

#[test]
fn system_target_adapters_cannot_use_resource_schema_support() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/system_target_adapter_with_resource_schema_support.rs");
}

#[test]
fn canonical_component_runtime_contract_errors_are_diagnostic() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/component_runtime_*.rs");
}

#[test]
fn canonical_component_adapter_runtime_contract_errors_are_diagnostic() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/component_adapter_runtime_*.rs");
}
