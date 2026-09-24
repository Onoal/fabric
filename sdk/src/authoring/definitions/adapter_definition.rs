use std::fmt;
use std::marker::PhantomData;

use fabric_core::{ContractRequirementDeclaration, ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric_host::HostRequirement;
use fabric_resource::{
    AdapterResourceSchemaSupport, ResourceCompatibilityError, ResourceSchemaDescriptor,
    ResourceSchemaRequirement,
};
use fabric_system::{
    AdapterSystemSchemaSupport, SystemCompatibilityError, SystemSchemaDescriptor,
    SystemSchemaRequirement,
};

use crate::authoring::{ComponentDefinition, RelationTargetDescriptor};
use fabric_component::{ComponentError, ComponentParticipationScope, ComponentRelationName};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterDefinitionIdError;

impl fmt::Display for AdapterDefinitionIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Adapter definition ID must be a non-empty stable key")
    }
}

impl std::error::Error for AdapterDefinitionIdError {}

/// Stable machine identity for one concrete Adapter realization definition.
///
/// It identifies neither a configured Adapter value, a Core provider module,
/// a live runtime occurrence, nor the semantic target it realizes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdapterDefinitionId(String);

impl AdapterDefinitionId {
    pub fn new(value: impl Into<String>) -> Result<Self, AdapterDefinitionIdError> {
        let value = value.into();
        let valid = !value.is_empty()
            && value.len() <= 128
            && value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'.' | b'-' | b'_')
            })
            && !value.starts_with('.')
            && !value.ends_with('.')
            && !value.contains("..");
        if valid {
            Ok(Self(value))
        } else {
            Err(AdapterDefinitionIdError)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AdapterDefinitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Target-owned context needed only when a canonical Adapter realizes a
/// Component participation. Resource and System targets provide an empty
/// implementation so `adapter!` remains one type-driven language.
#[doc(hidden)]
pub trait ComponentAdapterTarget: Send + Sync + 'static {
    type ComponentConfig: Clone + Send + Sync + 'static;
    type ComponentRelations: Clone + Send + Sync + 'static;

    fn component_adapter_context(
        config: &Self::ComponentConfig,
        scope: &ComponentParticipationScope,
    ) -> Result<(Self::ComponentConfig, Self::ComponentRelations), ComponentError>;

    /// Whether this target is a Component participation rather than a
    /// Resource or System provider.  Canonical Adapter lowering uses this
    /// target-owned fact to avoid allocating participation state for the
    /// provider-module lifetime.
    #[doc(hidden)]
    fn is_component_participation_target() -> bool;
}

/// Factory implemented by generated canonical Adapter provider runtimes.
/// Its ComponentInstanceBinding path creates a fresh participation runtime rather than
/// reusing the provider module's lifetime state.
#[doc(hidden)]
pub trait CanonicalComponentAdapterRuntime<T>: Send + Sync + 'static
where
    T: ComponentAdapterTarget,
{
    type ParticipationRuntime: Send + Sync + 'static;

    fn provider_runtime(&self) -> std::sync::Arc<Self::ParticipationRuntime>;

    fn prepare_component_runtime(
        &self,
        config: T::ComponentConfig,
        relations: T::ComponentRelations,
    ) -> Result<Self::ParticipationRuntime, ComponentError>;

    fn component_prepare(&self, runtime: &Self::ParticipationRuntime)
    -> Result<(), ComponentError>;

    fn component_teardown(
        &self,
        runtime: &Self::ParticipationRuntime,
    ) -> Result<(), ComponentError>;
}

/// Compatibility accepted by `ComponentSpec::using`. Canonical adapters are
/// inferred from the ComponentInstanceBinding target; handwritten advanced adapters retain
/// their exact `ComponentId` compatibility.
#[doc(hidden)]
pub trait ComponentAdapterCompatibility<C>: Clone + Send + Sync + 'static
where
    C: ComponentDefinition,
{
    fn accepts_component(&self) -> Result<(), ComponentError>;
}

/// Internal bridge selection for an Adapter definition.
///
/// `SemanticApi` means the Adapter provider itself exports the target's
/// primary semantic API.  It is deliberately runtime-authoring machinery,
/// not a new Fabric semantic subject.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterBridgeMode {
    /// An advanced/Core Adapter explicitly supplies an independently typed
    /// contract. Normal macro authoring never selects this mode.
    ExplicitContract,
    SemanticApi,
    /// The Adapter provides a target-owned effective realization contract;
    /// the semantic target composes its public API from that contract.
    DifferentialSemanticApi,
}

/// Target-derived support used by canonical `adapter! { Name for Target }`
/// authoring.  A target's schema determines exact support unless an author
/// deliberately supplies an advanced `supports:` override.
#[doc(hidden)]
pub struct CanonicalAdapterSupport<T> {
    requirement: CanonicalAdapterSupportRequirement,
    target: PhantomData<T>,
}

impl<T> Clone for CanonicalAdapterSupport<T> {
    fn clone(&self) -> Self {
        Self {
            requirement: self.requirement.clone(),
            target: PhantomData,
        }
    }
}

#[derive(Clone)]
enum CanonicalAdapterSupportRequirement {
    Inferred,
    Provisional,
    Versioned(String),
}

impl<T> CanonicalAdapterSupport<T> {
    pub fn inferred() -> Self {
        Self {
            requirement: CanonicalAdapterSupportRequirement::Inferred,
            target: PhantomData,
        }
    }

    pub fn provisional() -> Self {
        Self {
            requirement: CanonicalAdapterSupportRequirement::Provisional,
            target: PhantomData,
        }
    }

    pub fn versioned(requirement: impl Into<String>) -> Self {
        Self {
            requirement: CanonicalAdapterSupportRequirement::Versioned(requirement.into()),
            target: PhantomData,
        }
    }
}

/// Compatibility lowering used by Resource selections.  It is kept below the
/// normal authoring surface so existing explicit adapter definitions and the
/// canonical target-derived form share one selection path.
#[doc(hidden)]
pub trait ResourceAdapterCompatibility<R>: Clone + Send + Sync + 'static {
    fn accepts_resource_schema(
        &self,
        schema: &ResourceSchemaDescriptor,
    ) -> Result<(), ResourceCompatibilityError>;
}

impl<R> ResourceAdapterCompatibility<R> for AdapterResourceSchemaSupport {
    fn accepts_resource_schema(
        &self,
        schema: &ResourceSchemaDescriptor,
    ) -> Result<(), ResourceCompatibilityError> {
        self.accepts_schema(schema)
    }
}

impl<R> ResourceAdapterCompatibility<R> for CanonicalAdapterSupport<R>
where
    R: crate::authoring::ResourceDefinition,
{
    fn accepts_resource_schema(
        &self,
        schema: &ResourceSchemaDescriptor,
    ) -> Result<(), ResourceCompatibilityError> {
        let support = match &self.requirement {
            CanonicalAdapterSupportRequirement::Inferred => match schema.identity() {
                fabric_resource::ResourceSchemaIdentity::Provisional => {
                    AdapterResourceSchemaSupport::provisional(schema.resource().clone())
                }
                fabric_resource::ResourceSchemaIdentity::Versioned(version) => {
                    AdapterResourceSchemaSupport::versioned(
                        schema.resource().clone(),
                        ResourceSchemaRequirement::parse(format!("={version}"))
                            .expect("canonical adapter support derives a valid exact version"),
                    )
                }
            },
            CanonicalAdapterSupportRequirement::Provisional => {
                AdapterResourceSchemaSupport::provisional(R::resource_id())
            }
            CanonicalAdapterSupportRequirement::Versioned(requirement) => {
                AdapterResourceSchemaSupport::versioned(
                    R::resource_id(),
                    ResourceSchemaRequirement::parse(requirement)
                        .expect("adapter! validates static support requirements"),
                )
            }
        };
        support.accepts_schema(schema)
    }
}

/// Compatibility lowering used by System selections.  See
/// [`ResourceAdapterCompatibility`] for why this stays SDK machinery.
#[doc(hidden)]
pub trait SystemAdapterCompatibility<S>: Clone + Send + Sync + 'static {
    fn accepts_system_schema(
        &self,
        schema: &SystemSchemaDescriptor,
    ) -> Result<(), SystemCompatibilityError>;
}

impl<S> SystemAdapterCompatibility<S> for AdapterSystemSchemaSupport {
    fn accepts_system_schema(
        &self,
        schema: &SystemSchemaDescriptor,
    ) -> Result<(), SystemCompatibilityError> {
        self.accepts_schema(schema)
    }
}

impl<S> SystemAdapterCompatibility<S> for CanonicalAdapterSupport<S>
where
    S: crate::authoring::SystemDefinition,
{
    fn accepts_system_schema(
        &self,
        schema: &SystemSchemaDescriptor,
    ) -> Result<(), SystemCompatibilityError> {
        let support = match &self.requirement {
            CanonicalAdapterSupportRequirement::Inferred => match schema.identity() {
                fabric_system::SystemSchemaIdentity::Provisional => {
                    AdapterSystemSchemaSupport::provisional(schema.system().clone())
                }
                fabric_system::SystemSchemaIdentity::Versioned(version) => {
                    AdapterSystemSchemaSupport::versioned(
                        schema.system().clone(),
                        SystemSchemaRequirement::parse(format!("={version}"))
                            .expect("canonical adapter support derives a valid exact version"),
                    )
                }
            },
            CanonicalAdapterSupportRequirement::Provisional => {
                AdapterSystemSchemaSupport::provisional(S::system_id())
            }
            CanonicalAdapterSupportRequirement::Versioned(requirement) => {
                AdapterSystemSchemaSupport::versioned(
                    S::system_id(),
                    SystemSchemaRequirement::parse(requirement)
                        .expect("adapter! validates static support requirements"),
                )
            }
        };
        support.accepts_schema(schema)
    }
}

impl<C> ComponentAdapterCompatibility<C> for fabric_component::ComponentId
where
    C: ComponentDefinition,
{
    fn accepts_component(&self) -> Result<(), ComponentError> {
        if self == &C::component_id() {
            Ok(())
        } else {
            Err(ComponentError::Unavailable)
        }
    }
}

impl<C> ComponentAdapterCompatibility<C> for CanonicalAdapterSupport<C>
where
    C: ComponentDefinition,
{
    fn accepts_component(&self) -> Result<(), ComponentError> {
        match self.requirement {
            CanonicalAdapterSupportRequirement::Inferred => Ok(()),
            _ => Err(ComponentError::UnsupportedComponentAdapterSupportOverride),
        }
    }
}

pub trait AdapterDefinition: Clone + Send + Sync + 'static {
    type Target: Send + Sync + 'static;
    type Compatibility: Clone + Send + Sync + 'static;

    fn adapter_definition_id(&self) -> AdapterDefinitionId;

    fn compatibility(&self) -> Self::Compatibility;

    #[doc(hidden)]
    fn bridge_mode(&self) -> AdapterBridgeMode {
        AdapterBridgeMode::ExplicitContract
    }

    fn host_requirement(&self) -> HostRequirement {
        HostRequirement::new()
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration;

    #[doc(hidden)]
    fn relation_declarations(
        &self,
        provider_module_id: ModuleId,
    ) -> Vec<(
        ComponentRelationName,
        RelationTargetDescriptor,
        ContractRequirementDeclaration,
    )> {
        let _ = provider_module_id;
        Vec::new()
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        let _ = provider_module_id;
        None
    }
}

#[cfg(test)]
mod adapter_definition_id_tests {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    use super::AdapterDefinitionId;

    #[test]
    fn accepts_and_orders_stable_definition_ids() {
        let first = AdapterDefinitionId::new("example.memory-store").expect("valid ID");
        let same = AdapterDefinitionId::new("example.memory-store").expect("valid ID");
        let later = AdapterDefinitionId::new("example.postgres-store").expect("valid ID");

        assert_eq!(first.as_str(), "example.memory-store");
        assert_eq!(first.to_string(), "example.memory-store");
        assert_eq!(first, same);
        assert!(first < later);

        let mut first_hasher = DefaultHasher::new();
        first.hash(&mut first_hasher);
        let mut same_hasher = DefaultHasher::new();
        same.hash(&mut same_hasher);
        assert_eq!(first_hasher.finish(), same_hasher.finish());
    }

    #[test]
    fn rejects_non_stable_definition_id_forms() {
        for invalid in [
            "",
            ".leading",
            "trailing.",
            "double..dot",
            "Uppercase",
            "space value",
        ] {
            assert!(
                AdapterDefinitionId::new(invalid).is_err(),
                "{invalid:?} must be rejected"
            );
        }
    }
}
