use std::fmt;
use std::marker::PhantomData;

use fabric_core::{
    ContractKey, ContractProviderSelection, ContractRequirement, Module, ModuleDeclaration,
    ModuleId, ModuleRuntime,
};
use fabric_resource::{ResourceId, ResourceName};

use super::{PrimaryResourceContract, Requires, ResourceSelection};

/// An externally owned semantic contribution attached to one selected Resource occurrence.
///
/// The base Resource definition does not know this contribution exists. The
/// attachment lowers to a normal Core module that requires the target
/// occurrence's primary contract and provides the contribution's typed contract.
pub trait ResourceAugmentationDefinition<R>: Sized + Send + Sync + 'static
where
    R: PrimaryResourceContract,
{
    type Config: Clone + Send + Sync + 'static;
    type Contract: Clone + Send + Sync + 'static;

    /// Identifies the additional semantic contract owned by this contribution.
    fn contract_key() -> ContractKey<Self::Contract>;

    /// Creates any local runtime support for this attachment.
    fn materialize(attachment: &ResourceAugmentation<R, Self>) -> Option<Box<dyn ModuleRuntime>>;
}

/// One occurrence-specific attachment of externally owned Resource meaning.
pub struct ResourceAugmentation<R, X>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
    resource_id: ResourceId,
    resource_name: ResourceName,
    target_module_id: ModuleId,
    module_id: ModuleId,
    config: X::Config,
    marker: PhantomData<(R, X)>,
}

impl<R, X> Clone for ResourceAugmentation<R, X>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
    fn clone(&self) -> Self {
        Self {
            resource_id: self.resource_id.clone(),
            resource_name: self.resource_name.clone(),
            target_module_id: self.target_module_id.clone(),
            module_id: self.module_id.clone(),
            config: self.config.clone(),
            marker: PhantomData,
        }
    }
}

impl<R, X> ResourceAugmentation<R, X>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
    /// Attaches `X` to exactly `target`; it does not alter `R` or other
    /// occurrences of `R`.
    pub fn attach(
        target: &ResourceSelection<R>,
        config: X::Config,
    ) -> Result<Self, ResourceAugmentationError> {
        let module_id = derive_module_id(target.module_id(), X::contract_key().id())?;
        Ok(Self {
            resource_id: R::resource_id(),
            resource_name: target.name().clone(),
            target_module_id: target.module_id().clone(),
            module_id,
            config,
            marker: PhantomData,
        })
    }

    pub fn resource_id(&self) -> &ResourceId {
        &self.resource_id
    }

    pub fn resource_name(&self) -> &ResourceName {
        &self.resource_name
    }

    pub fn target_module_id(&self) -> &ModuleId {
        &self.target_module_id
    }

    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn config(&self) -> &X::Config {
        &self.config
    }

    pub fn requirement(&self) -> ContractRequirement<X::Contract> {
        ContractRequirement::provisional(X::contract_key().id().clone())
    }

    /// Creates a paired base-plus-augmentation requirement for this exact
    /// target occurrence. Passing another occurrence is rejected before Core
    /// composition validation.
    pub fn require_from(
        &self,
        target: &ResourceSelection<R>,
    ) -> Result<ResourceAugmentationRequirement<R, X>, ResourceAugmentationError> {
        if target.module_id() != &self.target_module_id {
            return Err(ResourceAugmentationError::TargetMismatch {
                expected_resource_id: self.resource_id.clone(),
                expected_resource_name: self.resource_name.clone(),
                actual_resource_id: R::resource_id(),
                actual_resource_name: target.name().clone(),
            });
        }
        Ok(ResourceAugmentationRequirement {
            base: Requires::provisional(),
            augmentation: self.requirement(),
            target_module_id: self.target_module_id.clone(),
            augmentation_module_id: self.module_id.clone(),
            marker: PhantomData,
        })
    }
}

impl<R, X> Module for ResourceAugmentation<R, X>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::new(self.module_id.clone())
            .with_required_contracts(vec![Requires::<R>::provisional().declaration().clone()])
            .with_provided_contracts(vec![X::contract_key().declaration()])
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        X::materialize(self)
    }
}

/// Typed requirements for one Resource occurrence plus one attached semantic.
pub struct ResourceAugmentationRequirement<R, X>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
    base: Requires<R>,
    augmentation: ContractRequirement<X::Contract>,
    target_module_id: ModuleId,
    augmentation_module_id: ModuleId,
    marker: PhantomData<X>,
}

impl<R, X> ResourceAugmentationRequirement<R, X>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
    pub fn base(&self) -> &Requires<R> {
        &self.base
    }

    pub fn augmentation(&self) -> &ContractRequirement<X::Contract> {
        &self.augmentation
    }

    pub fn provider_selections(&self, consumer: ModuleId) -> [ContractProviderSelection; 2] {
        [
            ContractProviderSelection::new(
                consumer.clone(),
                self.base.declaration().id().clone(),
                self.target_module_id.clone(),
            ),
            ContractProviderSelection::new(
                consumer,
                self.augmentation.id().clone(),
                self.augmentation_module_id.clone(),
            ),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceAugmentationError {
    InvalidModuleId,
    TargetMismatch {
        expected_resource_id: ResourceId,
        expected_resource_name: ResourceName,
        actual_resource_id: ResourceId,
        actual_resource_name: ResourceName,
    },
}

impl fmt::Display for ResourceAugmentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModuleId => formatter.write_str("invalid Resource augmentation module id"),
            Self::TargetMismatch {
                expected_resource_id,
                expected_resource_name,
                actual_resource_id,
                actual_resource_name,
            } => write!(
                formatter,
                "Resource augmentation targets `{expected_resource_id}({expected_resource_name:?})`, not `{actual_resource_id}({actual_resource_name:?})`"
            ),
        }
    }
}

impl std::error::Error for ResourceAugmentationError {}

fn derive_module_id(
    target_module_id: &ModuleId,
    contract_id: &fabric_core::ContractId,
) -> Result<ModuleId, ResourceAugmentationError> {
    let encoded_contract = contract_id
        .as_str()
        .bytes()
        .flat_map(|byte| [nibble(byte >> 4), nibble(byte & 0x0f)])
        .collect::<String>();
    ModuleId::new(format!(
        "{}.augmentation.{encoded_contract}",
        target_module_id.as_str()
    ))
    .map_err(|_| ResourceAugmentationError::InvalidModuleId)
}

fn nibble(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => unreachable!("nibble accepts 0..=15"),
    }
}
