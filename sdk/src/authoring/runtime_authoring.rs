use std::sync::Arc;

use fabric_core::{
    ContractKey, Health, InstanceGeneration, InstanceId, InstanceRuntimeContext, ModuleBindings,
    ModuleContract, ModuleDeclaration, ModuleError, ModuleId, ModuleRuntime,
};
use fabric_host::HostRequirement;

use super::AdapterDefinition;

/// Shared, occurrence-local state for normal Fabric runtime authoring.
///
/// A value is created once for each materialized runtime occurrence. Cloning
/// this handle shares that occurrence's state; rematerializing creates a new
/// state value.
pub struct RuntimeState<S>(Arc<S>);

impl<S> Clone for RuntimeState<S> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<S> RuntimeState<S> {
    pub fn new(state: S) -> Self {
        Self(Arc::new(state))
    }

    pub fn get(&self) -> &S {
        &self.0
    }
}

/// Generation-scoped Instance context made available after Core binds a
/// materialized runtime. It is intentionally absent during early cleanup.
#[derive(Clone, Default)]
pub struct RuntimeContext {
    context: Option<InstanceRuntimeContext>,
}

impl RuntimeContext {
    pub fn instance_id(&self) -> Option<&InstanceId> {
        self.context
            .as_ref()
            .map(InstanceRuntimeContext::instance_id)
    }

    pub fn generation(&self) -> Option<InstanceGeneration> {
        self.context
            .as_ref()
            .map(InstanceRuntimeContext::generation)
    }

    pub fn is_bound(&self) -> bool {
        self.context.is_some()
    }

    #[doc(hidden)]
    pub fn bind(&mut self, context: InstanceRuntimeContext) {
        self.context = Some(context);
    }
}

type RuntimeHook<S> =
    Arc<dyn Fn(&RuntimeState<S>, &RuntimeContext) -> Result<(), ModuleError> + Send + Sync>;
type RuntimeHealth<S> = Arc<dyn Fn(&RuntimeState<S>, &RuntimeContext) -> Health + Send + Sync>;

/// Public SDK machinery for a stateful, lifecycle-aware provider runtime.
///
/// This is runtime construction machinery, not a Fabric semantic subject. It
/// bridges normal authoring closures to Core's advanced `ModuleRuntime`
/// protocol so package authors do not need to implement that protocol.
pub struct StatefulRuntimeAuthoring<S, Contract> {
    state: Arc<dyn Fn() -> S + Send + Sync>,
    service: Arc<dyn Fn(RuntimeState<S>) -> Contract + Send + Sync>,
    initialize: RuntimeHook<S>,
    start: RuntimeHook<S>,
    stop: RuntimeHook<S>,
    health: RuntimeHealth<S>,
}

impl<S, Contract> Clone for StatefulRuntimeAuthoring<S, Contract> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            service: Arc::clone(&self.service),
            initialize: Arc::clone(&self.initialize),
            start: Arc::clone(&self.start),
            stop: Arc::clone(&self.stop),
            health: Arc::clone(&self.health),
        }
    }
}

impl<S, Contract> StatefulRuntimeAuthoring<S, Contract>
where
    S: Send + Sync + 'static,
    Contract: Send + Sync + 'static,
{
    pub fn new(
        state: impl Fn() -> S + Send + Sync + 'static,
        service: impl Fn(RuntimeState<S>) -> Contract + Send + Sync + 'static,
    ) -> Self {
        Self {
            state: Arc::new(state),
            service: Arc::new(service),
            initialize: Arc::new(|_, _| Ok(())),
            start: Arc::new(|_, _| Ok(())),
            stop: Arc::new(|_, _| Ok(())),
            health: Arc::new(|_, _| Health::Healthy),
        }
    }

    pub fn with_initialize(
        mut self,
        hook: impl Fn(&RuntimeState<S>, &RuntimeContext) -> Result<(), ModuleError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.initialize = Arc::new(hook);
        self
    }

    pub fn with_start(
        mut self,
        hook: impl Fn(&RuntimeState<S>, &RuntimeContext) -> Result<(), ModuleError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.start = Arc::new(hook);
        self
    }

    pub fn with_stop(
        mut self,
        hook: impl Fn(&RuntimeState<S>, &RuntimeContext) -> Result<(), ModuleError>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.stop = Arc::new(hook);
        self
    }

    pub fn with_health(
        mut self,
        hook: impl Fn(&RuntimeState<S>, &RuntimeContext) -> Health + Send + Sync + 'static,
    ) -> Self {
        self.health = Arc::new(hook);
        self
    }

    pub fn declaration(
        &self,
        module_id: ModuleId,
        contract: ContractKey<Contract>,
    ) -> ModuleDeclaration {
        ModuleDeclaration::new(module_id).with_provided_contracts(vec![contract.declaration()])
    }

    /// Materializes a fresh runtime occurrence and its fresh state.
    pub fn materialize(
        &self,
        module_id: ModuleId,
        contract: ContractKey<Contract>,
    ) -> Box<dyn ModuleRuntime> {
        let state = RuntimeState::new((self.state)());
        let service = Arc::new((self.service)(state.clone()));
        Box::new(StatefulRuntimeBridge {
            module_id,
            contract,
            state,
            service,
            context: RuntimeContext::default(),
            initialize: Arc::clone(&self.initialize),
            start: Arc::clone(&self.start),
            stop: Arc::clone(&self.stop),
            health: Arc::clone(&self.health),
        })
    }
}

/// A normal SDK Adapter definition backed by stateful runtime authoring.
///
/// This is construction machinery: it carries compatibility, Host
/// requirements, contract declaration, and runtime hooks into the existing
/// Adapter lowering path. It does not introduce a semantic lifecycle subject.
pub struct StatefulAdapterDefinition<Target, Compatibility, State, Contract>
where
    Contract: Send + Sync + 'static,
{
    compatibility: Compatibility,
    host_requirement: HostRequirement,
    contract: ContractKey<Contract>,
    runtime: StatefulRuntimeAuthoring<State, Contract>,
    target: std::marker::PhantomData<Target>,
}

impl<Target, Compatibility, State, Contract> Clone
    for StatefulAdapterDefinition<Target, Compatibility, State, Contract>
where
    Compatibility: Clone,
    Contract: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            compatibility: self.compatibility.clone(),
            host_requirement: self.host_requirement.clone(),
            contract: self.contract.clone(),
            runtime: self.runtime.clone(),
            target: std::marker::PhantomData,
        }
    }
}

impl<Target, Compatibility, State, Contract>
    StatefulAdapterDefinition<Target, Compatibility, State, Contract>
where
    Compatibility: Clone + Send + Sync + 'static,
    State: Send + Sync + 'static,
    Contract: Clone + Send + Sync + 'static,
{
    pub fn new(
        compatibility: Compatibility,
        host_requirement: HostRequirement,
        contract: ContractKey<Contract>,
        runtime: StatefulRuntimeAuthoring<State, Contract>,
    ) -> Self {
        Self {
            compatibility,
            host_requirement,
            contract,
            runtime,
            target: std::marker::PhantomData,
        }
    }
}

impl<Target, Compatibility, State, Contract> AdapterDefinition
    for StatefulAdapterDefinition<Target, Compatibility, State, Contract>
where
    Target: Send + Sync + 'static,
    Compatibility: Clone + Send + Sync + 'static,
    State: Send + Sync + 'static,
    Contract: Clone + Send + Sync + 'static,
{
    type Target = Target;
    type Compatibility = Compatibility;

    fn compatibility(&self) -> Self::Compatibility {
        self.compatibility.clone()
    }

    fn host_requirement(&self) -> HostRequirement {
        self.host_requirement.clone()
    }

    fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
        self.runtime
            .declaration(provider_module_id, self.contract.clone())
    }

    fn materialize_provider(&self, provider_module_id: ModuleId) -> Option<Box<dyn ModuleRuntime>> {
        Some(
            self.runtime
                .materialize(provider_module_id, self.contract.clone()),
        )
    }
}

struct StatefulRuntimeBridge<S, Contract>
where
    Contract: Send + Sync + 'static,
{
    module_id: ModuleId,
    contract: ContractKey<Contract>,
    state: RuntimeState<S>,
    service: Arc<Contract>,
    context: RuntimeContext,
    initialize: RuntimeHook<S>,
    start: RuntimeHook<S>,
    stop: RuntimeHook<S>,
    health: RuntimeHealth<S>,
}

impl<S, Contract> ModuleRuntime for StatefulRuntimeBridge<S, Contract>
where
    S: Send + Sync + 'static,
    Contract: Send + Sync + 'static,
{
    fn id(&self) -> &ModuleId {
        &self.module_id
    }

    fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
        vec![self.contract.declaration()]
    }

    fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
        Ok(vec![ModuleContract::new(
            &self.contract,
            Arc::clone(&self.service),
        )])
    }

    fn bind(&mut self, _bindings: &ModuleBindings) -> Result<(), ModuleError> {
        Ok(())
    }

    fn bind_instance_context(
        &mut self,
        context: &InstanceRuntimeContext,
    ) -> Result<(), ModuleError> {
        self.context.bind(context.clone());
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), ModuleError> {
        (self.initialize)(&self.state, &self.context)
    }

    fn start(&mut self) -> Result<(), ModuleError> {
        (self.start)(&self.state, &self.context)
    }

    fn stop(&mut self) -> Result<(), ModuleError> {
        (self.stop)(&self.state, &self.context)
    }

    fn health(&self) -> Health {
        (self.health)(&self.state, &self.context)
    }
}
