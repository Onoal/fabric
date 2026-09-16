use crate::{
    AdapterResourceSchemaSupport, ResourceBoundaryId, ResourceCompatibilityError,
    ResourceCompatibilityRole, ResourceContext, ResourceId, ResourceInstanceId, ResourceName,
    ResourceRequirement, ResourceSchemaDescriptor, ResourceSchemaRequirement,
    ResourceSchemaVersion, ResourceScope, resource_context_binding_provenance,
};

#[test]
fn open_resource_instance_identity_is_stable_without_closed_catalog() {
    let boundary = ResourceBoundaryId::new("apps.notes").expect("boundary");
    let context = ResourceContext::root(boundary.clone())
        .child(ResourceScope::new("release").expect("scope"));
    let name = ResourceName::new("primary").expect("name");

    let database = ResourceId::new("database").expect("database resource");
    let kv = ResourceId::new("kv").expect("kv resource");
    let queue = ResourceId::new("queue").expect("queue resource");

    let database_instance = ResourceInstanceId::canonical(&database, &context, &name);
    let database_instance_again = ResourceInstanceId::canonical(&database, &context, &name);
    let kv_instance = ResourceInstanceId::canonical(&kv, &context, &name);
    let queue_instance = ResourceInstanceId::canonical(&queue, &context, &name);

    assert_eq!(database_instance, database_instance_again);
    assert_eq!(database_instance.resource(), database);
    assert_eq!(kv_instance.resource(), kv);
    assert_eq!(queue_instance.resource(), queue);
    assert_ne!(database_instance, kv_instance);
    assert_ne!(database_instance, queue_instance);
    assert!(
        database_instance
            .as_str()
            .starts_with("fabric-resource-database-")
    );
    assert!(kv_instance.as_str().starts_with("fabric-resource-kv-"));
    assert!(
        queue_instance
            .as_str()
            .starts_with("fabric-resource-queue-")
    );
}

#[test]
fn structurally_distinct_resource_contexts_do_not_collapse_consumer_provenance() {
    let direct = ResourceContext::root(ResourceBoundaryId::new("apps:notes").expect("boundary"));
    let nested = ResourceContext::root(ResourceBoundaryId::new("apps").expect("boundary"))
        .child(ResourceScope::new("notes").expect("scope"));

    let direct_provenance = resource_context_binding_provenance(&direct);
    let nested_provenance = resource_context_binding_provenance(&nested);

    assert_ne!(
        direct_provenance.consumer_id(),
        nested_provenance.consumer_id()
    );
    assert_ne!(
        direct_provenance.identity_material(),
        nested_provenance.identity_material()
    );
}

#[test]
fn resource_requirement_is_open_and_not_enum_backed() {
    let resource = ResourceId::new("future.queue").expect("resource id");
    let requirement = ResourceRequirement::new(resource.clone(), "jobs").expect("requirement");

    assert_eq!(requirement.resource(), &resource);
    assert_eq!(requirement.name.as_str(), "jobs");
}

#[test]
fn resource_schema_version_preserves_its_semantic_revision() {
    let schema = ResourceSchemaVersion::parse("1.2.0").expect("schema version");

    assert_eq!(schema.to_string(), "1.2.0");
}

#[test]
fn versioned_resource_requirement_accepts_matching_schema_and_rejects_major_breaks() {
    let resource = ResourceId::new("future.queue").expect("resource");
    let requirement = ResourceRequirement::versioned(
        resource.clone(),
        "jobs",
        ResourceSchemaRequirement::parse("^1").expect("requirement"),
    )
    .expect("requirement");
    let compatible = ResourceSchemaDescriptor::versioned(
        resource.clone(),
        ResourceSchemaVersion::parse("1.4.0").expect("schema version"),
    );
    let incompatible = ResourceSchemaDescriptor::versioned(
        resource.clone(),
        ResourceSchemaVersion::parse("2.0.0").expect("schema version"),
    );

    assert!(requirement.accepts_schema(&compatible));
    assert!(!requirement.accepts_schema(&incompatible));
}

#[test]
fn adapter_schema_support_obeys_resource_schema_compatibility() {
    let resource = ResourceId::new("future.queue").expect("resource");
    let schema = ResourceSchemaDescriptor::versioned(
        resource.clone(),
        ResourceSchemaVersion::parse("1.3.0").expect("schema version"),
    );
    let compatible = AdapterResourceSchemaSupport::versioned(
        resource.clone(),
        ResourceSchemaRequirement::parse("^1").expect("requirement"),
    );
    let incompatible = AdapterResourceSchemaSupport::versioned(
        resource,
        ResourceSchemaRequirement::parse("^2").expect("requirement"),
    );

    assert!(compatible.accepts_schema(&schema).is_ok());
    assert!(matches!(
        incompatible.accepts_schema(&schema),
        Err(ResourceCompatibilityError::AdapterSupportIncompatible { .. })
    ));
}

#[test]
fn resource_schema_identity_must_match_requirement_before_schema_compatibility() {
    let required = ResourceId::new("future.queue").expect("resource");
    let wrong = ResourceId::new("future.cache").expect("resource");
    let requirement = ResourceRequirement::versioned(
        required.clone(),
        "jobs",
        ResourceSchemaRequirement::parse("^1").expect("requirement"),
    )
    .expect("requirement");
    let schema = ResourceSchemaDescriptor::versioned(
        wrong.clone(),
        ResourceSchemaVersion::parse("1.4.0").expect("schema version"),
    );
    let support = AdapterResourceSchemaSupport::versioned(
        required,
        ResourceSchemaRequirement::parse("^1").expect("requirement"),
    );

    assert!(matches!(
        requirement.evaluate_compatibility(&schema, &support),
        Err(ResourceCompatibilityError::ResourceIdentityMismatch {
            role: ResourceCompatibilityRole::Schema,
            expected,
            actual,
        }) if expected.as_str() == "future.queue" && actual.as_str() == "future.cache"
    ));
}

#[test]
fn adapter_support_identity_must_match_requirement_before_schema_compatibility() {
    let required = ResourceId::new("future.queue").expect("resource");
    let wrong = ResourceId::new("future.cache").expect("resource");
    let requirement = ResourceRequirement::versioned(
        required.clone(),
        "jobs",
        ResourceSchemaRequirement::parse("^1").expect("requirement"),
    )
    .expect("requirement");
    let schema = ResourceSchemaDescriptor::versioned(
        required.clone(),
        ResourceSchemaVersion::parse("1.4.0").expect("schema version"),
    );
    let support = AdapterResourceSchemaSupport::versioned(
        wrong.clone(),
        ResourceSchemaRequirement::parse("^1").expect("requirement"),
    );

    assert!(matches!(
        requirement.evaluate_compatibility(&schema, &support),
        Err(ResourceCompatibilityError::ResourceIdentityMismatch {
            role: ResourceCompatibilityRole::AdapterSupport,
            expected,
            actual,
        }) if expected.as_str() == "future.queue" && actual.as_str() == "future.cache"
    ));
}

#[test]
fn resource_requirement_and_adapter_support_are_evaluated_together_before_materialization() {
    let resource = ResourceId::new("future.queue").expect("resource");
    let requirement = ResourceRequirement::versioned(
        resource.clone(),
        "jobs",
        ResourceSchemaRequirement::parse("^1").expect("requirement"),
    )
    .expect("requirement");
    let schema = ResourceSchemaDescriptor::versioned(
        resource.clone(),
        ResourceSchemaVersion::parse("1.5.0").expect("schema version"),
    );
    let compatible_adapter = AdapterResourceSchemaSupport::versioned(
        resource.clone(),
        ResourceSchemaRequirement::parse("^1").expect("requirement"),
    );
    let incompatible_adapter = AdapterResourceSchemaSupport::versioned(
        resource.clone(),
        ResourceSchemaRequirement::parse("^2").expect("requirement"),
    );

    assert!(
        requirement
            .evaluate_compatibility(&schema, &compatible_adapter)
            .is_ok()
    );
    assert!(matches!(
        requirement.evaluate_compatibility(&schema, &incompatible_adapter),
        Err(ResourceCompatibilityError::AdapterSupportIncompatible { .. })
    ));
}

#[test]
fn provisional_resource_schema_rules_are_explicit() {
    let resource = ResourceId::new("future.queue").expect("resource");
    let provisional_requirement =
        ResourceRequirement::provisional(resource.clone(), "jobs").expect("requirement");
    let provisional_schema = ResourceSchemaDescriptor::provisional(resource.clone());
    let versioned_schema = ResourceSchemaDescriptor::versioned(
        resource.clone(),
        ResourceSchemaVersion::parse("1.0.0").expect("schema version"),
    );
    let provisional_adapter = AdapterResourceSchemaSupport::provisional(resource.clone());

    assert!(
        provisional_requirement
            .evaluate_compatibility(&provisional_schema, &provisional_adapter)
            .is_ok()
    );
    assert!(matches!(
        provisional_requirement.evaluate_compatibility(&versioned_schema, &provisional_adapter),
        Err(ResourceCompatibilityError::ResourceRequirementIncompatible { .. })
    ));

    let foreign_schema =
        ResourceSchemaDescriptor::provisional(ResourceId::new("future.cache").expect("resource"));
    let foreign_adapter = AdapterResourceSchemaSupport::provisional(
        ResourceId::new("future.cache").expect("resource"),
    );
    assert!(matches!(
        provisional_requirement.evaluate_compatibility(&foreign_schema, &provisional_adapter),
        Err(ResourceCompatibilityError::ResourceIdentityMismatch {
            role: ResourceCompatibilityRole::Schema,
            ..
        })
    ));
    assert!(matches!(
        provisional_requirement.evaluate_compatibility(&provisional_schema, &foreign_adapter),
        Err(ResourceCompatibilityError::ResourceIdentityMismatch {
            role: ResourceCompatibilityRole::AdapterSupport,
            ..
        })
    ));
}

#[test]
fn adapter_identity_does_not_enter_resource_instance_identity() {
    let boundary = ResourceBoundaryId::new("apps.notes").expect("boundary");
    let context =
        ResourceContext::root(boundary).child(ResourceScope::new("release").expect("scope"));
    let resource = ResourceId::new("queue").expect("resource");
    let name = ResourceName::new("primary").expect("name");
    let instance = ResourceInstanceId::canonical(&resource, &context, &name);

    for forbidden in ["sqlite", "fjall", "pingora", "deno", "process"] {
        assert!(
            !instance.as_str().contains(forbidden),
            "resource instance identity must not encode concrete adapter {forbidden}"
        );
    }
}
