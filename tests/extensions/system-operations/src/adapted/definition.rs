use fabric_core::{ContractKey, ContractRequirement, ContractVersion, ContractVersionRequirement};
use fabric_system::{SystemId, SystemSchemaVersion};

type AdaptedOperationsRealizationContract = adapted_operations::realization::raw::AdapterContract;

fabric_sdk::system! {
    pub AdaptedOperations {
        id: "fabric.test.operations.adapted";

        schema: "2.0.0";

        config {}

        contracts {
            primary Api {
                id: "fabric.test.system.operations.adapted";
                version: "1.4.0";

                fn current_marker(&self) -> crate::OperationMarker;
                fn realization_provider(&self) -> String;
                fn realization_identity(&self) -> String;
            }
        }

        adapter Adapter {
            id: "fabric.test.system.operations.adapted.realization";
            compatibility: "^1";

            fn current_marker(&self) -> crate::OperationMarker;
        }

        runtime {
            fn current_marker(&self) -> crate::OperationMarker {
                self.adapter.current_marker()
            }

            fn realization_provider(&self) -> String {
                self.adapter.provider().as_str().to_owned()
            }

            fn realization_identity(&self) -> String {
                self.adapter.identity().to_string()
            }
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for AdaptedOperationsConfig {
    fn default() -> Self {
        Self {}
    }
}

pub fn adapted_operations_system_id() -> SystemId {
    adapted_operations::raw::system_id()
}

pub fn adapted_operations_schema_version() -> SystemSchemaVersion {
    SystemSchemaVersion::parse("2.0.0").expect("static adapted operations schema version")
}

pub fn adapted_operations_contract_version() -> ContractVersion {
    ContractVersion::parse("1.4.0").expect("static adapted operations contract version")
}

pub fn adapted_operations_realization_contract_version() -> ContractVersion {
    ContractVersion::parse("1.0.0").expect("static adapted operations realization version")
}

pub fn adapted_operations_realization_contract_key()
-> ContractKey<AdaptedOperationsRealizationContract> {
    adapted_operations::realization::raw::contract_key(
        adapted_operations_realization_contract_version(),
    )
}

pub fn adapted_operations_realization_requirement()
-> ContractRequirement<AdaptedOperationsRealizationContract> {
    ContractRequirement::versioned(
        crate::adapted_operations_realization_contract_id(),
        ContractVersionRequirement::parse("^1")
            .expect("static adapted operations realization requirement"),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundAdaptedOperationsRealization {
    pub provider: fabric_core::ModuleId,
    pub identity: fabric_core::ContractIdentity,
}
