use crate::{
    BindingConsumer, BindingConsumerId, BindingConsumerKind, BindingId, BindingName,
    BindingTargetId, BindingTargetKind,
};
use fabric_resource::{
    ResourceBoundaryId, ResourceContext, ResourceId, ResourceInstanceId, ResourceName,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct SyntheticQueueRef {
    id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SyntheticResourceInstanceBinding {
    binding_id: BindingId,
    consumer: ResourceInstanceId,
    target: SyntheticQueueRef,
}

impl SyntheticQueueRef {
    fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl SyntheticResourceInstanceBinding {
    fn new(consumer: ResourceInstanceId, target: SyntheticQueueRef) -> Self {
        let name = BindingName::new("metadata").expect("binding name");
        let binding_id = BindingId::canonical(
            &BindingConsumer::new(
                BindingConsumerKind::new("resource-instance").expect("consumer kind"),
                BindingConsumerId::new(consumer.as_str()).expect("consumer id"),
            ),
            &name,
            &BindingTargetKind::new("synthetic.queue-ref").expect("target kind"),
            &BindingTargetId::new(target.id.clone()).expect("target id"),
        );
        Self {
            binding_id,
            consumer,
            target,
        }
    }
}

#[test]
fn binding_identity_is_open_to_unknown_consumers_and_targets() {
    let consumer = BindingConsumer::new(
        BindingConsumerKind::new("synthetic.resource-instance").expect("consumer kind"),
        BindingConsumerId::new("instance:alpha").expect("consumer id"),
    );
    let name = BindingName::new("metadata").expect("binding name");
    let target_kind = BindingTargetKind::new("synthetic.queue-ref").expect("target kind");
    let target_id = BindingTargetId::new("queue://beta").expect("target id");

    let binding_id = BindingId::canonical(&consumer, &name, &target_kind, &target_id);

    assert!(binding_id.as_str().starts_with("fabric-binding-"));
}

#[test]
fn binding_identity_parsing_preserves_existing_prefix() {
    let id = BindingId::parse(
        "fabric-binding-deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
    )
    .expect("binding id");

    assert_eq!(
        id.as_str(),
        "fabric-binding-deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
    );
}

#[test]
fn resource_instance_can_participate_as_a_binding_consumer_without_shared_catalog_edits() {
    let consumer = ResourceInstanceId::canonical(
        &ResourceId::new("synthetic.cache").expect("resource"),
        &ResourceContext::root(ResourceBoundaryId::new("apps.notes").expect("boundary")),
        &ResourceName::new("primary").expect("name"),
    );
    let target = SyntheticQueueRef::new("queue://metadata");
    let binding = SyntheticResourceInstanceBinding::new(consumer.clone(), target.clone());

    assert_eq!(binding.consumer, consumer);
    assert_eq!(binding.target, target);
    assert!(binding.binding_id.as_str().starts_with("fabric-binding-"));
}
