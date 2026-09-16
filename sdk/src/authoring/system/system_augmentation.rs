use std::fmt;
use std::marker::PhantomData;

use fabric_core::{
    ContractIdentity, ContractKey, ContractProviderSelection, ContractRequirement,
    ContractVersionRequirement, Module, ModuleDeclaration, ModuleId, ModuleRuntime,
};
use fabric_system::SystemId;

use super::{PrimarySystemContract, SystemRequires, SystemSelection};

/// Externally owned semantic meaning that may attach to one selected System.
///
/// This trait owns only the semantic contract for `X`. Runtime support is
/// supplied separately through [`SystemAugmentationSupportDefinition`].
pub trait SystemAugmentationDefinition<S>: Sized + Send + Sync + 'static
where
    S: PrimarySystemContract,
{
    type Config: Clone + Send + Sync + 'static;
    type Contract: Clone + Send + Sync + 'static;

    fn contract_key() -> ContractKey<Self::Contract>;
}

/// Independently authored runtime support for a System augmentation semantic.
pub trait SystemAugmentationSupportDefinition<S, X>: Clone + Send + Sync + 'static
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
{
    /// Declares implementation-owned dependencies for this support provider.
    /// Fabric adds the target System requirement and `X` contract provision.
    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration;

    fn materialize(
        &self,
        attachment: &SystemAugmentation<S, X>,
        provider_module_id: ModuleId,
    ) -> Option<Box<dyn ModuleRuntime>>;
}

/// One selected-System attachment of externally owned semantic meaning.
pub struct SystemAugmentation<S, X>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
{
    system_id: SystemId,
    target_module_id: ModuleId,
    module_id: ModuleId,
    config: X::Config,
    marker: PhantomData<(S, X)>,
}

impl<S, X> Clone for SystemAugmentation<S, X>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
{
    fn clone(&self) -> Self {
        Self {
            system_id: self.system_id.clone(),
            target_module_id: self.target_module_id.clone(),
            module_id: self.module_id.clone(),
            config: self.config.clone(),
            marker: PhantomData,
        }
    }
}

impl<S, X> SystemAugmentation<S, X>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
{
    pub fn attach(
        target: &SystemSelection<S>,
        config: X::Config,
    ) -> Result<Self, SystemAugmentationError> {
        let module_id = derive_module_id(target.module_id(), X::contract_key().id())?;
        Ok(Self {
            system_id: S::system_id(),
            target_module_id: target.module_id().clone(),
            module_id,
            config,
            marker: PhantomData,
        })
    }

    pub fn system_id(&self) -> &SystemId {
        &self.system_id
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

    pub fn base_requirement(&self) -> SystemRequires<S> {
        base_requirement()
    }

    pub fn using<P>(self, support: P) -> SystemAugmentationRealization<S, X, P>
    where
        P: SystemAugmentationSupportDefinition<S, X>,
    {
        SystemAugmentationRealization::new(self, support)
    }
}

impl<S, X> Module for SystemAugmentation<S, X>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
{
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::new(self.module_id.clone())
            .with_required_contracts(vec![self.base_requirement().declaration().clone()])
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(SystemAugmentationAttachmentRuntime::<S> {
            module_id: self.module_id.clone(),
            base: self.base_requirement(),
            marker: PhantomData,
        }))
    }
}

struct SystemAugmentationAttachmentRuntime<S>
where
    S: PrimarySystemContract,
{
    module_id: ModuleId,
    base: SystemRequires<S>,
    marker: PhantomData<S>,
}

impl<S> ModuleRuntime for SystemAugmentationAttachmentRuntime<S>
where
    S: PrimarySystemContract,
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

/// One externally supported realization of a System augmentation attachment.
pub struct SystemAugmentationRealization<S, X, P>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
    P: SystemAugmentationSupportDefinition<S, X>,
{
    attachment: SystemAugmentation<S, X>,
    support: P,
    provider_module_id: ModuleId,
}

impl<S, X, P> Clone for SystemAugmentationRealization<S, X, P>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
    P: SystemAugmentationSupportDefinition<S, X>,
{
    fn clone(&self) -> Self {
        Self {
            attachment: self.attachment.clone(),
            support: self.support.clone(),
            provider_module_id: self.provider_module_id.clone(),
        }
    }
}

impl<S, X, P> SystemAugmentationRealization<S, X, P>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
    P: SystemAugmentationSupportDefinition<S, X>,
{
    fn new(attachment: SystemAugmentation<S, X>, support: P) -> Self {
        let provider_module_id = derive_support_module_id(attachment.module_id())
            .expect("static augmentation support suffix must preserve module id validity");
        Self {
            attachment,
            support,
            provider_module_id,
        }
    }

    pub fn attachment(&self) -> &SystemAugmentation<S, X> {
        &self.attachment
    }
    pub fn support(&self) -> &P {
        &self.support
    }
    pub fn provider_module_id(&self) -> &ModuleId {
        &self.provider_module_id
    }

    pub fn require_from(
        &self,
        target: &SystemSelection<S>,
    ) -> Result<SystemAugmentationRequirement<S, X>, SystemAugmentationError> {
        if target.module_id() != self.attachment.target_module_id() {
            return Err(SystemAugmentationError::TargetMismatch {
                expected_system_id: self.attachment.system_id().clone(),
                actual_system_id: S::system_id(),
            });
        }
        Ok(SystemAugmentationRequirement {
            base: self.attachment.base_requirement(),
            augmentation: self.attachment.requirement(),
            target_module_id: self.attachment.target_module_id().clone(),
            augmentation_module_id: self.provider_module_id.clone(),
            marker: PhantomData,
        })
    }
}

impl<S, X, P> Module for SystemAugmentationRealization<S, X, P>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
    P: SystemAugmentationSupportDefinition<S, X>,
{
    fn declaration(&self) -> ModuleDeclaration {
        let support_declaration = self.support.declaration(self.provider_module_id.clone());
        let mut required_contracts = support_declaration.required_contracts().to_vec();
        required_contracts.push(self.attachment.base_requirement().declaration().clone());
        let mut provided_contracts = support_declaration.provided_contracts().to_vec();
        provided_contracts.push(X::contract_key().declaration());

        let declaration = ModuleDeclaration::new(support_declaration.module_id().clone())
            .with_required_contracts(required_contracts)
            .with_provided_contracts(provided_contracts)
            .with_optional_contracts(support_declaration.optional_contracts().to_vec());
        match support_declaration.host_requirement() {
            Some(requirement) => declaration.with_host_requirement(requirement.clone()),
            None => declaration,
        }
    }
    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        self.support
            .materialize(&self.attachment, self.provider_module_id.clone())
    }
}

/// Typed requirements for one System occurrence plus one attached semantic.
pub struct SystemAugmentationRequirement<S, X>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
{
    base: SystemRequires<S>,
    augmentation: ContractRequirement<X::Contract>,
    target_module_id: ModuleId,
    augmentation_module_id: ModuleId,
    marker: PhantomData<X>,
}

impl<S, X> SystemAugmentationRequirement<S, X>
where
    S: PrimarySystemContract,
    X: SystemAugmentationDefinition<S>,
{
    pub fn base(&self) -> &SystemRequires<S> {
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
pub enum SystemAugmentationError {
    InvalidModuleId,
    TargetMismatch {
        expected_system_id: SystemId,
        actual_system_id: SystemId,
    },
}

impl fmt::Display for SystemAugmentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModuleId => formatter.write_str("invalid System augmentation module id"),
            Self::TargetMismatch {
                expected_system_id,
                actual_system_id,
            } => write!(
                formatter,
                "System augmentation targets `{expected_system_id}`, not `{actual_system_id}`"
            ),
        }
    }
}

impl std::error::Error for SystemAugmentationError {}

fn base_requirement<S>() -> SystemRequires<S>
where
    S: PrimarySystemContract,
{
    let key = S::primary_contract_key();
    match key.identity() {
        ContractIdentity::Provisional => SystemRequires::provisional(),
        ContractIdentity::Versioned(version) => SystemRequires::versioned(
            ContractVersionRequirement::parse(format!("={version}"))
                .expect("a ContractVersion always forms an exact requirement"),
        ),
    }
}

fn requirement_for_key<T>(key: &ContractKey<T>) -> ContractRequirement<T>
where
    T: Send + Sync + 'static,
{
    match key.identity() {
        ContractIdentity::Provisional => ContractRequirement::provisional(key.id().clone()),
        ContractIdentity::Versioned(version) => ContractRequirement::versioned(
            key.id().clone(),
            ContractVersionRequirement::parse(format!("={version}"))
                .expect("a ContractVersion always forms an exact requirement"),
        ),
    }
}

fn derive_module_id(
    target_module_id: &ModuleId,
    contract_id: &fabric_core::ContractId,
) -> Result<ModuleId, SystemAugmentationError> {
    let encoded_contract = contract_id
        .as_str()
        .bytes()
        .flat_map(|byte| [nibble(byte >> 4), nibble(byte & 0x0f)])
        .collect::<String>();
    ModuleId::new(format!(
        "{}.augmentation.{encoded_contract}",
        target_module_id.as_str()
    ))
    .map_err(|_| SystemAugmentationError::InvalidModuleId)
}

fn derive_support_module_id(
    attachment_module_id: &ModuleId,
) -> Result<ModuleId, SystemAugmentationError> {
    ModuleId::new(format!("{}.support", attachment_module_id.as_str()))
        .map_err(|_| SystemAugmentationError::InvalidModuleId)
}

fn nibble(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => unreachable!("nibble accepts 0..=15"),
    }
}
