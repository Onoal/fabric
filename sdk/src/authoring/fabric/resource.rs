use super::manifest::{
    RealizationProvenance, RelationDeclarationOwner, RelationDeclarationProvenance,
    ResourceManifestEntry, adapter_realization_provenance,
};
use super::sealed::Sealed;
use crate::authoring::definitions::{
    AdaptableResourceDefinition, AdapterBridgeMode, AdapterDefinition,
    ResourceAdapterCompatibility, ResourceDefinition, ResourceRealization, ResourceSelection,
};
use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration};

pub trait IntoFabricResource: Sealed {
    #[doc(hidden)]
    fn into_fabric_resource(self) -> FabricResourceContribution;
}

pub struct FabricResourceContribution {
    entry: ResourceManifestEntry,
    modules: Vec<Box<dyn Module>>,
    declarations: Vec<ModuleDeclaration>,
    provider_selections: Vec<ContractProviderSelection>,
    relation_declarations: Vec<RelationDeclarationProvenance>,
}

impl FabricResourceContribution {
    pub(crate) fn entry(&self) -> &ResourceManifestEntry {
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

    pub(crate) fn relation_declarations(&self) -> &[RelationDeclarationProvenance] {
        &self.relation_declarations
    }

    fn new(
        entry: ResourceManifestEntry,
        modules: Vec<Box<dyn Module>>,
        declarations: Vec<ModuleDeclaration>,
        provider_selections: Vec<ContractProviderSelection>,
        relation_declarations: Vec<RelationDeclarationProvenance>,
    ) -> Self {
        Self {
            entry,
            modules,
            declarations,
            provider_selections,
            relation_declarations,
        }
    }
}

fn resource_relation_declarations<R>(
    name: &fabric_resource::ResourceName,
    consumer_module_id: fabric_core::ModuleId,
) -> Vec<RelationDeclarationProvenance>
where
    R: ResourceDefinition,
{
    R::relation_declarations(consumer_module_id.clone())
        .into_iter()
        .map(|(role, target, requirement)| {
            RelationDeclarationProvenance::new(
                RelationDeclarationOwner::Resource {
                    resource_id: R::resource_id(),
                    resource_name: name.clone(),
                },
                role,
                target,
                consumer_module_id.clone(),
                requirement,
            )
        })
        .collect()
}

impl<R> Sealed for ResourceSelection<R> where R: ResourceDefinition {}

impl<R> IntoFabricResource for ResourceSelection<R>
where
    R: ResourceDefinition,
{
    fn into_fabric_resource(self) -> FabricResourceContribution {
        let realization = if R::has_self_realization() {
            RealizationProvenance::SelfRealization {
                runtime_module_id: Some(self.module_id().clone()),
            }
        } else {
            RealizationProvenance::DeclarationOnly
        };
        let entry = ResourceManifestEntry::new(
            R::resource_id(),
            self.name().clone(),
            R::schema(),
            R::api_metadata(),
            realization,
            self.module_id().clone(),
        );
        let relation_declarations =
            resource_relation_declarations::<R>(self.name(), self.module_id().clone());
        let declarations = vec![self.declaration()];
        FabricResourceContribution::new(
            entry,
            vec![Box::new(self)],
            declarations,
            Vec::new(),
            relation_declarations,
        )
    }
}

impl<R, A> Sealed for ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R>,
    A::Compatibility: ResourceAdapterCompatibility<R>,
{
}

impl<R, A> IntoFabricResource for ResourceRealization<R, A>
where
    R: AdaptableResourceDefinition,
    A: AdapterDefinition<Target = R>,
    A::Compatibility: ResourceAdapterCompatibility<R>,
{
    fn into_fabric_resource(self) -> FabricResourceContribution {
        let (resource, adapter, selection, bridge_mode) = self.into_bridge_parts();
        let semantic_provider_module_id = resource.module_id().clone();
        let consumer_module_id = if bridge_mode == AdapterBridgeMode::SemanticApi {
            adapter.provider_module_id().clone()
        } else {
            resource.module_id().clone()
        };
        let mut relation_declarations =
            resource_relation_declarations::<R>(resource.name(), consumer_module_id);
        let adapter_definition_id = adapter.adapter().adapter_definition_id();
        relation_declarations.extend(
            adapter
                .adapter()
                .relation_declarations(adapter.provider_module_id().clone())
                .into_iter()
                .map(|(role, target, requirement)| {
                    RelationDeclarationProvenance::new(
                        RelationDeclarationOwner::AdapterRealizationUse {
                            adapter_definition_id: adapter_definition_id.clone(),
                            provider_module_id: adapter.provider_module_id().clone(),
                        },
                        role,
                        target,
                        adapter.provider_module_id().clone(),
                        requirement,
                    )
                }),
        );
        let realization = adapter_realization_provenance(
            bridge_mode,
            adapter_definition_id,
            adapter.provider_module_id().clone(),
            resource.module_id().clone(),
        );
        let entry = ResourceManifestEntry::new(
            R::resource_id(),
            resource.name().clone(),
            R::schema(),
            R::api_metadata(),
            realization,
            semantic_provider_module_id,
        );
        if bridge_mode == AdapterBridgeMode::SemanticApi {
            // The semantic subject remains declaration-only, but its declared
            // Relations still constrain the selected live realization's graph
            // position. This copies declarations only; it creates neither a
            // Resource runtime nor a method-forwarding proxy.
            let declaration = adapter.declaration();
            FabricResourceContribution::new(
                entry,
                vec![Box::new(adapter)],
                vec![declaration],
                Vec::new(),
                relation_declarations,
            )
        } else {
            let declarations = vec![resource.declaration(), adapter.declaration()];
            FabricResourceContribution::new(
                entry,
                vec![Box::new(resource), Box::new(adapter)],
                declarations,
                vec![selection.expect("legacy adapter realization selects its provider")],
                relation_declarations,
            )
        }
    }
}

#[cfg(test)]
mod realization_provenance_tests {
    use super::*;
    use crate::authoring::AdapterDefinition;
    use crate::authoring::fabric::manifest::AdapterRealizationMode;

    crate::resource! {
        ProvenanceResource {
            id: "fabric.test.provenance.resource";
            api { fn read(&self) -> u64; }
        }
    }

    crate::resource! {
        ProvenanceSelfResource {
            id: "fabric.test.provenance.self-resource";
            api { fn read(&self) -> u64; }
            runtime { fn read(&self) -> u64 { 1 } }
        }
    }

    crate::adapter! {
        ProvenanceResourceAdapter for ProvenanceResource {
            id: "test.provenance.resource-adapter";
            runtime { fn read(&self) -> u64 { 1 } }
        }
    }

    struct StaticSelfResource;

    impl crate::authoring::ResourceDefinition for StaticSelfResource {
        type Config = ();

        fn resource_id() -> crate::resource::ResourceId {
            crate::resource::ResourceId::new("fabric.test.provenance.static-self")
                .expect("static resource id")
        }

        fn schema() -> crate::resource::ResourceSchemaDescriptor {
            crate::resource::ResourceSchemaDescriptor::provisional(Self::resource_id())
        }

        fn declaration(
            selection: &crate::authoring::ResourceSelection<Self>,
        ) -> fabric_core::ModuleDeclaration {
            fabric_core::ModuleDeclaration::new(selection.module_id().clone())
        }

        fn has_self_realization() -> bool {
            true
        }

        fn materialize(
            _selection: &crate::authoring::ResourceSelection<Self>,
        ) -> Option<Box<dyn fabric_core::ModuleRuntime>> {
            panic!("realization classification must not materialize a runtime")
        }
    }

    #[test]
    fn resource_lowering_retains_declaration_self_and_direct_adapter_truth() {
        let static_self =
            <StaticSelfResource as crate::authoring::ResourceDefinition>::select("static", ())
                .expect("selection")
                .into_fabric_resource();
        assert!(matches!(
            static_self.entry().realization(),
            RealizationProvenance::SelfRealization {
                runtime_module_id: Some(_)
            }
        ));

        let declaration = ProvenanceResource::select("declaration")
            .expect("selection")
            .into_fabric_resource();
        assert!(matches!(
            declaration.entry().realization(),
            RealizationProvenance::DeclarationOnly
        ));

        let self_realized = ProvenanceSelfResource::select("self")
            .expect("selection")
            .into_fabric_resource();
        assert!(matches!(
            self_realized.entry().realization(),
            RealizationProvenance::SelfRealization {
                runtime_module_id: Some(_)
            }
        ));

        let adapter = ProvenanceResourceAdapter::new();
        let adapter_id = adapter.adapter_definition_id();
        let direct = ProvenanceResource::select("direct")
            .expect("selection")
            .using(adapter)
            .expect("adapter compatibility")
            .into_fabric_resource();
        let (primary_definition_id, primary_provider_module_id) = match direct.entry().realization()
        {
            RealizationProvenance::Adapter(provenance) => {
                assert_eq!(provenance.definition_id, adapter_id);
                assert_eq!(provenance.mode, AdapterRealizationMode::Direct);
                assert!(provenance.semantic_owner_module_id.is_none());
                (
                    provenance.definition_id.clone(),
                    provenance.provider_module_id.clone(),
                )
            }
            other => panic!("expected direct Adapter provenance, got {other:?}"),
        };

        let archive = ProvenanceResource::select("archive")
            .expect("selection")
            .using(ProvenanceResourceAdapter::new())
            .expect("adapter compatibility")
            .into_fabric_resource();
        match archive.entry().realization() {
            RealizationProvenance::Adapter(provenance) => {
                assert_eq!(provenance.definition_id, primary_definition_id);
                assert_ne!(provenance.provider_module_id, primary_provider_module_id);
            }
            other => panic!("expected archive Adapter provenance, got {other:?}"),
        }
    }
}
