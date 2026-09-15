use fabric::prelude::*;

#[test]
fn sdk_exposes_generic_host_compatibility_with_open_facilities() {
    let host = HostDescriptor::native()
        .with_facility(HostFacilityId::new("third-party.custom-runtime").expect("facility"));
    let requirement = HostRequirement::new()
        .require_facility(HostFacilityId::new("third-party.custom-runtime").expect("facility"));

    requirement
        .evaluate(&host)
        .expect("host with third-party facility should be compatible");
}
