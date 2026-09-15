use fabric_component::{ComponentId, ComponentResourceRequirementName};
use fabric_core::{
    BlockId, CompositionExportDeclaration, ContractProviderSelection, ModuleDeclaration,
};
use fabric_resource::{ResourceId, ResourceName, ResourceSchemaDescriptor};

#[derive(Clone, Debug)]
pub struct ResourceManifestEntry {
    resource_id: ResourceId,
    name: ResourceName,
    schema: ResourceSchemaDescriptor,
}

impl ResourceManifestEntry {
    pub(crate) fn new(
        resource_id: ResourceId,
        name: ResourceName,
        schema: ResourceSchemaDescriptor,
    ) -> Self {
        Self {
            resource_id,
            name,
            schema,
        }
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn name(&self) -> &ResourceName {
        &self.name
    }

    pub fn schema(&self) -> &ResourceSchemaDescriptor {
        &self.schema
    }
}

#[derive(Clone, Debug)]
pub struct SystemManifestEntry {
    system_id: fabric_system::SystemId,
    schema: fabric_system::SystemSchemaDescriptor,
}

#[derive(Clone, Debug)]
pub struct ComponentResourceBindingManifestEntry {
    component_id: ComponentId,
    requirement_name: ComponentResourceRequirementName,
    resource_id: ResourceId,
    resource_name: ResourceName,
}

impl ComponentResourceBindingManifestEntry {
    pub(crate) fn new(
        component_id: ComponentId,
        requirement_name: ComponentResourceRequirementName,
        resource_id: ResourceId,
        resource_name: ResourceName,
    ) -> Self {
        Self {
            component_id,
            requirement_name,
            resource_id,
            resource_name,
        }
    }
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    pub fn requirement_name(&self) -> &ComponentResourceRequirementName {
        &self.requirement_name
    }
    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }
    pub fn resource_name(&self) -> &ResourceName {
        &self.resource_name
    }
}

#[derive(Clone, Debug)]
pub struct ComponentSystemBindingManifestEntry {
    component_id: ComponentId,
    system_id: fabric_system::SystemId,
}
impl ComponentSystemBindingManifestEntry {
    pub(crate) fn new(component_id: ComponentId, system_id: fabric_system::SystemId) -> Self {
        Self {
            component_id,
            system_id,
        }
    }
    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }
    pub fn system_id(&self) -> &fabric_system::SystemId {
        &self.system_id
    }
}

impl SystemManifestEntry {
    pub(crate) fn new(
        system_id: fabric_system::SystemId,
        schema: fabric_system::SystemSchemaDescriptor,
    ) -> Self {
        Self { system_id, schema }
    }

    pub fn system_id(&self) -> &fabric_system::SystemId {
        &self.system_id
    }

    pub fn schema(&self) -> &fabric_system::SystemSchemaDescriptor {
        &self.schema
    }
}

pub struct FabricManifestDiagnostics<'a> {
    module_declarations: &'a [ModuleDeclaration],
    provider_selections: &'a [ContractProviderSelection],
    raw_blocks: &'a [BlockId],
    composition_exports: &'a [CompositionExportDeclaration],
}

impl<'a> FabricManifestDiagnostics<'a> {
    pub fn module_declarations(&self) -> &'a [ModuleDeclaration] {
        self.module_declarations
    }
    pub fn provider_selections(&self) -> &'a [ContractProviderSelection] {
        self.provider_selections
    }
    pub fn raw_blocks(&self) -> &'a [BlockId] {
        self.raw_blocks
    }
    pub fn composition_exports(&self) -> &'a [CompositionExportDeclaration] {
        self.composition_exports
    }
}

#[derive(Clone, Debug, Default)]
pub struct FabricManifest {
    resources: Vec<ResourceManifestEntry>,
    systems: Vec<SystemManifestEntry>,
    components: Vec<fabric_component::ComponentDeclaration>,
    module_declarations: Vec<ModuleDeclaration>,
    provider_selections: Vec<ContractProviderSelection>,
    component_resource_bindings: Vec<ComponentResourceBindingManifestEntry>,
    component_system_bindings: Vec<ComponentSystemBindingManifestEntry>,
    raw_blocks: Vec<BlockId>,
    composition_exports: Vec<CompositionExportDeclaration>,
}

impl FabricManifest {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        resources: Vec<ResourceManifestEntry>,
        systems: Vec<SystemManifestEntry>,
        components: Vec<fabric_component::ComponentDeclaration>,
        module_declarations: Vec<ModuleDeclaration>,
        provider_selections: Vec<ContractProviderSelection>,
        component_resource_bindings: Vec<ComponentResourceBindingManifestEntry>,
        component_system_bindings: Vec<ComponentSystemBindingManifestEntry>,
        raw_blocks: Vec<BlockId>,
        composition_exports: Vec<CompositionExportDeclaration>,
    ) -> Self {
        Self {
            resources,
            systems,
            components,
            module_declarations,
            provider_selections,
            component_resource_bindings,
            component_system_bindings,
            raw_blocks,
            composition_exports,
        }
    }

    pub fn resources(&self) -> &[ResourceManifestEntry] {
        &self.resources
    }

    pub fn systems(&self) -> &[SystemManifestEntry] {
        &self.systems
    }

    pub fn components(&self) -> &[fabric_component::ComponentDeclaration] {
        &self.components
    }

    pub fn component_resource_bindings(&self) -> &[ComponentResourceBindingManifestEntry] {
        &self.component_resource_bindings
    }
    pub fn component_system_bindings(&self) -> &[ComponentSystemBindingManifestEntry] {
        &self.component_system_bindings
    }

    pub fn diagnostics(&self) -> FabricManifestDiagnostics<'_> {
        FabricManifestDiagnostics {
            module_declarations: &self.module_declarations,
            provider_selections: &self.provider_selections,
            raw_blocks: &self.raw_blocks,
            composition_exports: &self.composition_exports,
        }
    }
}
