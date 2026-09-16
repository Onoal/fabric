use fabric_core::{ContractIdentity, ContractKey, ContractRequirement, ContractVersionRequirement};

/// Converts a semantic contract key into the exact requirement for that
/// semantic contract. This is authoring mechanics, not augmentation ontology.
pub(crate) fn requirement_for_key<T>(key: &ContractKey<T>) -> ContractRequirement<T>
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
