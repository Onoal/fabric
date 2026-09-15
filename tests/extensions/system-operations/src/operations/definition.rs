use fabric_core::ContractVersion;
use fabric_system::{SystemId, SystemSchemaVersion};

use crate::OperationSequence;

fabric_sdk::system! {
    pub TestOperations {
        id: "fabric.test.operations";

        schema: "1.0.0";

        config {
            sequence: OperationSequence;
        }

        contracts {
            primary Api {
                id: "fabric.test.system.operations";
                version: "1.4.0";

                fn current_marker(&self) -> crate::OperationMarker;
            }
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                self.config.sequence.current_marker()
            }
        }
    }
}

impl TestOperationsConfig {
    pub fn new(seed: u64, step: u64) -> Self {
        Self {
            sequence: OperationSequence::new(seed, step),
        }
    }
}

pub fn operations_system_id() -> SystemId {
    test_operations::raw::system_id()
}

pub fn operations_schema_version() -> SystemSchemaVersion {
    SystemSchemaVersion::parse("1.0.0").expect("static operations schema version")
}

pub fn operations_contract_version() -> ContractVersion {
    ContractVersion::parse("1.4.0").expect("static operations contract version")
}
