use fabric_core::{
    BlockBuilder, BlockId, Composition, CompositionBuilder, CompositionError, CompositionId,
    HostMaterializationRequirement, InstanceId, Module, ModuleBindings, ModuleContract,
    ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_host::{
    HostArchitecture, HostCompatibilityError, HostDescriptor, HostFacilityId, HostOperatingSystem,
    HostRequirement,
};

#[derive(Clone)]
struct PlainModuleRuntime {
    module_id: ModuleId,
}

impl PlainModuleRuntime {
    fn new(module_id: &str) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
        }
    }
}

impl ModuleRuntime for PlainModuleRuntime {
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(Vec::new())
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        Ok(())
    }

    fn stop(&mut self) {}

    fn health(&self) -> fabric_core::Health {
        fabric_core::Health::Healthy
    }
}

#[derive(Clone)]
struct HostDeclaredModule {
    module_id: ModuleId,
    requirement: HostRequirement,
}

impl HostDeclaredModule {
    fn new(module_id: &str, requirement: HostRequirement) -> Self {
        Self {
            module_id: ModuleId::new(module_id).expect("module id"),
            requirement,
        }
    }
}

impl Module for HostDeclaredModule {
    fn declaration(&self) -> ModuleDeclaration {
        ModuleDeclaration::new(self.module_id.clone()).with_host_requirement(
            HostMaterializationRequirement::new(self.module_id.clone(), self.requirement.clone()),
        )
    }

    fn materialize(&self) -> Option<Box<dyn ModuleRuntime>> {
        Some(Box::new(PlainModuleRuntime {
            module_id: self.module_id.clone(),
        }))
    }
}

fn build_composition(module: impl Module + 'static) -> Composition {
    CompositionBuilder::new(
        CompositionId::new("fabric.test.raw.host-materialization".to_owned())
            .expect("composition id"),
    )
    .register_block(
        BlockBuilder::new(BlockId::new("runtime".to_owned()).expect("block id"))
            .register_module(module)
            .build(),
    )
    .build()
    .expect("composition")
}

fn host(os: &str, arch: &str) -> HostDescriptor {
    HostDescriptor::new(
        HostOperatingSystem::new(os).expect("host os"),
        HostArchitecture::new(arch).expect("host arch"),
    )
}

fn host_with_facility(os: &str, arch: &str, facility: HostFacilityId) -> HostDescriptor {
    host(os, arch).with_facility(facility)
}

#[test]
fn host_free_composition_still_materializes_without_explicit_host() {
    let composition = build_composition(PlainModuleRuntime::new("plain.module"));
    let mut instance = composition
        .materialize(InstanceId::new("fabric.test.raw.host-free".to_owned()).expect("instance"))
        .expect("host-free materialization");

    instance.start().expect("start");
    instance.stop();
}

#[test]
fn host_constrained_module_requires_explicit_host_descriptor() {
    let module_id = ModuleId::new("host.bound.module".to_owned()).expect("module id");
    let composition = build_composition(HostDeclaredModule {
        module_id: module_id.clone(),
        requirement: HostRequirement::new(),
    });

    let error = composition
        .materialize(InstanceId::new("fabric.test.raw.host-required".to_owned()).expect("instance"))
        .expect_err("host-constrained module should require explicit host");

    assert!(matches!(
        error,
        CompositionError::HostDescriptorRequired { module_ids }
            if module_ids == vec![module_id]
    ));
}

#[test]
fn materialize_on_accepts_compatible_host() {
    let facility = HostFacilityId::new("fabric.test.host.clock".to_owned()).expect("facility");
    let composition = build_composition(HostDeclaredModule::new(
        "host.bound.module",
        HostRequirement::new()
            .allow_operating_system(HostOperatingSystem::new("linux").expect("os"))
            .allow_architecture(HostArchitecture::new("x86_64").expect("arch"))
            .require_facility(facility.clone()),
    ));

    let mut instance = composition
        .materialize_on(
            InstanceId::new("fabric.test.raw.host-compatible".to_owned()).expect("instance"),
            &host_with_facility("linux", "x86_64", facility),
        )
        .expect("compatible host");

    instance.start().expect("start");
    instance.stop();
}

#[test]
fn materialize_on_reports_operating_system_mismatch_structurally() {
    let module_id = ModuleId::new("host.os.module".to_owned()).expect("module id");
    let composition = build_composition(HostDeclaredModule::new(
        module_id.as_str(),
        HostRequirement::new()
            .allow_operating_system(HostOperatingSystem::new("linux").expect("os")),
    ));

    let error = composition
        .materialize_on(
            InstanceId::new("fabric.test.raw.host-os-mismatch".to_owned()).expect("instance"),
            &host("darwin", "x86_64"),
        )
        .expect_err("unsupported os should fail");

    assert!(matches!(
        error,
        CompositionError::HostIncompatible {
            module_id: actual_module_id,
            source: HostCompatibilityError::UnsupportedOperatingSystem { actual, allowed },
        } if actual_module_id == module_id
            && actual.as_str() == "darwin"
            && allowed
                == vec![HostOperatingSystem::new("linux").expect("os")]
    ));
}

#[test]
fn materialize_on_reports_architecture_mismatch_structurally() {
    let module_id = ModuleId::new("host.arch.module".to_owned()).expect("module id");
    let composition = build_composition(HostDeclaredModule::new(
        module_id.as_str(),
        HostRequirement::new().allow_architecture(HostArchitecture::new("aarch64").expect("arch")),
    ));

    let error = composition
        .materialize_on(
            InstanceId::new("fabric.test.raw.host-arch-mismatch".to_owned()).expect("instance"),
            &host("linux", "x86_64"),
        )
        .expect_err("unsupported architecture should fail");

    assert!(matches!(
        error,
        CompositionError::HostIncompatible {
            module_id: actual_module_id,
            source: HostCompatibilityError::UnsupportedArchitecture { actual, allowed },
        } if actual_module_id == module_id
            && actual.as_str() == "x86_64"
            && allowed
                == vec![HostArchitecture::new("aarch64").expect("arch")]
    ));
}

#[test]
fn materialize_on_reports_missing_facility_structurally() {
    let module_id = ModuleId::new("host.facility.module".to_owned()).expect("module id");
    let required_facility =
        HostFacilityId::new("fabric.test.host.required".to_owned()).expect("facility");
    let composition = build_composition(HostDeclaredModule::new(
        module_id.as_str(),
        HostRequirement::new().require_facility(required_facility.clone()),
    ));

    let error = composition
        .materialize_on(
            InstanceId::new("fabric.test.raw.host-facility-mismatch".to_owned()).expect("instance"),
            &host("linux", "x86_64"),
        )
        .expect_err("missing facility should fail");

    assert!(matches!(
        error,
        CompositionError::HostIncompatible {
            module_id: actual_module_id,
            source: HostCompatibilityError::MissingRequiredFacilities { missing },
        } if actual_module_id == module_id && missing == vec![required_facility]
    ));
}

#[test]
fn same_portable_composition_can_succeed_or_fail_on_different_hosts() {
    let required_facility =
        HostFacilityId::new("fabric.test.host.portable".to_owned()).expect("facility");
    let composition = build_composition(HostDeclaredModule::new(
        "portable.module",
        HostRequirement::new().require_facility(required_facility.clone()),
    ));

    let incompatible = composition.materialize_on(
        InstanceId::new("fabric.test.raw.portable-fail".to_owned()).expect("instance"),
        &host("linux", "x86_64"),
    );
    assert!(matches!(
        incompatible,
        Err(CompositionError::HostIncompatible { .. })
    ));

    let mut compatible = composition
        .materialize_on(
            InstanceId::new("fabric.test.raw.portable-pass".to_owned()).expect("instance"),
            &host_with_facility("linux", "x86_64", required_facility),
        )
        .expect("compatible host should materialize");
    compatible.start().expect("start");
    compatible.stop();
}
