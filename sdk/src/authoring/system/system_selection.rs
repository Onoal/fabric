use std::marker::PhantomData;

use fabric_core::{ContractProviderSelection, Module, ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric_system::{SystemCompatibilityError, SystemId};

use super::{AdaptableSystemDefinition, SystemDefinition, SystemRealization};
use crate::authoring::definitions::{
    AdapterBridgeMode, AdapterDefinition, AdapterProviderModule, SystemAdapterCompatibility,
};

pub struct SystemSelection<S>
where
    S: SystemDefinition,
{
    module_id: ModuleId,
    config: S::Config,
    marker: PhantomData<S>,
}

impl<S> Clone for SystemSelection<S>
where
    S: SystemDefinition,
{
    fn clone(&self) -> Self {
        Self {
            module_id: self.module_id.clone(),
            config: self.config.clone(),
            marker: PhantomData,
        }
    }
}

impl<S> SystemSelection<S>
where
    S: SystemDefinition,
{
    pub fn new(config: S::Config) -> Result<Self, SystemCompatibilityError> {
        let system_id = S::system_id();
        S::schema().ensure_system(&system_id)?;
        Ok(Self {
            module_id: derive_module_id(&system_id)?,
            config,
            marker: PhantomData,
        })
    }

    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn config(&self) -> &S::Config {
        &self.config
    }
}

impl<S> SystemSelection<S>
where
    S: AdaptableSystemDefinition,
{
    pub fn using<A>(self, adapter: A) -> Result<SystemRealization<S, A>, SystemCompatibilityError>
    where
        A: AdapterDefinition<Target = S>,
        A::Compatibility: SystemAdapterCompatibility<S>,
    {
        let system_id = S::system_id();
        let schema = S::schema();
        schema.ensure_system(&system_id)?;
        adapter.compatibility().accepts_system_schema(&schema)?;
        let bridge_mode = adapter.bridge_mode();
        if bridge_mode == AdapterBridgeMode::SemanticApi && !S::supports_semantic_api_adapter() {
            return Err(
                SystemCompatibilityError::CanonicalAdapterRequiresApiRealization {
                    system: S::system_id(),
                },
            );
        }
        let provider_module_id = if bridge_mode == AdapterBridgeMode::SemanticApi {
            self.module_id.clone()
        } else {
            derive_realization_provider_module_id(&self.module_id)?
        };
        let selection = (bridge_mode == AdapterBridgeMode::LegacyRealization).then(|| {
            ContractProviderSelection::new(
                self.module_id.clone(),
                S::realization_requirement().id().clone(),
                provider_module_id.clone(),
            )
        });
        let semantic_requirements = if bridge_mode == AdapterBridgeMode::SemanticApi {
            S::declaration(&self).required_contracts().to_vec()
        } else {
            Vec::new()
        };
        Ok(SystemRealization::new(
            self,
            AdapterProviderModule::new(provider_module_id, adapter)
                .with_semantic_requirements(semantic_requirements),
            selection,
            bridge_mode,
        ))
    }
}

impl<S> Module for SystemSelection<S>
where
    S: SystemDefinition,
{
    fn declaration(&self) -> ModuleDeclaration {
        S::declaration(self)
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        S::materialize(self)
    }
}

fn derive_module_id(system_id: &SystemId) -> Result<ModuleId, SystemCompatibilityError> {
    ModuleId::new(format!("fabric.system.{}", encode_system_id(system_id))).map_err(|_error| {
        SystemCompatibilityError::InvalidInput {
            message: "failed to derive system module id".to_owned(),
        }
    })
}

fn encode_system_id(system_id: &SystemId) -> String {
    system_id
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
    system_module_id: &ModuleId,
) -> Result<ModuleId, SystemCompatibilityError> {
    ModuleId::new(format!("{}.realization", system_module_id.as_str())).map_err(|_error| {
        SystemCompatibilityError::InvalidInput {
            message: "failed to derive system realization provider module id".to_owned(),
        }
    })
}
