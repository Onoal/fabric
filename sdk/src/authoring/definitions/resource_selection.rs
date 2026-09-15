use std::marker::PhantomData;

use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric_resource::{
    AdapterResourceSchemaSupport, ResourceCompatibilityError, ResourceCompatibilityRole,
    ResourceError, ResourceId, ResourceName,
};

use super::{
    AdaptableResourceDefinition, AdapterDefinition, AdapterProviderModule, IntoResourceName,
    ResourceDefinition, ResourceRealization,
};

pub struct ResourceSelection<R>
where
    R: ResourceDefinition,
{
    name: ResourceName,
    module_id: ModuleId,
    config: R::Config,
    marker: PhantomData<R>,
}

impl<R> Clone for ResourceSelection<R>
where
    R: ResourceDefinition,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            module_id: self.module_id.clone(),
            config: self.config.clone(),
            marker: PhantomData,
        }
    }
}

impl<R> ResourceSelection<R>
where
    R: ResourceDefinition,
{
    pub fn new(name: impl IntoResourceName, config: R::Config) -> Result<Self, ResourceError> {
        let name = name.into_resource_name()?;
        let module_id = derive_module_id(&R::resource_id(), &name)?;
        Ok(Self {
            name,
            module_id,
            config,
            marker: PhantomData,
        })
    }

    pub fn name(&self) -> &ResourceName {
        &self.name
    }

    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn config(&self) -> &R::Config {
        &self.config
    }
}

impl<R> Module for ResourceSelection<R>
where
    R: ResourceDefinition,
{
    fn declaration(&self) -> ModuleDeclaration {
        R::declaration(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        R::materialize(self)
    }
}

impl<R> ResourceSelection<R>
where
    R: AdaptableResourceDefinition,
{
    pub fn using<A>(
        self,
        adapter: A,
    ) -> Result<ResourceRealization<R, A>, ResourceCompatibilityError>
    where
        A: AdapterDefinition<Target = R, SchemaSupport = AdapterResourceSchemaSupport>,
    {
        let schema = R::schema();
        if R::resource_id() != *schema.resource() {
            return Err(ResourceCompatibilityError::ResourceIdentityMismatch {
                expected: R::resource_id(),
                actual: schema.resource().clone(),
                role: ResourceCompatibilityRole::Schema,
            });
        }
        adapter.schema_support().accepts_schema(&schema)?;
        let provider_module_id = derive_realization_provider_module_id(&self.module_id)
            .expect("static realization suffix must preserve module id validity");
        let selection = ContractProviderSelection::new(
            self.module_id.clone(),
            R::realization_requirement().id().clone(),
            provider_module_id.clone(),
        );
        Ok(ResourceRealization::new(
            self,
            AdapterProviderModule::new(provider_module_id, adapter),
            selection,
        ))
    }
}

fn derive_module_id(
    resource_id: &ResourceId,
    resource_name: &ResourceName,
) -> Result<ModuleId, ResourceError> {
    let encoded_name = encode_resource_name(resource_name);
    ModuleId::new(format!(
        "{}.selection.{}",
        resource_id.as_str(),
        encoded_name
    ))
    .map_err(|_error| ResourceError::PrepareFailed {
        message: "failed to derive module id from resource selection".to_owned(),
    })
}

fn encode_resource_name(resource_name: &ResourceName) -> String {
    resource_name
        .as_str()
        .as_bytes()
        .iter()
        .flat_map(|byte| [encode_nibble(byte >> 4), encode_nibble(byte & 0x0f)])
        .collect()
}

fn encode_nibble(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("nibble encoding only accepts 0..=15"),
    }
}

fn derive_realization_provider_module_id(
    resource_module_id: &ModuleId,
) -> Result<ModuleId, ResourceError> {
    ModuleId::new(format!("{}.realization", resource_module_id.as_str())).map_err(|_error| {
        ResourceError::PrepareFailed {
            message: "failed to derive realization provider module id".to_owned(),
        }
    })
}
