use crate::{
    HostArchitecture, HostCompatibilityError, HostDescriptor, HostFacilityId, HostOperatingSystem,
    HostRequirement,
};

#[test]
fn host_requirement_evaluates_os_arch_and_facility_constraints() {
    let host = HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("x86_64").expect("arch"),
    )
    .with_facility(HostFacilityId::new("test.fast-runtime").expect("facility"));
    let requirement = HostRequirement::new()
        .allow_operating_system(HostOperatingSystem::new("linux").expect("os"))
        .allow_architecture(HostArchitecture::new("x86_64").expect("arch"))
        .require_facility(HostFacilityId::new("test.fast-runtime").expect("facility"));

    requirement.evaluate(&host).expect("compatible host");
}

#[test]
fn host_requirement_reports_structured_incompatibilities() {
    let host = HostDescriptor::new(
        HostOperatingSystem::new("macos").expect("os"),
        HostArchitecture::new("aarch64").expect("arch"),
    );
    let os_requirement = HostRequirement::new()
        .allow_operating_system(HostOperatingSystem::new("linux").expect("os"));
    match os_requirement.evaluate(&host) {
        Err(HostCompatibilityError::UnsupportedOperatingSystem { actual, allowed }) => {
            assert_eq!(actual.as_str(), "macos");
            assert_eq!(allowed[0].as_str(), "linux");
        }
        other => panic!("unexpected host os result: {other:?}"),
    }

    let arch_requirement =
        HostRequirement::new().allow_architecture(HostArchitecture::new("x86_64").expect("arch"));
    match arch_requirement.evaluate(&host) {
        Err(HostCompatibilityError::UnsupportedArchitecture { actual, allowed }) => {
            assert_eq!(actual.as_str(), "aarch64");
            assert_eq!(allowed[0].as_str(), "x86_64");
        }
        other => panic!("unexpected host architecture result: {other:?}"),
    }

    let facility_requirement = HostRequirement::new()
        .require_facility(HostFacilityId::new("test.fast-runtime").expect("facility"));
    match facility_requirement.evaluate(&host) {
        Err(HostCompatibilityError::MissingRequiredFacilities { missing }) => {
            assert_eq!(missing[0].as_str(), "test.fast-runtime");
        }
        other => panic!("unexpected host facility result: {other:?}"),
    }
}

#[test]
fn host_facilities_remain_open_to_unknown_third_party_requirements() {
    let facility =
        HostFacilityId::new("third-party.custom-accelerator").expect("third-party facility");
    let host = HostDescriptor::new(
        HostOperatingSystem::new("linux").expect("os"),
        HostArchitecture::new("aarch64").expect("arch"),
    )
    .with_facility(facility.clone());
    let requirement = HostRequirement::new().require_facility(facility);

    requirement.evaluate(&host).expect("open facility");
}

#[test]
fn native_host_descriptor_reports_os_arch_without_assuming_facilities() {
    let native = HostDescriptor::native();

    assert!(!native.operating_system().as_str().is_empty());
    assert!(!native.architecture().as_str().is_empty());
    assert!(native.facilities().is_empty());
}
