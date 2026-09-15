use std::collections::BTreeSet;

use crate::{HostArchitecture, HostFacilityId, HostOperatingSystem};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDescriptor {
    operating_system: HostOperatingSystem,
    architecture: HostArchitecture,
    facilities: BTreeSet<HostFacilityId>,
}

impl HostDescriptor {
    pub fn new(operating_system: HostOperatingSystem, architecture: HostArchitecture) -> Self {
        Self {
            operating_system,
            architecture,
            facilities: BTreeSet::new(),
        }
    }

    pub fn native() -> Self {
        Self::new(
            HostOperatingSystem::new(std::env::consts::OS).expect("native operating system"),
            HostArchitecture::new(std::env::consts::ARCH).expect("native architecture"),
        )
    }

    pub fn with_facility(mut self, facility: HostFacilityId) -> Self {
        self.facilities.insert(facility);
        self
    }

    pub fn with_facilities(mut self, facilities: impl IntoIterator<Item = HostFacilityId>) -> Self {
        self.facilities.extend(facilities);
        self
    }

    pub fn operating_system(&self) -> &HostOperatingSystem {
        &self.operating_system
    }

    pub fn architecture(&self) -> &HostArchitecture {
        &self.architecture
    }

    pub fn facilities(&self) -> &BTreeSet<HostFacilityId> {
        &self.facilities
    }
}
