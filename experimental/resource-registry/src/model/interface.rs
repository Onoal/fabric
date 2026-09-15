use std::fmt;

use fabric_core::ContractId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ResourceInterfaceRole {
    Provides,
    Requires,
    Optional,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceInterface {
    contract_id: ContractId,
    role: ResourceInterfaceRole,
}

impl ResourceInterface {
    pub fn provided(contract_id: ContractId) -> Self {
        Self {
            contract_id,
            role: ResourceInterfaceRole::Provides,
        }
    }

    pub fn required(contract_id: ContractId) -> Self {
        Self {
            contract_id,
            role: ResourceInterfaceRole::Requires,
        }
    }

    pub fn optional(contract_id: ContractId) -> Self {
        Self {
            contract_id,
            role: ResourceInterfaceRole::Optional,
        }
    }

    pub fn contract_id(&self) -> &ContractId {
        &self.contract_id
    }

    pub fn role(&self) -> ResourceInterfaceRole {
        self.role
    }
}

impl fmt::Display for ResourceInterfaceRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Provides => "provides",
            Self::Requires => "requires",
            Self::Optional => "optional",
        };
        f.write_str(value)
    }
}
