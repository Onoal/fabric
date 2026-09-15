use fabric_core::{ModuleDeclaration, ModuleRuntime};
use fabric_system::{SystemCompatibilityError, SystemId, SystemSchemaDescriptor};

use super::SystemSelection;

pub trait SystemDefinition: Sized + Send + Sync + 'static {
    type Config: Clone + Send + Sync + 'static;

    fn system_id() -> SystemId;

    fn schema() -> SystemSchemaDescriptor;

    fn select(config: Self::Config) -> Result<SystemSelection<Self>, SystemCompatibilityError> {
        SystemSelection::new(config)
    }

    fn declaration(selection: &SystemSelection<Self>) -> ModuleDeclaration;

    fn materialize(selection: &SystemSelection<Self>) -> Option<Box<dyn ModuleRuntime>> {
        let _ = selection;
        None
    }
}
