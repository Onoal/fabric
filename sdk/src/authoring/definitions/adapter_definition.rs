use std::marker::PhantomData;

use fabric_core::{ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric_host::HostRequirement;
use fabric_resource::{
    AdapterResourceSchemaSupport, ResourceCompatibilityError, ResourceSchemaDescriptor,
    ResourceSchemaRequirement,
};
use fabric_system::{
    AdapterSystemSchemaSupport, SystemCompatibilityError, SystemSchemaDescriptor,
    SystemSchemaRequirement,
};

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

pub trait AdapterDefinition: Clone + Send + Sync + 'static {
    type Target: Send + Sync + 'static;
    type Compatibility: Clone + Send + Sync + 'static;

    fn compatibility(&self) -> Self::Compatibility;

    #[doc(hidden)]
    fn bridge_mode(&self) -> AdapterBridgeMode {
        AdapterBridgeMode::ExplicitContract
    }

    fn host_requirement(&self) -> HostRequirement {
        HostRequirement::new()
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration;

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        let _ = provider_module_id;
        None
    }
}
