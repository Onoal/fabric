use std::collections::BTreeSet;

use crate::{
    HostArchitecture, HostCompatibilityError, HostDescriptor, HostFacilityId, HostOperatingSystem,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostRequirement {
    operating_systems: BTreeSet<HostOperatingSystem>,
    architectures: BTreeSet<HostArchitecture>,
    required_facilities: BTreeSet<HostFacilityId>,
}

impl HostRequirement {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_operating_system(mut self, operating_system: HostOperatingSystem) -> Self {
        self.operating_systems.insert(operating_system);
        self
    }

    pub fn allow_architecture(mut self, architecture: HostArchitecture) -> Self {
        self.architectures.insert(architecture);
        self
    }

    pub fn require_facility(mut self, facility: HostFacilityId) -> Self {
        self.required_facilities.insert(facility);
        self
    }

    pub fn evaluate(&self, host: &HostDescriptor) -> Result<(), HostCompatibilityError> {
        if !self.operating_systems.is_empty()
            && !self.operating_systems.contains(host.operating_system())
        {
            return Err(HostCompatibilityError::UnsupportedOperatingSystem {
                actual: host.operating_system().clone(),
                allowed: self.operating_systems.iter().cloned().collect(),
            });
        }
        if !self.architectures.is_empty() && !self.architectures.contains(host.architecture()) {
            return Err(HostCompatibilityError::UnsupportedArchitecture {
                actual: host.architecture().clone(),
                allowed: self.architectures.iter().cloned().collect(),
            });
        }
        let missing = self
            .required_facilities
            .iter()
            .filter(|facility| !host.facilities().contains(*facility))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(HostCompatibilityError::MissingRequiredFacilities { missing });
        }
        Ok(())
    }

    pub fn operating_systems(&self) -> &BTreeSet<HostOperatingSystem> {
        &self.operating_systems
    }

    pub fn architectures(&self) -> &BTreeSet<HostArchitecture> {
        &self.architectures
    }

    pub fn required_facilities(&self) -> &BTreeSet<HostFacilityId> {
        &self.required_facilities
    }
}
