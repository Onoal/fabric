mod bindings;
mod graph;
mod selection;

pub use bindings::{ModuleBindings, ResolvedContract};
use graph::dependency_order;
pub use selection::ContractProviderSelection;

use std::collections::{BTreeMap, BTreeSet};

use fabric_host::HostDescriptor;

use crate::block::{Block, RuntimeBlock};
use crate::contract::{
    ContractRequirementDeclaration, ModuleContract, ProvidedContractDeclaration,
};
use crate::error::CompositionError;
use crate::export::{CompositionExport, CompositionExportDeclaration};
use crate::host_materialization::HostMaterializationRequirement;
use crate::identifiers::{CompositionId, ContractId, InstanceId, ModuleId};
use crate::instance::{Instance, InstanceRuntimeContext};
use crate::{ModuleDeclaration, ModuleRuntime};

type RuntimeBlockLocation = (usize, usize);
type MaterializedRuntimeBlocks = (
    Vec<RuntimeBlock>,
    Vec<RuntimeBlockLocation>,
    BTreeMap<ContractId, ModuleContract>,
);
type ProviderSelections = BTreeMap<(ModuleId, ContractId), ModuleId>;

#[derive(Clone)]
pub(crate) struct DeclaredBinding {
    pub(crate) provider: ModuleId,
    pub(crate) declaration: ProvidedContractDeclaration,
}

pub(crate) type DeclaredBindings = BTreeMap<ModuleId, BTreeMap<ContractId, DeclaredBinding>>;

struct ValidatedDeclarations {
    declarations: Vec<ModuleDeclaration>,
    bindings: DeclaredBindings,
    start_order: Vec<usize>,
}

pub struct CompositionBuilder {
    composition_id: CompositionId,
    blocks: Vec<Block>,
    provider_selections: Vec<ContractProviderSelection>,
    exports: Vec<CompositionExportDeclaration>,
}

pub struct Composition {
    composition_id: CompositionId,
    blocks: Vec<Block>,
    provider_selections: Vec<ContractProviderSelection>,
    exports: Vec<CompositionExportDeclaration>,
}

impl std::fmt::Debug for Composition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Composition")
            .field("id", &self.composition_id)
            .finish()
    }
}

impl CompositionBuilder {
    pub fn new(composition_id: CompositionId) -> Self {
        Self {
            composition_id,
            blocks: Vec::new(),
            provider_selections: Vec::new(),
            exports: Vec::new(),
        }
    }

    pub fn register_block(mut self, block: Block) -> Self {
        self.blocks.push(block);
        self
    }

    pub fn select_provider(mut self, selection: ContractProviderSelection) -> Self {
        self.provider_selections.push(selection);
        self
    }

    /// Deliberately exposes one typed runtime capability to an external
    /// operator. It does not make arbitrary module contracts discoverable.
    pub fn export<T>(mut self, export: CompositionExport<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.exports.push(export.declaration());
        self
    }

    pub fn build(self) -> Result<Composition, CompositionError> {
        ensure_unique_block_ids(&self.blocks)?;
        validate_declarations(&self.blocks, &self.provider_selections)?;
        ensure_unique_exports(&self.exports)?;
        Ok(Composition {
            composition_id: self.composition_id,
            blocks: self.blocks,
            provider_selections: self.provider_selections,
            exports: self.exports,
        })
    }
}

impl Composition {
    pub fn id(&self) -> &CompositionId {
        &self.composition_id
    }

    pub fn exports(&self) -> &[CompositionExportDeclaration] {
        &self.exports
    }

    pub fn materialize(&self, instance_id: InstanceId) -> Result<Instance, CompositionError> {
        let requirements = declared_host_materialization_requirements(&self.blocks);
        if !requirements.is_empty() {
            return Err(CompositionError::HostDescriptorRequired {
                module_ids: requirements
                    .into_iter()
                    .map(|requirement| requirement.module_id().clone())
                    .collect(),
            });
        }
        self.materialize_with_host(instance_id, None)
    }

    pub fn materialize_on(
        &self,
        instance_id: InstanceId,
        host: &HostDescriptor,
    ) -> Result<Instance, CompositionError> {
        self.materialize_with_host(instance_id, Some(host))
    }

    fn materialize_with_host(
        &self,
        instance_id: InstanceId,
        host: Option<&HostDescriptor>,
    ) -> Result<Instance, CompositionError> {
        if let Some(host) = host {
            validate_host_materialization_requirements(&self.blocks, host)?;
        }
        let declarations = validate_declarations(&self.blocks, &self.provider_selections)?;
        validate_export_declarations(&self.exports, &declarations.declarations)?;
        let context = InstanceRuntimeContext::new(instance_id, crate::InstanceGeneration::mint());
        let (blocks, start_order, exports) = materialize_runtime_blocks(
            &self.composition_id,
            &self.blocks,
            declarations,
            &self.exports,
            &context,
        )?;
        Ok(Instance::materialize(
            self.composition_id.clone(),
            context,
            blocks,
            start_order,
            exports,
        ))
    }
}

fn ensure_unique_block_ids(blocks: &[Block]) -> Result<(), CompositionError> {
    let mut ids = BTreeSet::new();
    for block in blocks {
        if !ids.insert(block.block_id.clone()) {
            return Err(CompositionError::DuplicateBlockId {
                block_id: block.block_id.clone(),
            });
        }
    }
    Ok(())
}

fn ensure_unique_exports(exports: &[CompositionExportDeclaration]) -> Result<(), CompositionError> {
    let mut ids = BTreeSet::new();
    for export in exports {
        if !ids.insert(export.id().clone()) {
            return Err(CompositionError::DuplicateCompositionExport {
                export_id: export.id().clone(),
            });
        }
    }
    Ok(())
}

fn validate_export_declarations(
    exports: &[CompositionExportDeclaration],
    declarations: &[ModuleDeclaration],
) -> Result<(), CompositionError> {
    for export in exports {
        let providers = declarations
            .iter()
            .flat_map(|module| {
                module
                    .provided_contracts()
                    .iter()
                    .map(move |provided| (module.module_id(), provided))
            })
            .filter(|(_, provided)| provided.id() == export.requirement().id())
            .collect::<Vec<_>>();
        if providers.is_empty() {
            return Err(CompositionError::MissingExportProvider {
                export_id: export.id().clone(),
                contract_id: export.requirement().id().clone(),
            });
        }
        let compatible = providers
            .iter()
            .filter(|(_, provided)| {
                export
                    .requirement()
                    .compatibility()
                    .accepts(provided.identity())
            })
            .collect::<Vec<_>>();
        match compatible.len() {
            0 => {
                return Err(CompositionError::IncompatibleExportProvider {
                    export_id: export.id().clone(),
                    contract_id: export.requirement().id().clone(),
                });
            }
            1 => {}
            _ => {
                return Err(CompositionError::AmbiguousExportProvider {
                    export_id: export.id().clone(),
                    contract_id: export.requirement().id().clone(),
                    providers: compatible
                        .into_iter()
                        .map(|(module_id, _)| (*module_id).clone())
                        .collect(),
                });
            }
        }
    }
    Ok(())
}

fn declared_host_materialization_requirements(
    blocks: &[Block],
) -> Vec<HostMaterializationRequirement> {
    let mut requirements = declarations(blocks)
        .into_iter()
        .filter_map(|declaration| declaration.host_requirement().cloned())
        .collect::<Vec<_>>();
    requirements.sort_by(|left, right| left.module_id().cmp(right.module_id()));
    requirements
}

fn validate_host_materialization_requirements(
    blocks: &[Block],
    host: &HostDescriptor,
) -> Result<(), CompositionError> {
    for requirement in declared_host_materialization_requirements(blocks) {
        requirement.requirement().evaluate(host).map_err(|source| {
            CompositionError::HostIncompatible {
                module_id: requirement.module_id().clone(),
                source,
            }
        })?;
    }
    Ok(())
}

fn declarations(blocks: &[Block]) -> Vec<ModuleDeclaration> {
    blocks
        .iter()
        .flat_map(|block| block.modules.iter().map(|module| module.declaration()))
        .collect()
}

fn validate_declarations(
    blocks: &[Block],
    provider_selections: &[ContractProviderSelection],
) -> Result<ValidatedDeclarations, CompositionError> {
    let declarations = declarations(blocks);
    ensure_unique_module_ids(&declarations)?;
    ensure_unique_requirement_contract_ids(&declarations)?;
    ensure_unique_provided_contract_declarations(&declarations)?;
    let provider_selections = validate_selected_providers(&declarations, provider_selections)?;
    let bindings = declaration_bindings(&declarations, &provider_selections)?;
    let start_order = dependency_order(&declarations, &bindings)?;
    Ok(ValidatedDeclarations {
        declarations,
        bindings,
        start_order,
    })
}

fn materialize_runtime_blocks(
    composition_id: &CompositionId,
    blocks: &[Block],
    validated: ValidatedDeclarations,
    external_export_declarations: &[CompositionExportDeclaration],
    context: &InstanceRuntimeContext,
) -> Result<MaterializedRuntimeBlocks, CompositionError> {
    let counts = blocks
        .iter()
        .map(|block| block.modules.len())
        .collect::<Vec<_>>();
    let mut modules = blocks
        .iter()
        .flat_map(|block| block.modules.iter())
        .map(|module| {
            module
                .materialize()
                .ok_or_else(|| CompositionError::MissingRuntimeMaterializer {
                    module_id: module.declaration().module_id().clone(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    ensure_runtime_module_ids(&validated.declarations, &modules)?;
    bind_instance_context(composition_id, &mut modules, context)?;
    let exported = collect_exports(composition_id, &modules)?;
    ensure_runtime_exports(&validated.declarations, &exported)?;
    let external_exports = resolve_runtime_exports(external_export_declarations, &exported)?;
    let bindings = runtime_bindings(&validated.bindings, &exported)?;
    bind_runtime_modules(composition_id, &validated, &mut modules, &bindings)?;

    let start_order = module_start_locations(&counts, &validated.start_order);
    let runtime_blocks = into_runtime_blocks(blocks, counts, modules);
    Ok((runtime_blocks, start_order, external_exports))
}

fn resolve_runtime_exports(
    declarations: &[CompositionExportDeclaration],
    exported: &BTreeMap<ModuleId, Vec<ModuleContract>>,
) -> Result<BTreeMap<ContractId, ModuleContract>, CompositionError> {
    let mut result = BTreeMap::new();
    for export in declarations {
        let matches = exported
            .iter()
            .flat_map(|(module_id, contracts)| {
                contracts.iter().map(move |contract| (module_id, contract))
            })
            .filter(|(_, contract)| contract.declaration().id() == export.requirement().id())
            .filter(|(_, contract)| {
                export
                    .requirement()
                    .compatibility()
                    .accepts(contract.declaration().identity())
            })
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => {
                return Err(CompositionError::MissingExportProvider {
                    export_id: export.id().clone(),
                    contract_id: export.requirement().id().clone(),
                });
            }
            [(_, contract)] => {
                result.insert(export.id().clone(), (*contract).clone());
            }
            _ => {
                return Err(CompositionError::AmbiguousExportProvider {
                    export_id: export.id().clone(),
                    contract_id: export.requirement().id().clone(),
                    providers: matches
                        .into_iter()
                        .map(|(module_id, _)| (*module_id).clone())
                        .collect(),
                });
            }
        }
    }
    Ok(result)
}

fn ensure_runtime_module_ids(
    declarations: &[ModuleDeclaration],
    modules: &[Box<dyn ModuleRuntime>],
) -> Result<(), CompositionError> {
    for (declaration, module) in declarations.iter().zip(modules) {
        if declaration.module_id() != module.id() {
            return Err(CompositionError::RuntimeModuleIdMismatch {
                declared_module_id: declaration.module_id().clone(),
                runtime_module_id: module.id().clone(),
            });
        }
    }
    Ok(())
}

fn bind_instance_context(
    composition_id: &CompositionId,
    modules: &mut [Box<dyn ModuleRuntime>],
    context: &InstanceRuntimeContext,
) -> Result<(), CompositionError> {
    for module in modules {
        module.bind_instance_context(context).map_err(|source| {
            CompositionError::ModuleFailure {
                composition_id: composition_id.clone(),
                module_id: module.id().clone(),
                phase: "instance_context",
                source,
            }
        })?;
    }
    Ok(())
}

fn bind_runtime_modules(
    composition_id: &CompositionId,
    validated: &ValidatedDeclarations,
    modules: &mut [Box<dyn ModuleRuntime>],
    bindings: &bindings::ResolvedBindings,
) -> Result<(), CompositionError> {
    // Binding follows the already-resolved provider-before-consumer dependency
    // order, never authoring or Block order. Stored module order is untouched.
    for module_index in validated.start_order.iter().copied() {
        let declaration = &validated.declarations[module_index];
        let scoped = scoped_bindings(declaration, bindings);
        modules[module_index]
            .bind(&scoped)
            .map_err(|source| CompositionError::ModuleFailure {
                composition_id: composition_id.clone(),
                module_id: declaration.module_id().clone(),
                phase: "bind",
                source,
            })?;
    }
    Ok(())
}

fn module_start_locations(counts: &[usize], order: &[usize]) -> Vec<RuntimeBlockLocation> {
    order
        .iter()
        .map(|index| {
            let mut offset = 0;
            for (block, count) in counts.iter().copied().enumerate() {
                if *index < offset + count {
                    return (block, *index - offset);
                }
                offset += count;
            }
            unreachable!("validated module index")
        })
        .collect()
}

fn into_runtime_blocks(
    blocks: &[Block],
    counts: Vec<usize>,
    mut modules: Vec<Box<dyn ModuleRuntime>>,
) -> Vec<RuntimeBlock> {
    blocks
        .iter()
        .zip(counts)
        .map(|(block, count)| RuntimeBlock {
            block_id: block.block_id.clone(),
            modules: modules.drain(..count).collect(),
            lifecycle: crate::LifecycleState::Ready,
        })
        .collect()
}

fn ensure_unique_module_ids(declarations: &[ModuleDeclaration]) -> Result<(), CompositionError> {
    let mut seen = BTreeSet::new();
    for declaration in declarations {
        let module_id = declaration.module_id().clone();
        if !seen.insert(module_id.clone()) {
            return Err(CompositionError::DuplicateModuleId { module_id });
        }
    }
    Ok(())
}

fn ensure_unique_requirement_contract_ids(
    declarations: &[ModuleDeclaration],
) -> Result<(), CompositionError> {
    for declaration in declarations {
        let mut seen = BTreeSet::new();
        for requirement in declaration
            .required_contracts()
            .iter()
            .chain(declaration.optional_contracts())
        {
            if !seen.insert(requirement.id().clone()) {
                return Err(CompositionError::DuplicateContractRequirement {
                    module_id: declaration.module_id().clone(),
                    contract_id: requirement.id().clone(),
                });
            }
        }
    }
    Ok(())
}

fn ensure_unique_provided_contract_declarations(
    declarations: &[ModuleDeclaration],
) -> Result<(), CompositionError> {
    for declaration in declarations {
        let mut seen = BTreeSet::new();
        for contract in declaration.provided_contracts() {
            if !seen.insert(contract.clone()) {
                return Err(CompositionError::DuplicateProvidedContractDeclaration {
                    module_id: declaration.module_id().clone(),
                    contract_id: contract.id().clone(),
                    identity: contract.identity().clone(),
                });
            }
        }
    }
    Ok(())
}

fn declaration_bindings(
    declarations: &[ModuleDeclaration],
    provider_selections: &ProviderSelections,
) -> Result<DeclaredBindings, CompositionError> {
    let mut candidates =
        BTreeMap::<ContractId, Vec<(ModuleId, ProvidedContractDeclaration)>>::new();
    for declaration in declarations {
        for contract in declaration.provided_contracts() {
            candidates
                .entry(contract.id().clone())
                .or_default()
                .push((declaration.module_id().clone(), contract.clone()));
        }
    }

    let mut bindings = BTreeMap::new();
    for declaration in declarations {
        let module_bindings = bindings
            .entry(declaration.module_id().clone())
            .or_insert_with(BTreeMap::new);
        for requirement in declaration.required_contracts() {
            let binding = resolve_declaration_binding(
                declaration.module_id(),
                requirement,
                false,
                &candidates,
                provider_selections,
            )?
            .expect("required contract binding");
            module_bindings.insert(requirement.id().clone(), binding);
        }
        for requirement in declaration.optional_contracts() {
            if let Some(binding) = resolve_declaration_binding(
                declaration.module_id(),
                requirement,
                true,
                &candidates,
                provider_selections,
            )? {
                module_bindings.insert(requirement.id().clone(), binding);
            }
        }
    }
    Ok(bindings)
}

fn resolve_declaration_binding(
    consumer: &ModuleId,
    requirement: &ContractRequirementDeclaration,
    optional: bool,
    candidates: &BTreeMap<ContractId, Vec<(ModuleId, ProvidedContractDeclaration)>>,
    provider_selections: &ProviderSelections,
) -> Result<Option<DeclaredBinding>, CompositionError> {
    let selected_provider = provider_selections
        .get(&(consumer.clone(), requirement.id().clone()))
        .cloned();
    let providers = candidates
        .get(requirement.id())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|(provider, _)| {
            selected_provider
                .as_ref()
                .is_none_or(|selected| provider == selected)
        })
        .collect::<Vec<_>>();
    if providers.is_empty() {
        return if let Some(provider) = selected_provider {
            Err(CompositionError::SelectedProviderMissingContract {
                consumer: consumer.clone(),
                contract_id: requirement.id().clone(),
                provider,
            })
        } else if optional {
            Ok(None)
        } else {
            Err(CompositionError::MissingProvider {
                module_id: consumer.clone(),
                contract_id: requirement.id().clone(),
            })
        };
    }
    let compatible = providers
        .iter()
        .filter(|(_, declaration)| requirement.compatibility().accepts(declaration.identity()))
        .cloned()
        .collect::<Vec<_>>();
    match compatible.as_slice() {
        [] if selected_provider.is_some() => Err(CompositionError::SelectedProviderIncompatible {
            consumer: consumer.clone(),
            contract_id: requirement.id().clone(),
            provider: selected_provider.expect("checked"),
            required_compatibility: requirement.compatibility().clone(),
            available_identities: providers
                .into_iter()
                .map(|(_, declaration)| declaration.identity().clone())
                .collect(),
        }),
        [] if optional => Ok(None),
        [] => Err(CompositionError::IncompatibleProvider {
            module_id: consumer.clone(),
            contract_id: requirement.id().clone(),
            required_compatibility: requirement.compatibility().clone(),
            available_identities: providers
                .into_iter()
                .map(|(_, declaration)| declaration.identity().clone())
                .collect(),
        }),
        [(provider, declaration)] => Ok(Some(DeclaredBinding {
            provider: provider.clone(),
            declaration: declaration.clone(),
        })),
        _ => Err(CompositionError::AmbiguousProvider {
            module_id: consumer.clone(),
            contract_id: requirement.id().clone(),
            providers: compatible
                .into_iter()
                .map(|(provider, _)| provider)
                .collect(),
        }),
    }
}

fn validate_selected_providers(
    declarations: &[ModuleDeclaration],
    selections: &[ContractProviderSelection],
) -> Result<ProviderSelections, CompositionError> {
    let mut requirements = BTreeMap::new();
    let mut provided = BTreeMap::new();
    for declaration in declarations {
        requirements.insert(
            declaration.module_id().clone(),
            declaration
                .required_contracts()
                .iter()
                .chain(declaration.optional_contracts())
                .map(|requirement| requirement.id().clone())
                .collect::<BTreeSet<_>>(),
        );
        provided.insert(
            declaration.module_id().clone(),
            declaration
                .provided_contracts()
                .iter()
                .map(|contract| contract.id().clone())
                .collect::<BTreeSet<_>>(),
        );
    }
    let mut validated = BTreeMap::new();
    for selection in selections {
        if !requirements.contains_key(selection.consumer()) {
            return Err(CompositionError::UnknownSelectionConsumer {
                consumer: selection.consumer().clone(),
                contract_id: selection.contract_id().clone(),
                provider: selection.provider().clone(),
            });
        }
        if !provided.contains_key(selection.provider()) {
            return Err(CompositionError::UnknownSelectionProvider {
                consumer: selection.consumer().clone(),
                contract_id: selection.contract_id().clone(),
                provider: selection.provider().clone(),
            });
        }
        if !requirements
            .get(selection.consumer())
            .expect("consumer checked")
            .contains(selection.contract_id())
        {
            return Err(CompositionError::SelectedUndeclaredRequirement {
                consumer: selection.consumer().clone(),
                contract_id: selection.contract_id().clone(),
                provider: selection.provider().clone(),
            });
        }
        if !provided
            .get(selection.provider())
            .expect("provider checked")
            .contains(selection.contract_id())
        {
            return Err(CompositionError::SelectedProviderMissingContract {
                consumer: selection.consumer().clone(),
                contract_id: selection.contract_id().clone(),
                provider: selection.provider().clone(),
            });
        }
        let key = (
            selection.consumer().clone(),
            selection.contract_id().clone(),
        );
        if let Some(first_provider) = validated.insert(key.clone(), selection.provider().clone()) {
            return Err(CompositionError::DuplicateContractProviderSelection {
                consumer: key.0,
                contract_id: key.1,
                first_provider,
                second_provider: selection.provider().clone(),
            });
        }
    }
    Ok(validated)
}

fn collect_exports(
    composition_id: &CompositionId,
    modules: &[Box<dyn ModuleRuntime>],
) -> Result<BTreeMap<ModuleId, Vec<ModuleContract>>, CompositionError> {
    let mut exported = BTreeMap::new();
    for module in modules {
        let contracts =
            module
                .export_contracts()
                .map_err(|source| CompositionError::ModuleFailure {
                    composition_id: composition_id.clone(),
                    module_id: module.id().clone(),
                    phase: "export_contracts",
                    source,
                })?;
        exported.insert(module.id().clone(), contracts);
    }
    Ok(exported)
}

fn ensure_runtime_exports(
    declarations: &[ModuleDeclaration],
    exported: &BTreeMap<ModuleId, Vec<ModuleContract>>,
) -> Result<(), CompositionError> {
    for declaration in declarations {
        let actual = exported
            .get(declaration.module_id())
            .expect("runtime id checked");
        ensure_unique_exported_contract_declarations(declaration.module_id(), actual)?;
        let declared = declaration
            .provided_contracts()
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let actual = actual
            .iter()
            .map(ModuleContract::declaration)
            .collect::<BTreeSet<_>>();
        if let Some(contract) = actual.difference(&declared).next() {
            return Err(CompositionError::UndeclaredProvidedContract {
                module_id: declaration.module_id().clone(),
                contract_id: contract.id().clone(),
            });
        }
        if let Some(contract) = declared.difference(&actual).next() {
            return Err(CompositionError::MissingDeclaredExport {
                module_id: declaration.module_id().clone(),
                contract_id: contract.id().clone(),
            });
        }
    }
    Ok(())
}

fn ensure_unique_exported_contract_declarations(
    module_id: &ModuleId,
    exported: &[ModuleContract],
) -> Result<(), CompositionError> {
    let mut seen = BTreeSet::new();
    for contract in exported {
        let declaration = contract.declaration();
        if !seen.insert(declaration.clone()) {
            return Err(CompositionError::DuplicateProvidedContractDeclaration {
                module_id: module_id.clone(),
                contract_id: declaration.id().clone(),
                identity: declaration.identity().clone(),
            });
        }
    }
    Ok(())
}

fn runtime_bindings(
    declarations: &DeclaredBindings,
    exported: &BTreeMap<ModuleId, Vec<ModuleContract>>,
) -> Result<bindings::ResolvedBindings, CompositionError> {
    let mut resolved = BTreeMap::new();
    for (consumer, contracts) in declarations {
        let consumer_bindings = resolved
            .entry(consumer.clone())
            .or_insert_with(BTreeMap::new);
        for (contract_id, binding) in contracts {
            let provided = find_exact_export(&binding.provider, &binding.declaration, exported)?;
            consumer_bindings.insert(
                contract_id.clone(),
                bindings::BoundContract {
                    provider: binding.provider.clone(),
                    identity: provided.identity.clone(),
                    type_id: provided.type_id,
                    value: provided.value.clone(),
                },
            );
        }
    }
    Ok(resolved)
}

fn find_exact_export<'a>(
    provider: &ModuleId,
    declaration: &ProvidedContractDeclaration,
    exported: &'a BTreeMap<ModuleId, Vec<ModuleContract>>,
) -> Result<&'a ModuleContract, CompositionError> {
    let exports = exported.get(provider).expect("provider export present");
    let mut matches = exports
        .iter()
        .filter(|contract| contract.declaration() == *declaration);
    let Some(selected) = matches.next() else {
        return Err(CompositionError::MissingDeclaredExport {
            module_id: provider.clone(),
            contract_id: declaration.id().clone(),
        });
    };
    if matches.next().is_some() {
        return Err(CompositionError::DuplicateProvidedContractDeclaration {
            module_id: provider.clone(),
            contract_id: declaration.id().clone(),
            identity: declaration.identity().clone(),
        });
    }
    Ok(selected)
}

fn scoped_bindings(
    declaration: &ModuleDeclaration,
    bindings: &bindings::ResolvedBindings,
) -> ModuleBindings {
    let declarations = declaration
        .required_contracts()
        .iter()
        .chain(declaration.optional_contracts())
        .map(|requirement| (requirement.id().clone(), requirement.clone()))
        .collect();
    let contracts = bindings
        .get(declaration.module_id())
        .cloned()
        .unwrap_or_default();
    ModuleBindings::new(declaration.module_id().clone(), declarations, contracts)
}
