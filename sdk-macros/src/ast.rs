use syn::{Block, Expr, Ident, LitStr, Path, Type, Visibility};

pub struct ResourceInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub resource_id: LitStr,
    pub schema: VersionLiteral,
    pub config: ConfigDefinition,
    pub relations: Vec<RelationDefinition>,
    pub api: ApiDefinition,
    /// Optional delta from the public API to the Adapter's effective
    /// realization contract.
    pub differential_realization: Option<DifferentialRealizationDefinition>,
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
    pub differential_realization: Option<DifferentialRealizationDefinition>,
    /// Present only when this semantic System explicitly owns a live
    /// self-realization or mediation layer.
    pub runtime_methods: Option<Vec<RuntimeMethod>>,
    pub runtime_state: Option<RuntimeStateDefinition>,
    pub lifecycle: RuntimeLifecycleDefinition,
}

pub struct AdapterInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub target: Path,
    /// An explicit, advanced target-schema compatibility override.  When it
    /// is absent, adapter support is derived exactly from the target's schema.
    pub schema: Option<RequirementLiteral>,
    pub config: ConfigDefinition,
    pub relations: Vec<RelationDefinition>,
    pub host_requirement: Option<Expr>,
    pub runtime_methods: Vec<RuntimeMethod>,
    pub runtime_state: Option<RuntimeStateDefinition>,
    /// Participation-owned hooks used only when the resolved target is a
    /// Component. Resource/System provider lifecycle remains separate.
    pub component_prepare: Option<Block>,
    pub component_teardown: Option<Block>,
    pub lifecycle: RuntimeLifecycleDefinition,
}

pub struct ComponentInput {
    pub visibility: Visibility,
    pub name: Ident,
    pub component_id: LitStr,
    pub config: ConfigDefinition,
    /// The canonical declaration-time requirements.  Their target kind is
    /// intentionally absent: `RelationTarget` owns that lowering detail.
    pub relations: Vec<RelationDefinition>,
    /// Canonical Component callable declaration. It has no handler or
    /// realization attachment.
    pub api: Option<ApiDefinition>,
    /// Canonical self-realization authoring. Unlike the legacy operations
    /// frontend, this is an implementation of the already-declared API.
    pub runtime: Option<ComponentParticipationRealization>,
    /// Transitional 0.5.0 self-realizing frontend.  It remains isolated so
    /// canonical declaration lowering never depends on it.
    pub legacy_requires: Vec<RequirementDefinition>,
    pub legacy_systems: Vec<SystemDependencyDefinition>,
    pub legacy_operations: Option<Vec<ComponentOperationDefinition>>,
    pub teardown: Option<Block>,
}

pub struct ComponentParticipationRealization {
    pub methods: Vec<RuntimeMethod>,
    pub state: Option<RuntimeStateDefinition>,
    pub prepare: Option<Block>,
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
/// definition.
pub struct ApiDefinition {
    pub name: Ident,
    pub version: VersionLiteral,
    pub methods: Vec<ContractMethod>,
}

#[derive(Clone)]
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

/// The authored delta used when a semantic owner mediates part of its API.
/// Unlisted API methods remain direct Adapter requirements.
pub struct DifferentialRealizationDefinition {
    pub mediated: Vec<Ident>,
    pub operations: Vec<ContractMethod>,
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
