use std::fmt;
use std::marker::PhantomData;

use fabric_core::{
    ContractIdentity, ContractKey, ContractProviderSelection, ContractRequirement,
    ContractVersionRequirement, Module, ModuleDeclaration, ModuleId, ModuleRuntime,
};
use fabric_resource::{ResourceId, ResourceName};

use crate::authoring::requirement_for_key;

use super::{PrimaryResourceContract, Requires, ResourceSelection};

/// An externally owned semantic contribution that may be attached to one
/// selected Resource occurrence.
///
/// This trait owns only `X`'s semantic contract. Runtime support is supplied
/// separately through [`ResourceAugmentationSupportDefinition`].
pub trait ResourceAugmentationDefinition<R>: Sized + Send + Sync + 'static
where
    R: PrimaryResourceContract,
{
    type Config: Clone + Send + Sync + 'static;
    type Contract: Clone + Send + Sync + 'static;

    /// Identifies the additional semantic contract owned by this contribution.
    fn contract_key() -> ContractKey<Self::Contract>;
}

/// An independently authored runtime implementation for a Resource
/// augmentation semantic.
///
/// Implementors may depend on both `R` and `X`, but neither the base Resource
/// nor `X` needs to know that this support implementation exists.
pub trait ResourceAugmentationSupportDefinition<R, X>: Clone + Send + Sync + 'static
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
    /// Declares implementation-owned dependencies for this support provider.
    ///
    /// Fabric adds the targeted base requirement and `X` contract provision to
    /// the returned declaration.
    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration;

    /// Materializes implementation-owned runtime support for the attachment.
    fn materialize(
        &self,
        attachment: &ResourceAugmentation<R, X>,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>>;
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
        requirement_for_key(&X::contract_key())
    }

    /// Returns the exact primary-contract requirement for this attachment's
    /// Resource definition.
    pub fn base_requirement(&self) -> Requires<R> {
        base_requirement()
    }

    /// Binds this semantic attachment to an independently authored support
    /// implementation.
    pub fn using<S>(self, support: S) -> ResourceAugmentationRealization<R, X, S>
    where
        S: ResourceAugmentationSupportDefinition<R, X>,
    {
        ResourceAugmentationRealization::new(self, support)
    }
}

impl<R, X> Module for ResourceAugmentation<R, X>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
{
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::new(self.module_id.clone())
            .with_required_contracts(vec![self.base_requirement().declaration().clone()])
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(ResourceAugmentationAttachmentRuntime::<R> {
            module_id: self.module_id.clone(),
            base: self.base_requirement(),
            marker: PhantomData,
        }))
    }
}

struct ResourceAugmentationAttachmentRuntime<R>
where
    R: PrimaryResourceContract,
{
    module_id: ModuleId,
    base: Requires<R>,
    marker: PhantomData<R>,
}

impl<R> ModuleRuntime for ResourceAugmentationAttachmentRuntime<R>
where
    R: PrimaryResourceContract,
{
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn required_contract_declarations(&self) -> Vec<fabric_core::ContractRequirementDeclaration> {
        vec![self.base.declaration().clone()]
    }

    fn export_contracts(
        &self,
    ) -> Result<Vec<fabric_core::ModuleContract>, fabric_core::ModuleError> {
        Ok(Vec::new())
    }

    fn bind(
        &mut self,
        bindings: &fabric_core::ModuleBindings,
    ) -> Result<(), fabric_core::ModuleError> {
        self.base
            .resolve(bindings)
            .map(|_| ())
            .map_err(|error| fabric_core::ModuleError::new(error.to_string()))
    }

    fn initialize(&mut self) -> Result<(), fabric_core::ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), fabric_core::ModuleError> {
        Ok(())
    }

    fn stop(&mut self) {}

    fn health(&self) -> fabric_core::Health {
        fabric_core::Health::Healthy
    }
}

/// One externally supported realization of a Resource augmentation attachment.
pub struct ResourceAugmentationRealization<R, X, S>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
    S: ResourceAugmentationSupportDefinition<R, X>,
{
    attachment: ResourceAugmentation<R, X>,
    support: S,
    provider_module_id: ModuleId,
}

impl<R, X, S> Clone for ResourceAugmentationRealization<R, X, S>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
    S: ResourceAugmentationSupportDefinition<R, X>,
{
    fn clone(&self) -> Self {
        Self {
            attachment: self.attachment.clone(),
            support: self.support.clone(),
            provider_module_id: self.provider_module_id.clone(),
        }
    }
}

impl<R, X, S> ResourceAugmentationRealization<R, X, S>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
    S: ResourceAugmentationSupportDefinition<R, X>,
{
    fn new(attachment: ResourceAugmentation<R, X>, support: S) -> Self {
        let provider_module_id = derive_support_module_id(attachment.module_id())
            .expect("static augmentation support suffix must preserve module id validity");
        Self {
            attachment,
            support,
            provider_module_id,
        }
    }

    pub fn attachment(&self) -> &ResourceAugmentation<R, X> {
        &self.attachment
    }

    pub fn support(&self) -> &S {
        &self.support
    }

    pub fn provider_module_id(&self) -> &ModuleId {
        &self.provider_module_id
    }

    /// Creates a paired base-plus-augmentation requirement for this exact
    /// supported attachment. Passing another occurrence is rejected before
    /// Core composition validation.
    pub fn require_from(
        &self,
        target: &ResourceSelection<R>,
    ) -> Result<ResourceAugmentationRequirement<R, X>, ResourceAugmentationError> {
        if target.module_id() != self.attachment.target_module_id() {
            return Err(ResourceAugmentationError::TargetMismatch {
                expected_resource_id: self.attachment.resource_id().clone(),
                expected_resource_name: self.attachment.resource_name().clone(),
                actual_resource_id: R::resource_id(),
                actual_resource_name: target.name().clone(),
            });
        }
        Ok(ResourceAugmentationRequirement {
            base: base_requirement(),
            augmentation: self.attachment.requirement(),
            target_module_id: self.attachment.target_module_id().clone(),
            augmentation_module_id: self.provider_module_id.clone(),
            marker: PhantomData,
        })
    }
}

impl<R, X, S> Module for ResourceAugmentationRealization<R, X, S>
where
    R: PrimaryResourceContract,
    X: ResourceAugmentationDefinition<R>,
    S: ResourceAugmentationSupportDefinition<R, X>,
{
    fn declaration(&self) -> ModuleDeclaration {
        self.support
            .declaration(self.provider_module_id.clone())
            .with_required_contracts(vec![
                self.attachment.base_requirement().declaration().clone(),
            ])
            .with_provided_contracts(vec![X::contract_key().declaration()])
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        self.support
            .materialize(&self.attachment, self.provider_module_id.clone())
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

fn base_requirement<R>() -> Requires<R>
where
    R: PrimaryResourceContract,
{
    let key = R::primary_contract_key();
    match key.identity() {
        ContractIdentity::Provisional => Requires::provisional(),
        ContractIdentity::Versioned(version) => Requires::versioned(
            ContractVersionRequirement::parse(format!("={version}"))
                .expect("a ContractVersion always forms an exact requirement"),
        ),
    }
}

fn derive_support_module_id(
    attachment_module_id: &ModuleId,
) -> Result<ModuleId, ResourceAugmentationError> {
    ModuleId::new(format!("{}.support", attachment_module_id.as_str()))
        .map_err(|_| ResourceAugmentationError::InvalidModuleId)
}

fn nibble(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => unreachable!("nibble accepts 0..=15"),
    }
}
