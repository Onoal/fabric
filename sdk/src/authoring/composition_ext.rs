use fabric_core::{Composition as CoreComposition, CompositionError, Instance};
use fabric_host::HostDescriptor;

use crate::ids::IntoInstanceId;

/// Advanced named-instance convenience for [`CoreComposition`].
///
/// Normal SDK [`crate::Composition`] has inherent high-level materialization
/// methods and intentionally does not implement this trait.
pub trait CompositionExt {
    fn materialize_core<I>(&self, instance_id: I) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId;

    fn materialize_core_on<I>(
        &self,
        instance_id: I,
        host: &HostDescriptor,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId;
}

impl CompositionExt for CoreComposition {
    fn materialize_core<I>(&self, instance_id: I) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        CoreComposition::materialize(self, instance_id.into_instance_id()?)
    }

    fn materialize_core_on<I>(
        &self,
        instance_id: I,
        host: &HostDescriptor,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        CoreComposition::materialize_on(self, instance_id.into_instance_id()?, host)
    }
}
