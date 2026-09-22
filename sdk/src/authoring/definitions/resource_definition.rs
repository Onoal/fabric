use fabric_core::{ContractRequirement, ModuleDeclaration, ModuleRuntime};
use fabric_resource::{ResourceError, ResourceId, ResourceName, ResourceSchemaDescriptor};

use super::ResourceSelection;

pub trait ResourceDefinition: Sized + Send + Sync + 'static {
    /// Declarative input for one Resource selection.
    ///
    /// A Composition is reusable and each materialization receives its own
    /// Config value, so Config remains `Clone` even though normal no-Config
    /// macro authoring does not expose an empty Config object.
    type Config: Clone + Send + Sync + 'static;

    fn resource_id() -> ResourceId;

    fn schema() -> ResourceSchemaDescriptor;

    fn select(
        name: impl IntoResourceName,
        config: Self::Config,
    ) -> Result<ResourceSelection<Self>, ResourceError> {
        ResourceSelection::new(name, config)
    }

    fn declaration(selection: &ResourceSelection<Self>) -> ModuleDeclaration;

    fn materialize(selection: &ResourceSelection<Self>) -> Option<Box<dyn ModuleRuntime>> {
        let _ = selection;
        None
    }
}

pub trait AdaptableResourceDefinition: ResourceDefinition {
    type RealizationContract: Send + Sync + 'static;

    fn realization_requirement() -> ContractRequirement<Self::RealizationContract>;

    /// Whether the target's primary semantic API is its effective normal
    /// Adapter contract. Differential realization declarations return
    /// `false` and expose their owner-derived effective contract instead.
    #[doc(hidden)]
    fn supports_semantic_api_adapter() -> bool {
        false
    }
}

pub trait IntoResourceName {
    fn into_resource_name(self) -> Result<ResourceName, ResourceError>;
}

impl IntoResourceName for ResourceName {
    fn into_resource_name(self) -> Result<ResourceName, ResourceError> {
        Ok(self)
    }
}

impl IntoResourceName for &str {
    fn into_resource_name(self) -> Result<ResourceName, ResourceError> {
        ResourceName::new(self)
    }
}

impl IntoResourceName for String {
    fn into_resource_name(self) -> Result<ResourceName, ResourceError> {
        ResourceName::new(self)
    }
}
