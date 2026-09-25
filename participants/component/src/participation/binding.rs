use fabric_core::InstanceId;

use crate::{ComponentHost, ComponentId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentInstanceBinding {
    component_id: ComponentId,
    instance_id: InstanceId,
}

impl ComponentInstanceBinding {
    pub fn bind(component_id: ComponentId, runtime: &ComponentHost) -> Self {
        Self {
            component_id,
            instance_id: runtime.instance_id(),
        }
    }

    pub fn for_instance(component_id: ComponentId, instance_id: InstanceId) -> Self {
        Self {
            component_id,
            instance_id,
        }
    }

    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }
}
