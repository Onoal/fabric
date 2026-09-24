use fabric_component::ComponentRelationName;
use fabric_core::{ContractRequirementDeclaration, ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric_system::{SystemCompatibilityError, SystemId, SystemSchemaDescriptor};

use super::SystemSelection;
use crate::authoring::RelationTargetDescriptor;
use crate::authoring::fabric::SemanticApiMetadata;

pub trait SystemDefinition: Sized + Send + Sync + 'static {
    /// Declarative input for one System selection.
    ///
    /// A Composition is reusable and each materialization receives its own
    /// Config value, so Config remains `Clone` even though normal no-Config
    /// macro authoring does not expose an empty Config object.
    type Config: Clone + Send + Sync + 'static;

    fn system_id() -> SystemId;

    fn schema() -> SystemSchemaDescriptor;

    fn api_metadata() -> SemanticApiMetadata {
        SemanticApiMetadata::empty()
    }

    fn select(config: Self::Config) -> Result<SystemSelection<Self>, SystemCompatibilityError> {
        SystemSelection::new(config)
    }

    fn declaration(selection: &SystemSelection<Self>) -> ModuleDeclaration;

    #[doc(hidden)]
    fn relation_declarations(
        consumer_module_id: ModuleId,
    ) -> Vec<(
        ComponentRelationName,
        RelationTargetDescriptor,
        ContractRequirementDeclaration,
    )> {
        let _ = consumer_module_id;
        Vec::new()
    }

    /// Whether this definition supplies its own static realization.
    ///
    /// This is declarative metadata only; Core must not construct a runtime
    /// merely to classify a built Composition.
    #[doc(hidden)]
    fn has_self_realization() -> bool {
        false
    }

    fn materialize(selection: &SystemSelection<Self>) -> Option<Box<dyn ModuleRuntime>> {
        let _ = selection;
        None
    }
}
