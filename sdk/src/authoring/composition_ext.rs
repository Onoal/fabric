use fabric_core::{Composition, CompositionError, Instance};
use fabric_host::HostDescriptor;

use crate::ids::IntoInstanceId;

pub trait CompositionExt {
    fn materialize_named<I>(&self, instance_id: I) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId;

    fn materialize_named_on<I>(
        &self,
        instance_id: I,
        host: &HostDescriptor,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId;
}

impl CompositionExt for Composition {
    fn materialize_named<I>(&self, instance_id: I) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.materialize(instance_id.into_instance_id()?)
    }

    fn materialize_named_on<I>(
        &self,
        instance_id: I,
        host: &HostDescriptor,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.materialize_on(instance_id.into_instance_id()?, host)
    }
}
