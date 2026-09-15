use crate::{
    AdapterSystemSchemaSupport, SystemCompatibilityError, SystemId,
    SystemSchemaCompatibilityRequirement, SystemSchemaDescriptor, SystemSchemaIdentity,
    SystemSchemaRequirement, SystemSchemaVersion,
};

#[test]
fn system_schema_version_is_distinct_from_resource_and_contract_versions() {
    assert_ne!(
        std::any::TypeId::of::<SystemSchemaVersion>(),
        std::any::TypeId::of::<fabric_core::ContractVersion>()
    );
    assert_ne!(
        std::any::TypeId::of::<SystemSchemaVersion>(),
        std::any::TypeId::of::<fabric_resource::ResourceSchemaVersion>()
    );
}

#[test]
fn versioned_system_schema_compatibility_is_explicit() {
    let system = SystemId::new("fabric.test.operations").expect("system id");
    let schema = SystemSchemaDescriptor::versioned(
        system,
        SystemSchemaVersion::parse("1.4.0").expect("schema version"),
    );
    let compatible = SystemSchemaCompatibilityRequirement::versioned(
        SystemSchemaRequirement::parse("^1").expect("requirement"),
    );
    let incompatible = SystemSchemaCompatibilityRequirement::versioned(
        SystemSchemaRequirement::parse("^2").expect("requirement"),
    );

    assert!(schema.ensure_compatible(&compatible).is_ok());
    assert!(matches!(
        schema.ensure_compatible(&incompatible),
        Err(SystemCompatibilityError::SchemaIncompatible { .. })
    ));
}

#[test]
fn system_schema_descriptor_preserves_identity_and_scope() {
    let system = SystemId::new("fabric.test.operations").expect("system id");
    let schema = SystemSchemaDescriptor::provisional(system.clone());

    assert_eq!(schema.system(), &system);
    assert_eq!(schema.identity(), &SystemSchemaIdentity::provisional());
    assert!(schema.ensure_system(&system).is_ok());
}

#[test]
fn adapter_system_schema_support_uses_structured_identity_and_compatibility_checks() {
    let system = SystemId::new("fabric.test.system").expect("system id");
    let schema = SystemSchemaDescriptor::versioned(
        system.clone(),
        SystemSchemaVersion::parse("1.2.0").expect("schema version"),
    );

    assert!(
        AdapterSystemSchemaSupport::versioned(
            system.clone(),
            SystemSchemaRequirement::parse("^1").expect("schema requirement"),
        )
        .accepts_schema(&schema)
        .is_ok()
    );
    assert!(matches!(
        AdapterSystemSchemaSupport::provisional(
            SystemId::new("fabric.test.foreign").expect("foreign system id"),
        )
        .accepts_schema(&schema),
        Err(SystemCompatibilityError::SystemIdentityMismatch { .. })
    ));
    assert!(matches!(
        AdapterSystemSchemaSupport::versioned(
            system,
            SystemSchemaRequirement::parse("^2").expect("schema requirement"),
        )
        .accepts_schema(&schema),
        Err(SystemCompatibilityError::SchemaIncompatible { .. })
    ));
}
