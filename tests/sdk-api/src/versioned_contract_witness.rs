use std::sync::{Arc, Mutex};

use fabric_sdk::prelude::*;
use fabric_sdk::{
    authoring::{CompositionExt, FabricBuilder},
    contracts::*,
    core::*,
    ids::*,
};

use crate::support::{
    MismatchedNotesConsumer, NotesCapture, NotesConsumer, RequirementMismatchCapture,
    VersionedNotesProvider, notes_contract_id,
};

#[test]
fn sdk_versioned_contract_authoring_builds_raw_composition_and_preserves_provenance() {
    let provider = VersionedNotesProvider::new(
        "fabric.test.sdk.provider",
        vec![(
            versioned_provider::<crate::support::NotesContract>(notes_contract_id(), "1.4.0")
                .expect("provider"),
            "hello from sdk".to_owned(),
        )],
    );
    let capture: NotesCapture = Arc::new(Mutex::new(None));
    let consumer = NotesConsumer::new(
        "fabric.test.sdk.consumer",
        versioned_requirement::<crate::support::NotesContract>(notes_contract_id(), "^1.2")
            .expect("requirement"),
        Arc::clone(&capture),
    );

    let composition = FabricBuilder::new("fabric.test.sdk.composition")
        .expect("builder")
        .block("runtime", |block| block.module(provider))
        .expect("runtime block")
        .block("consumer", |block| block.module(consumer))
        .expect("consumer block")
        .build()
        .expect("composition");

    takes_raw_composition(&composition);
    let mut instance = composition
        .materialize_named("fabric.test.sdk.instance")
        .expect("instance");
    instance.start().expect("start");
    instance.stop();

    let resolved = capture
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured resolution");
    assert_eq!(resolved.provider.as_str(), "fabric.test.sdk.provider");
    assert_eq!(
        resolved.identity,
        ContractIdentity::versioned(contract_version("1.4.0").expect("version")),
    );
    assert_eq!(resolved.value, "hello from sdk");
}

#[test]
fn sdk_preserves_requirement_declaration_integrity_failures() {
    let provider = VersionedNotesProvider::new(
        "fabric.test.sdk.provider",
        vec![(
            versioned_provider::<crate::support::NotesContract>(notes_contract_id(), "1.4.0")
                .expect("provider"),
            "hello from sdk".to_owned(),
        )],
    );
    let capture: RequirementMismatchCapture = Arc::new(Mutex::new(None));
    let consumer = MismatchedNotesConsumer::new(
        "fabric.test.sdk.consumer",
        versioned_requirement::<crate::support::NotesContract>(notes_contract_id(), "^1")
            .expect("declared requirement"),
        versioned_requirement::<crate::support::NotesContract>(notes_contract_id(), "^2")
            .expect("bind requirement"),
        Arc::clone(&capture),
    );

    let composition = FabricBuilder::new("fabric.test.sdk.composition")
        .expect("builder")
        .block("runtime", |block| block.module(provider))
        .expect("runtime block")
        .block("consumer", |block| block.module(consumer))
        .expect("consumer block")
        .build()
        .expect("declaration graph validates without executing runtime bind");

    let error = composition
        .materialize_named("fabric.test.sdk.instance")
        .expect_err("runtime bind must reject the undeclared requirement");

    assert!(matches!(
        error,
        CompositionError::ModuleFailure { phase: "bind", .. }
    ));
    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(crate::support::CapturedRequirementMismatch {
            module_id: module("fabric.test.sdk.consumer").expect("module id"),
            contract_id: notes_contract_id(),
            declared_compatibility: ContractCompatibilityRequirement::versioned(
                contract_requirement("^1").expect("requirement"),
            ),
            requested_compatibility: ContractCompatibilityRequirement::versioned(
                contract_requirement("^2").expect("requirement"),
            ),
        })
    );
}

#[test]
fn sdk_provider_selection_is_a_thin_authoring_wrapper_over_core() {
    let capture: NotesCapture = Arc::new(Mutex::new(None));
    let provider_a = VersionedNotesProvider::new(
        "fabric.test.sdk.provider.a",
        vec![(
            versioned_provider::<crate::support::NotesContract>(notes_contract_id(), "1.4.0")
                .expect("provider"),
            "from a".to_owned(),
        )],
    );
    let provider_b = VersionedNotesProvider::new(
        "fabric.test.sdk.provider.b",
        vec![(
            versioned_provider::<crate::support::NotesContract>(notes_contract_id(), "1.5.0")
                .expect("provider"),
            "from b".to_owned(),
        )],
    );
    let consumer = NotesConsumer::new(
        "fabric.test.sdk.consumer",
        versioned_requirement::<crate::support::NotesContract>(notes_contract_id(), "^1")
            .expect("requirement"),
        Arc::clone(&capture),
    );

    let selection = ContractProviderSelection::new(
        module("fabric.test.sdk.consumer").expect("consumer"),
        notes_contract_id(),
        module("fabric.test.sdk.provider.b").expect("provider"),
    );
    let composition: fabric_core::Composition = FabricBuilder::new("fabric.test.sdk.composition")
        .expect("builder")
        .block("providers", |block| {
            block.module(provider_a).module(provider_b)
        })
        .expect("providers")
        .block("consumer", |block| block.module(consumer))
        .expect("consumer")
        .select_provider(selection)
        .build()
        .expect("composition");

    let mut instance = composition
        .materialize_named("fabric.test.sdk.instance")
        .expect("instance");
    instance.start().expect("start");

    let resolved = capture
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured resolution");
    assert_eq!(
        resolved.provider,
        module("fabric.test.sdk.provider.b").expect("provider id")
    );
    assert_eq!(resolved.value, "from b");

    instance.stop();
}

fn takes_raw_composition(_composition: &fabric_core::Composition) {}
