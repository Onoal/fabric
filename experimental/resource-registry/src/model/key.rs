use fabric_core::{CompositionError, ModuleId};

pub(crate) fn resource_registry_module_id() -> Result<ModuleId, CompositionError> {
    ModuleId::new("fabric.resource.registry.module")
}

pub(crate) fn validate_resource_registry_key(
    value: &str,
    label: &str,
) -> Result<(), crate::ResourceRegistryError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.contains("..");
    if valid {
        Ok(())
    } else {
        Err(crate::ResourceRegistryError::invalid_input(format!(
            "invalid {label}"
        )))
    }
}
