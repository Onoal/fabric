use fabric_host::HostRequirement;

use crate::ModuleId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostMaterializationRequirement {
    module_id: ModuleId,
    requirement: HostRequirement,
}

impl HostMaterializationRequirement {
    pub fn new(module_id: ModuleId, requirement: HostRequirement) -> Self {
        Self {
            module_id,
            requirement,
        }
    }

    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn requirement(&self) -> &HostRequirement {
        &self.requirement
    }
}
