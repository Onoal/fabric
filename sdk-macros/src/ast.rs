use syn::{Block, Expr, Ident, LitStr, Path, Type, Visibility};

pub struct ResourceInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub resource_id: LitStr,
    pub schema: VersionLiteral,
    pub config_fields: Vec<ConfigField>,
    pub requires: Vec<RequirementDefinition>,
    pub contracts: Vec<ContractDefinition>,
    pub realization: Option<RealizationDefinition>,
    pub runtime_methods: Vec<RuntimeMethod>,
    pub runtime_state: Option<RuntimeStateDefinition>,
    pub lifecycle: RuntimeLifecycleDefinition,
}

pub struct SystemInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub system_id: LitStr,
    pub schema: VersionLiteral,
    pub config_fields: Vec<ConfigField>,
    pub systems: Vec<SystemDependencyDefinition>,
    pub contracts: Vec<ContractDefinition>,
    pub realization: Option<RealizationDefinition>,
    pub runtime_methods: Vec<RuntimeMethod>,
    pub runtime_state: Option<RuntimeStateDefinition>,
    pub lifecycle: RuntimeLifecycleDefinition,
}

pub struct AdapterInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub target_kind: AdapterTargetKind,
    pub target: Path,
    pub realization_interface: Path,
    /// An explicit, advanced target-schema compatibility override.  When it
    /// is absent, adapter support is derived exactly from the target's schema.
    pub schema: Option<RequirementLiteral>,
    pub realization: VersionLiteral,
    pub config_fields: Vec<ConfigField>,
    pub systems: Vec<SystemDependencyDefinition>,
    pub host_requirement: Option<Expr>,
    pub runtime_methods: Vec<RuntimeMethod>,
    pub runtime_state: Option<RuntimeStateDefinition>,
    pub lifecycle: RuntimeLifecycleDefinition,
}

pub struct ComponentInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub component_id: LitStr,
    pub config_fields: Vec<ConfigField>,
    pub requires: Vec<RequirementDefinition>,
    pub systems: Vec<SystemDependencyDefinition>,
    pub operations: Vec<ComponentOperationDefinition>,
    pub teardown: Option<Block>,
}

pub struct ConfigField {
    pub name: Ident,
    pub ty: Type,
}

pub struct ContractDefinition {
    pub is_primary: bool,
    pub name: Ident,
    pub contract_id: LitStr,
    pub version: VersionLiteral,
    pub methods: Vec<ContractMethod>,
}

pub struct ContractMethod {
    pub signature: syn::Signature,
}

pub struct RequirementDefinition {
    pub field: Ident,
    pub resource: Path,
    pub compatibility: RequirementLiteral,
}

pub struct SystemDependencyDefinition {
    pub field: Ident,
    pub system: Path,
    pub compatibility: RequirementLiteral,
}

pub struct RuntimeMethod {
    pub signature: syn::Signature,
    pub body: Block,
}

pub struct RuntimeStateDefinition {
    pub ty: Type,
    pub initializer: Expr,
}

#[derive(Default)]
pub struct RuntimeLifecycleDefinition {
    pub initialize: Option<Block>,
    pub start: Option<Block>,
    pub stop: Option<Block>,
    pub health: Option<Expr>,
}

pub struct ComponentOperationDefinition {
    pub name: Ident,
    pub operation_id: LitStr,
    pub input_ty: Type,
    pub input_type_id: LitStr,
    pub output_ty: Type,
    pub output_type_id: LitStr,
    pub context: ComponentOperationContext,
    pub handler: Expr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentOperationContext {
    None,
    Invocation,
}

pub struct RealizationDefinition {
    pub name: Ident,
    pub contract_id: LitStr,
    pub compatibility: RequirementLiteral,
    pub methods: Vec<ContractMethod>,
}

#[derive(Clone, Copy)]
pub enum AdapterTargetKind {
    Resource,
    System,
}

#[derive(Clone)]
pub enum VersionLiteral {
    Provisional,
    Versioned(LitStr),
}

pub enum RequirementLiteral {
    Provisional,
    Versioned(LitStr),
}
