mod definition;
pub use definition::test_operations::raw::{
    ApiContract as OperationsContract, ApiService as OperationsService,
    primary_contract_id as operations_contract_id, primary_contract_key as operations_contract_key,
};
pub use definition::{
    TestOperations, TestOperationsConfig, operations_contract_version, operations_schema_version,
    operations_system_id,
};
