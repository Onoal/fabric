use fabric::authoring::{PrimaryResourceContract, ResourceDefinition};
use fabric::prelude::{ResourceId, ResourceSelection};
use fabric::resource::ResourceSchemaDescriptor;
use fabric_core::{ContractId, ContractKey, ModuleRuntime};
use fabric_test_adapter_clock_memory::MemoryClock;
use fabric_test_resource_clock::ClockContract;

struct OtherClock;

impl PrimaryResourceContract for OtherClock {
    type Contract = ClockContract;

    fn primary_contract_key() -> ContractKey<Self::Contract> {
        ContractKey::provisional(ContractId::new("fabric.test.other.clock").expect("contract id"))
    }
}

impl ResourceDefinition for OtherClock {
    type Config = ();

    fn resource_id() -> ResourceId {
        ResourceId::new("fabric.test.other-clock").expect("resource id")
    }

    fn schema() -> ResourceSchemaDescriptor {
        ResourceSchemaDescriptor::provisional(Self::resource_id())
    }

    fn declaration(_selection: &ResourceSelection<Self>) -> fabric_core::ModuleDeclaration {
        unreachable!("not needed for compile-fail proof")
    }

    fn materialize(_selection: &ResourceSelection<Self>) -> Option<Box<dyn ModuleRuntime>> {
        panic!("not needed for compile-fail proof")
    }
}

fn main() {
    let _ = OtherClock::select("secondary", ())
        .expect("selection")
        .using(MemoryClock::default());
}
