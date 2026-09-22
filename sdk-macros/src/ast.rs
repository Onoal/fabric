use syn::{Block, Expr, Ident, LitStr, Path, Type, Visibility};

pub struct ResourceInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub resource_id: LitStr,
    pub schema: VersionLiteral,
    pub config: ConfigDefinition,
    pub relations: Vec<RelationDefinition>,
    pub api: ApiDefinition,
    pub realization: Option<RealizationDefinition>,
    /// Present only when this semantic Resource explicitly owns a live
    /// self-realization or mediation layer.
    pub runtime_methods: Option<Vec<RuntimeMethod>>,
    pub runtime_state: Option<RuntimeStateDefinition>,
    pub lifecycle: RuntimeLifecycleDefinition,
}

pub struct SystemInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub system_id: LitStr,
    pub schema: VersionLiteral,
    pub config: ConfigDefinition,
    pub relations: Vec<RelationDefinition>,
    pub api: ApiDefinition,
    pub realization: Option<RealizationDefinition>,
    /// Present only when this semantic System explicitly owns a live
    /// self-realization or mediation layer.
    pub runtime_methods: Option<Vec<RuntimeMethod>>,
    pub runtime_state: Option<RuntimeStateDefinition>,
    pub lifecycle: RuntimeLifecycleDefinition,
}

pub struct AdapterInput {
    pub visibility: Visibility,
    pub name: Ident,
    /// Legacy headers carry an explicit target kind. Canonical Adapter
    /// authoring derives the kind from the target and leaves this absent.
    pub target_kind: Option<AdapterTargetKind>,
    pub target: Path,
    /// The advanced, legacy realization interface. Canonical Adapter
    /// authoring derives the normal interface from the target semantic API.
    pub realization_interface: Option<Path>,
    /// An explicit, advanced target-schema compatibility override.  When it
    /// is absent, adapter support is derived exactly from the target's schema.
    pub schema: Option<RequirementLiteral>,
    pub realization: VersionLiteral,
    pub config: ConfigDefinition,
    pub relations: Vec<RelationDefinition>,
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

/// The creator-owned, typed configuration contract for a definition.
///
/// `None` is intentionally distinct from an inline declaration with zero
/// fields: the former has no author-facing configuration API, while the latter
/// remains accepted as an explicit legacy declaration.
pub enum ConfigDefinition {
    None,
    Inline(Vec<ConfigField>),
    Type(Type),
}

/// The single semantic API a Resource or System exposes to its consumers.
///
/// Normal `api { ... }` authoring derives identity and version from the owning
/// definition. The explicit form exists only while the legacy `contracts`
/// grammar remains source compatible.
pub struct ApiDefinition {
    pub name: Ident,
    pub identity: ApiIdentity,
    pub version: VersionLiteral,
    pub methods: Vec<ContractMethod>,
}

pub enum ApiIdentity {
    OwnerDerived,
    LegacyExplicit(LitStr),
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

/// A named, typed capability relation declared by a definition.
///
/// The target kind is deliberately not represented here: the generated SDK
/// code obtains the target's primary semantic contract through RelationTarget.
pub struct RelationDefinition {
    pub field: Ident,
    pub target: Path,
    pub compatibility: Option<RequirementLiteral>,
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

impl RuntimeLifecycleDefinition {
    pub fn is_empty(&self) -> bool {
        self.initialize.is_none()
            && self.start.is_none()
            && self.stop.is_none()
            && self.health.is_none()
    }
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
