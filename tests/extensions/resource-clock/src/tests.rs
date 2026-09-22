use fabric::authoring::{
    AdaptableResourceDefinition, PrimaryResourceContract, Requires, ResourceDefinition,
};
use fabric_core::{
    ContractRequirementDeclaration, ContractVersionRequirement, ModuleRuntime,
    ProvidedContractDeclaration,
};

use crate::{
    Clock, ClockConfig, NativeClock, clock_contract_id, clock_contract_version,
    clock_realization_contract_id, clock_realization_contract_key,
    clock_realization_contract_version, clock_realization_contract_version_requirement,
    clock_resource_id, clock_schema_version,
};

#[test]
fn clock_resource_owns_distinct_outward_and_realization_contracts() {
    let module = NativeClock::new("clock.module", ClockConfig::default());

    assert_eq!(clock_resource_id().as_str(), "fabric.test.clock");
    assert_eq!(clock_contract_id().as_str(), "fabric.test.resource.clock");
    assert_eq!(
        clock_realization_contract_id().as_str(),
        "fabric.test.resource.clock.realization"
    );
    assert_ne!(clock_contract_id(), clock_realization_contract_id());
    assert_eq!(
        module.provided_contract_declarations(),
        vec![ProvidedContractDeclaration::versioned(
            clock_contract_id(),
            clock_contract_version(),
        )]
    );
    assert_eq!(
        module.required_contract_declarations(),
        vec![ContractRequirementDeclaration::versioned(
            clock_realization_contract_id(),
            clock_realization_contract_version_requirement(),
        )]
    );
    assert_eq!(
        Clock::primary_contract_key().declaration(),
        ProvidedContractDeclaration::versioned(clock_contract_id(), clock_contract_version())
    );
    assert_eq!(
        Clock::realization_requirement().declaration(),
        &ContractRequirementDeclaration::versioned(
            clock_realization_contract_id(),
            clock_realization_contract_version_requirement(),
        )
    );
    assert_eq!(
        clock_realization_contract_key().declaration(),
        ProvidedContractDeclaration::versioned(
            clock_realization_contract_id(),
            clock_realization_contract_version(),
        )
    );
}

#[test]
fn clock_resource_exposes_unknown_resource_identity_and_versioned_schema() {
    let selection = Clock::select("primary", ClockConfig::default()).expect("selection");

    assert_eq!(Clock::resource_id(), clock_resource_id());
    assert_eq!(
        Clock::schema(),
        fabric_resource::ResourceSchemaDescriptor::versioned(
            clock_resource_id(),
            clock_schema_version(),
        )
    );
    assert_eq!(selection.name().as_str(), "primary");
}

#[test]
fn requires_clock_public_apis_always_target_the_primary_contract_id() {
    let provisional = Requires::<Clock>::provisional();
    let versioned = Requires::<Clock>::versioned(
        ContractVersionRequirement::parse("^1.2").expect("version requirement"),
    );

    assert_eq!(
        provisional.as_contract_requirement().id(),
        Clock::primary_contract_key().id()
    );
    assert_eq!(
        versioned.as_contract_requirement().id(),
        Clock::primary_contract_key().id()
    );
    assert_eq!(
        provisional.declaration(),
        &ContractRequirementDeclaration::provisional(clock_contract_id())
    );
    assert_eq!(
        versioned.declaration(),
        &ContractRequirementDeclaration::versioned(
            clock_contract_id(),
            ContractVersionRequirement::parse("^1.2").expect("version requirement"),
        )
    );
}

#[test]
fn resource_selection_derives_total_deterministic_module_ids_for_valid_resource_names() {
    let primary = Clock::select("primary", ClockConfig::default()).expect("primary selection");
    let primary_again =
        Clock::select("primary", ClockConfig::default()).expect("primary selection");
    let edge = Clock::select("edge@home", ClockConfig::default()).expect("edge selection");
    let unicode = Clock::select("naive-東京", ClockConfig::default()).expect("unicode selection");

    assert_eq!(primary.module_id(), primary_again.module_id());
    assert_ne!(primary.module_id(), edge.module_id());
    assert_ne!(edge.module_id(), unicode.module_id());
    assert_eq!(
        primary.module_id().as_str(),
        expected_module_id_for_name("primary")
    );
    assert_eq!(
        edge.module_id().as_str(),
        expected_module_id_for_name("edge@home")
    );
    assert_eq!(
        unicode.module_id().as_str(),
        expected_module_id_for_name("naive-東京")
    );
}

#[test]
fn clock_remains_a_handwritten_resource_definition() {
    let definition =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/definition.rs"))
            .expect("read clock definition");

    assert!(
        definition.contains("impl ResourceDefinition for Clock"),
        "Clock should remain a handwritten Resource definition"
    );
    assert!(
        !definition.contains("fabric::resource!"),
        "Clock should remain a handwritten Resource definition"
    );
}

fn expected_module_id_for_name(name: &str) -> String {
    format!(
        "{}.selection.{}",
        clock_resource_id().as_str(),
        name.as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}
