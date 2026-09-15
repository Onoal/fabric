use fabric_core::{ModuleDeclaration, ModuleId, ModuleRuntime};
use fabric_host::HostRequirement;

pub trait AdapterDefinition: Clone + Send + Sync + 'static {
    type Target: Send + Sync + 'static;
    type SchemaSupport: Clone + Send + Sync + 'static;

    fn schema_support(&self) -> Self::SchemaSupport;

    fn host_requirement(&self) -> HostRequirement {
        HostRequirement::new()
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration;

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        let _ = provider_module_id;
        None
    }
}
