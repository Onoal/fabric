use super::manifest::{RealizationProvenance, SystemManifestEntry, adapter_realization_provenance};
use super::sealed::Sealed;
use crate::authoring::definitions::{
    AdapterBridgeMode, AdapterDefinition, SystemAdapterCompatibility,
};
use crate::authoring::system::{
    AdaptableSystemDefinition, SystemDefinition, SystemRealization, SystemSelection,
};
use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration};

pub trait IntoFabricSystem: Sealed {
    #[doc(hidden)]
    fn into_fabric_system(self) -> FabricSystemContribution;
}

pub struct FabricSystemContribution {
    entry: SystemManifestEntry,
    modules: Vec<Box<dyn Module>>,
    declarations: Vec<ModuleDeclaration>,
    provider_selections: Vec<ContractProviderSelection>,
}

impl FabricSystemContribution {
    pub(crate) fn entry(&self) -> &SystemManifestEntry {
        &self.entry
    }

    pub(crate) fn modules(self) -> Vec<Box<dyn Module>> {
        self.modules
    }

    pub(crate) fn declarations(&self) -> &[ModuleDeclaration] {
        &self.declarations
    }

    pub(crate) fn provider_selections(&self) -> &[ContractProviderSelection] {
        &self.provider_selections
    }

    fn new(
        entry: SystemManifestEntry,
        modules: Vec<Box<dyn Module>>,
        declarations: Vec<ModuleDeclaration>,
        provider_selections: Vec<ContractProviderSelection>,
    ) -> Self {
        Self {
            entry,
            modules,
            declarations,
            provider_selections,
        }
    }
}

impl<S> Sealed for SystemSelection<S> where S: SystemDefinition {}

impl<S> IntoFabricSystem for SystemSelection<S>
where
    S: SystemDefinition,
{
    fn into_fabric_system(self) -> FabricSystemContribution {
        let realization = if S::has_self_realization() {
            RealizationProvenance::SelfRealization {
                runtime_module_id: Some(self.module_id().clone()),
            }
        } else {
            RealizationProvenance::DeclarationOnly
        };
        let entry = SystemManifestEntry::new(S::system_id(), S::schema(), realization);
        let declarations = vec![self.declaration()];
        FabricSystemContribution::new(entry, vec![Box::new(self)], declarations, Vec::new())
    }
}

impl<S, A> Sealed for SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S>,
    A::Compatibility: SystemAdapterCompatibility<S>,
{
}

impl<S, A> IntoFabricSystem for SystemRealization<S, A>
where
    S: AdaptableSystemDefinition,
    A: AdapterDefinition<Target = S>,
    A::Compatibility: SystemAdapterCompatibility<S>,
{
    fn into_fabric_system(self) -> FabricSystemContribution {
        let (system, adapter, selection, bridge_mode) = self.into_bridge_parts();
        let realization = adapter_realization_provenance(
            bridge_mode,
            adapter.adapter().adapter_definition_id(),
            adapter.provider_module_id().clone(),
            system.module_id().clone(),
        );
        let entry = SystemManifestEntry::new(S::system_id(), S::schema(), realization);
        if bridge_mode == AdapterBridgeMode::SemanticApi {
            // See the Resource equivalent: semantic Relations constrain the
            // Adapter-owned live participant without reviving a System proxy.
            let declaration = adapter.declaration();
            FabricSystemContribution::new(
                entry,
                vec![Box::new(adapter)],
                vec![declaration],
                Vec::new(),
            )
        } else {
            let declarations = vec![system.declaration(), adapter.declaration()];
            FabricSystemContribution::new(
                entry,
                vec![Box::new(system), Box::new(adapter)],
                declarations,
                vec![selection.expect("legacy adapter realization selects its provider")],
            )
        }
    }
}

#[cfg(test)]
mod realization_provenance_tests {
    use super::*;
    use crate::authoring::AdapterDefinition;
    use crate::authoring::fabric::manifest::AdapterRealizationMode;

    crate::system! {
        ProvenanceSystem {
            id: "fabric.test.provenance.system";
            api { fn read(&self) -> u64; }
        }
    }

    crate::system! {
        ProvenanceSelfSystem {
            id: "fabric.test.provenance.self-system";
            api { fn read(&self) -> u64; }
            runtime { fn read(&self) -> u64 { 1 } }
        }
    }

    crate::adapter! {
        ProvenanceSystemAdapter for ProvenanceSystem {
            id: "test.provenance.system-adapter";
            runtime { fn read(&self) -> u64 { 1 } }
        }
    }

    #[test]
    fn system_lowering_retains_declaration_self_and_direct_adapter_truth() {
        let declaration = ProvenanceSystem::select()
            .expect("selection")
            .into_fabric_system();
        assert!(matches!(
            declaration.entry().realization(),
            RealizationProvenance::DeclarationOnly
        ));

        let self_realized = ProvenanceSelfSystem::select()
            .expect("selection")
            .into_fabric_system();
        assert!(matches!(
            self_realized.entry().realization(),
            RealizationProvenance::SelfRealization {
                runtime_module_id: Some(_)
            }
        ));

        let adapter = ProvenanceSystemAdapter::new();
        let adapter_id = adapter.adapter_definition_id();
        let direct = ProvenanceSystem::select()
            .expect("selection")
            .using(adapter)
            .expect("adapter compatibility")
            .into_fabric_system();
        match direct.entry().realization() {
            RealizationProvenance::Adapter(provenance) => {
                assert_eq!(provenance.definition_id, adapter_id);
                assert_eq!(provenance.mode, AdapterRealizationMode::Direct);
                assert!(provenance.semantic_owner_module_id.is_none());
            }
            other => panic!("expected direct Adapter provenance, got {other:?}"),
        }
    }
}
