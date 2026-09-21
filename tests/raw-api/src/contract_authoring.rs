use std::sync::{Arc, Mutex};

use fabric_core::{
    BlockBuilder, BlockId, CompositionBuilder, CompositionError, CompositionId,
    ContractCompatibilityRequirement, ContractIdentity, ContractProviderSelection, ContractVersion,
    ContractVersionRequirement, InstanceId, Module, ModuleBindings, ModuleContract,
    ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};

use crate::support::{
    MismatchedNotesConsumer, NotesCapture, NotesConsumer, OptionalNotesCapture,
    OptionalNotesConsumer, RequirementMismatchCapture, VersionedNotesProvider,
    captured_optional_resolution, captured_resolution, notes_contract_id, notes_contract_key,
    notes_requirement,
};

#[test]
fn downstream_module_authoring_uses_versioned_contract_declarations_only() {
    let provider = VersionedNotesProvider::new(
        "fabric.test.raw.provider",
        vec![(notes_contract_key("1.4.0"), "hello from raw api".to_owned())],
    );
    let capture: NotesCapture = Arc::new(Mutex::new(None));
    let consumer = NotesConsumer::new(
        "fabric.test.raw.consumer",
        notes_requirement("^1.2"),
        capture.clone(),
    );

    let composition = single_block_composition(
        "fabric.test.raw.composition",
        vec![dynamic_module(provider), dynamic_module(consumer)],
        Vec::new(),
    )
    .expect("composition");
    let mut instance = composition
        .materialize(InstanceId::new("fabric-test-raw-instance".to_owned()).expect("instance"))
        .expect("instance");
    instance.start().expect("start instance");

    let resolved = captured_resolution(&capture);
    assert_eq!(resolved.provider.as_str(), "fabric.test.raw.provider");
    assert_eq!(
        resolved.identity,
        ContractIdentity::versioned(ContractVersion::parse("1.4.0").expect("version")),
    );
    assert_eq!(resolved.value, "hello from raw api");

    instance.stop().expect("stop instance");
}

#[test]
fn downstream_module_cannot_bind_a_requirement_that_differs_from_its_declaration() {
    let provider = VersionedNotesProvider::new(
        "fabric.test.raw.provider",
        vec![(notes_contract_key("1.4.0"), "hello from raw api".to_owned())],
    );
    let capture: RequirementMismatchCapture = Arc::new(Mutex::new(None));
    let consumer = MismatchedNotesConsumer::new(
        "fabric.test.raw.consumer",
        notes_requirement("^1.2"),
        notes_requirement("^2"),
        Arc::clone(&capture),
    );

    let composition = single_block_composition(
        "fabric.test.raw.composition",
        vec![dynamic_module(provider), dynamic_module(consumer)],
        Vec::new(),
    )
    .expect("declarations should validate");
    let error = composition
        .materialize(
            InstanceId::new("fabric.test.raw.requirement-drift".to_owned()).expect("instance"),
        )
        .expect_err("runtime binding should reject requirement drift");

    assert!(matches!(
        error,
        CompositionError::ModuleFailure { phase: "bind", .. }
    ));
    assert_eq!(
        capture.lock().expect("capture lock").clone(),
        Some(crate::support::CapturedRequirementMismatch {
            module_id: ModuleId::new("fabric.test.raw.consumer".to_owned()).expect("module id"),
            contract_id: notes_contract_id(),
            declared_compatibility: ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^1.2").expect("requirement"),
            ),
            requested_compatibility: ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^2").expect("requirement"),
            ),
        })
    );
}

#[test]
fn ambiguous_provider_resolution_stays_default_without_explicit_selection() {
    let error = single_block_composition(
        "fabric.test.raw.ambiguous",
        vec![
            dynamic_module(VersionedNotesProvider::new(
                "fabric.test.raw.provider.a",
                vec![(notes_contract_key("1.4.0"), "from a".to_owned())],
            )),
            dynamic_module(VersionedNotesProvider::new(
                "fabric.test.raw.provider.b",
                vec![(notes_contract_key("1.5.0"), "from b".to_owned())],
            )),
            dynamic_module(NotesConsumer::new(
                "fabric.test.raw.consumer",
                notes_requirement("^1"),
                Arc::new(Mutex::new(None)),
            )),
        ],
        Vec::new(),
    )
    .expect_err("composition should stay ambiguous without explicit selection");

    assert!(matches!(
        error,
        CompositionError::AmbiguousProvider {
            module_id,
            contract_id,
            providers
        } if module_id.as_str() == "fabric.test.raw.consumer"
            && contract_id == notes_contract_id()
            && providers == vec![
                ModuleId::new("fabric.test.raw.provider.a".to_owned()).expect("provider a"),
                ModuleId::new("fabric.test.raw.provider.b".to_owned()).expect("provider b"),
            ]
    ));
}

#[test]
fn explicit_provider_selection_chooses_the_selected_provider() {
    let capture: NotesCapture = Arc::new(Mutex::new(None));
    let composition = single_block_composition(
        "fabric.test.raw.selected-a",
        vec![
            dynamic_module(VersionedNotesProvider::new(
                "fabric.test.raw.provider.a",
                vec![(notes_contract_key("1.4.0"), "from a".to_owned())],
            )),
            dynamic_module(VersionedNotesProvider::new(
                "fabric.test.raw.provider.b",
                vec![(notes_contract_key("1.5.0"), "from b".to_owned())],
            )),
            dynamic_module(NotesConsumer::new(
                "fabric.test.raw.consumer",
                notes_requirement("^1"),
                Arc::clone(&capture),
            )),
        ],
        vec![selection(
            "fabric.test.raw.consumer",
            "fabric.test.raw.provider.a",
        )],
    )
    .expect("composition");

    let mut instance = composition
        .materialize(InstanceId::new("fabric-test-raw-selected-a".to_owned()).expect("instance"))
        .expect("instance");
    instance.start().expect("start");

    let resolved = captured_resolution(&capture);
    assert_eq!(resolved.provider.as_str(), "fabric.test.raw.provider.a");
    assert_eq!(resolved.value, "from a");

    instance.stop().expect("stop instance");
}

#[test]
fn provider_selection_is_scoped_per_consumer() {
    let capture_one: NotesCapture = Arc::new(Mutex::new(None));
    let capture_two: NotesCapture = Arc::new(Mutex::new(None));
    let composition = single_block_composition(
        "fabric.test.raw.two-consumers",
        vec![
            dynamic_module(VersionedNotesProvider::new(
                "fabric.test.raw.provider.a",
                vec![(notes_contract_key("1.4.0"), "from a".to_owned())],
            )),
            dynamic_module(VersionedNotesProvider::new(
                "fabric.test.raw.provider.b",
                vec![(notes_contract_key("1.5.0"), "from b".to_owned())],
            )),
            dynamic_module(NotesConsumer::new(
                "fabric.test.raw.consumer.one",
                notes_requirement("^1"),
                Arc::clone(&capture_one),
            )),
            dynamic_module(NotesConsumer::new(
                "fabric.test.raw.consumer.two",
                notes_requirement("^1"),
                Arc::clone(&capture_two),
            )),
        ],
        vec![
            selection("fabric.test.raw.consumer.one", "fabric.test.raw.provider.a"),
            selection("fabric.test.raw.consumer.two", "fabric.test.raw.provider.b"),
        ],
    )
    .expect("composition");

    let mut instance = composition
        .materialize(InstanceId::new("fabric-test-raw-two-consumers".to_owned()).expect("id"))
        .expect("instance");
    instance.start().expect("start");

    assert_eq!(
        captured_resolution(&capture_one).provider.as_str(),
        "fabric.test.raw.provider.a"
    );
    assert_eq!(
        captured_resolution(&capture_two).provider.as_str(),
        "fabric.test.raw.provider.b"
    );

    instance.stop().expect("stop instance");
}

#[test]
fn explicit_selection_never_falls_back_to_another_compatible_provider() {
    let error = single_block_composition(
        "fabric.test.raw.selected-incompatible",
        vec![
            dynamic_module(VersionedNotesProvider::new(
                "fabric.test.raw.provider.v1",
                vec![(notes_contract_key("1.4.0"), "v1".to_owned())],
            )),
            dynamic_module(VersionedNotesProvider::new(
                "fabric.test.raw.provider.v2",
                vec![(notes_contract_key("2.1.0"), "v2".to_owned())],
            )),
            dynamic_module(NotesConsumer::new(
                "fabric.test.raw.consumer",
                notes_requirement("^1"),
                Arc::new(Mutex::new(None)),
            )),
        ],
        vec![selection(
            "fabric.test.raw.consumer",
            "fabric.test.raw.provider.v2",
        )],
    )
    .expect_err("incompatible explicit selection must fail");

    assert!(matches!(
        error,
        CompositionError::SelectedProviderIncompatible {
            consumer,
            contract_id,
            provider,
            required_compatibility,
            available_identities,
        } if consumer.as_str() == "fabric.test.raw.consumer"
            && contract_id == notes_contract_id()
            && provider.as_str() == "fabric.test.raw.provider.v2"
            && required_compatibility == ContractCompatibilityRequirement::versioned(
                ContractVersionRequirement::parse("^1").expect("requirement"),
            )
            && available_identities == vec![
                ContractIdentity::versioned(ContractVersion::parse("2.1.0").expect("version")),
            ]
    ));
}

#[test]
fn selected_provider_keeps_version_ambiguity_when_it_exports_multiple_matches() {
    let error = single_block_composition(
        "fabric.test.raw.selected-provider-ambiguous",
        vec![
            dynamic_module(VersionedNotesProvider::new(
                "fabric.test.raw.provider",
                vec![
                    (notes_contract_key("1.3.0"), "v1.3".to_owned()),
                    (notes_contract_key("1.4.0"), "v1.4".to_owned()),
                ],
            )),
            dynamic_module(NotesConsumer::new(
                "fabric.test.raw.consumer",
                notes_requirement("^1"),
                Arc::new(Mutex::new(None)),
            )),
        ],
        vec![selection(
            "fabric.test.raw.consumer",
            "fabric.test.raw.provider",
        )],
    )
    .expect_err("selected provider should remain ambiguous across multiple versions");

    assert!(matches!(
        error,
        CompositionError::AmbiguousProvider {
            module_id,
            contract_id,
            providers,
        } if module_id.as_str() == "fabric.test.raw.consumer"
            && contract_id == notes_contract_id()
            && providers == vec![
                ModuleId::new("fabric.test.raw.provider".to_owned()).expect("provider"),
                ModuleId::new("fabric.test.raw.provider".to_owned()).expect("provider"),
            ]
    ));
}

#[test]
fn optional_requirement_without_selection_stays_optional() {
    let capture: OptionalNotesCapture = Arc::new(Mutex::new(None));
    let composition = single_block_composition(
        "fabric.test.raw.optional",
        vec![dynamic_module(OptionalNotesConsumer::new(
            "fabric.test.raw.consumer",
            notes_requirement("^1"),
            Arc::clone(&capture),
        ))],
        Vec::new(),
    )
    .expect("composition");

    let mut instance = composition
        .materialize(InstanceId::new("fabric-test-raw-optional".to_owned()).expect("instance"))
        .expect("instance");
    instance.start().expect("start");

    assert_eq!(captured_optional_resolution(&capture), None);

    instance.stop().expect("stop instance");
}

#[test]
fn explicit_selection_for_optional_requirement_is_enforced() {
    let error = single_block_composition(
        "fabric.test.raw.optional-selected",
        vec![dynamic_module(OptionalNotesConsumer::new(
            "fabric.test.raw.consumer",
            notes_requirement("^1"),
            Arc::new(Mutex::new(None)),
        ))],
        vec![selection(
            "fabric.test.raw.consumer",
            "fabric.test.raw.provider",
        )],
    )
    .expect_err("missing selected provider must fail even for optional requirements");

    assert!(matches!(
        error,
        CompositionError::UnknownSelectionProvider {
            consumer,
            contract_id,
            provider,
        } if consumer.as_str() == "fabric.test.raw.consumer"
            && contract_id == notes_contract_id()
            && provider.as_str() == "fabric.test.raw.provider"
    ));
}

#[test]
fn invalid_provider_selections_fail_structurally() {
    let notes_id = notes_contract_id();
    let provider = dynamic_module(VersionedNotesProvider::new(
        "fabric.test.raw.provider",
        vec![(notes_contract_key("1.4.0"), "hello".to_owned())],
    ));
    let consumer = dynamic_module(NotesConsumer::new(
        "fabric.test.raw.consumer",
        notes_requirement("^1"),
        Arc::new(Mutex::new(None)),
    ));

    assert!(matches!(
        single_block_composition(
            "fabric.test.raw.unknown-consumer",
            vec![provider.clone(), consumer.clone()],
            vec![selection("fabric.test.raw.missing", "fabric.test.raw.provider")],
        )
        .expect_err("unknown consumer"),
        CompositionError::UnknownSelectionConsumer {
            consumer,
            contract_id,
            provider,
        } if consumer.as_str() == "fabric.test.raw.missing"
            && contract_id == notes_id
            && provider.as_str() == "fabric.test.raw.provider"
    ));

    assert!(matches!(
        single_block_composition(
            "fabric.test.raw.unknown-provider",
            vec![provider.clone(), consumer.clone()],
            vec![selection("fabric.test.raw.consumer", "fabric.test.raw.missing")],
        )
        .expect_err("unknown provider"),
        CompositionError::UnknownSelectionProvider {
            consumer,
            contract_id,
            provider,
        } if consumer.as_str() == "fabric.test.raw.consumer"
            && contract_id == notes_id
            && provider.as_str() == "fabric.test.raw.missing"
    ));

    assert!(matches!(
        single_block_composition(
            "fabric.test.raw.undeclared-selection",
            vec![
                provider.clone(),
                dynamic_module(IdleModule::new("fabric.test.raw.consumer")),
            ],
            vec![selection("fabric.test.raw.consumer", "fabric.test.raw.provider")],
        )
        .expect_err("selection must target a declared requirement"),
        CompositionError::SelectedUndeclaredRequirement {
            consumer,
            contract_id,
            provider,
        } if consumer.as_str() == "fabric.test.raw.consumer"
            && contract_id == notes_id
            && provider.as_str() == "fabric.test.raw.provider"
    ));

    assert!(matches!(
        single_block_composition(
            "fabric.test.raw.provider-missing-contract",
            vec![
                dynamic_module(EmptyProvider::new("fabric.test.raw.provider")),
                consumer.clone(),
            ],
            vec![selection("fabric.test.raw.consumer", "fabric.test.raw.provider")],
        )
        .expect_err("selected provider must declare the contract"),
        CompositionError::SelectedProviderMissingContract {
            consumer,
            contract_id,
            provider,
        } if consumer.as_str() == "fabric.test.raw.consumer"
            && contract_id == notes_id
            && provider.as_str() == "fabric.test.raw.provider"
    ));

    assert!(matches!(
        single_block_composition(
            "fabric.test.raw.duplicate-selection",
            vec![
                provider,
                consumer,
                dynamic_module(VersionedNotesProvider::new(
                    "fabric.test.raw.provider.b",
                    vec![(notes_contract_key("1.5.0"), "other".to_owned())],
                )),
            ],
            vec![
                selection("fabric.test.raw.consumer", "fabric.test.raw.provider"),
                selection("fabric.test.raw.consumer", "fabric.test.raw.provider.b"),
            ],
        )
        .expect_err("duplicate selections"),
        CompositionError::DuplicateContractProviderSelection {
            consumer,
            contract_id,
            first_provider,
            second_provider,
        } if consumer.as_str() == "fabric.test.raw.consumer"
            && contract_id == notes_id
            && first_provider.as_str() == "fabric.test.raw.provider"
            && second_provider.as_str() == "fabric.test.raw.provider.b"
    ));
}

fn single_block_composition(
    composition_id: &str,
    modules: Vec<DynModuleFactory>,
    selections: Vec<ContractProviderSelection>,
) -> Result<fabric_core::Composition, CompositionError> {
    let mut block =
        BlockBuilder::new(BlockId::new("fabric.test.raw.block".to_owned()).expect("block"));
    for module in modules {
        block = block.register_module(module);
    }

    let mut builder = CompositionBuilder::new(
        CompositionId::new(composition_id.to_owned()).expect("composition id"),
    )
    .register_block(block.build());
    for selection in selections {
        builder = builder.select_provider(selection);
    }
    builder.build()
}

#[derive(Clone)]
struct DynModuleFactory {
    declaration: ModuleDeclaration,
    inner: Arc<dyn Fn() -> Box<dyn ModuleRuntime> + Send + Sync>,
}

impl Module for DynModuleFactory {
    fn declaration(&self) -> ModuleDeclaration {
        self.declaration.clone()
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some((self.inner)())
    }
}

fn dynamic_module<M>(module: M) -> DynModuleFactory
where
    M: ModuleRuntime + Clone + Send + Sync + 'static,
{
    DynModuleFactory {
        declaration: ModuleDeclaration::from_runtime(&module),
        inner: Arc::new(move || Box::new(module.clone())),
    }
}

fn selection(consumer: &str, provider: &str) -> ContractProviderSelection {
    ContractProviderSelection::new(
        ModuleId::new(consumer.to_owned()).expect("consumer id"),
        notes_contract_id(),
        ModuleId::new(provider.to_owned()).expect("provider id"),
    )
}

#[derive(Clone)]
struct IdleModule {
    module_id: ModuleId,
}

impl IdleModule {
    fn new(module_id: &str) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
        }
    }
}

impl ModuleRuntime for IdleModule {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> fabric_core::Health {
        fabric_core::Health::Healthy
    }
}

#[derive(Clone)]
struct EmptyProvider {
    module_id: ModuleId,
}

impl EmptyProvider {
    fn new(module_id: &str) -> Self {
        Self {
            module_id: ModuleId::new(module_id.to_owned()).expect("module id"),
        }
    }
}

impl ModuleRuntime for EmptyProvider {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn health(&self) -> fabric_core::Health {
        fabric_core::Health::Healthy
    }
}
