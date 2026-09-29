use fabric_component::ComponentHostHandle;
use fabric_core::{CompositionError, CompositionExport, CompositionId, ModuleId};
use fabric_host::HostDescriptor;

use crate::composition::{
    ComponentInspection, Composition, FabricManifest, ResourceInspection,
    SemanticRelationBindingManifestEntry, SystemInspection,
};
use crate::ids::IntoInstanceId;
use crate::instance::{Instance, InstanceComponents};

const DEFAULT_MATERIALIZATION_PROFILE: &str = "default";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterializationProfileName(String);

impl MaterializationProfileName {
    pub fn new(value: impl Into<String>) -> Result<Self, CompositionError> {
        validate_profile_name(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MaterializationProfileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializationProfile {
    name: MaterializationProfileName,
}

impl MaterializationProfile {
    pub fn new(name: impl Into<String>) -> Result<Self, CompositionError> {
        Ok(Self {
            name: MaterializationProfileName::new(name)?,
        })
    }

    pub fn default_profile() -> Self {
        Self {
            name: MaterializationProfileName(DEFAULT_MATERIALIZATION_PROFILE.to_owned()),
        }
    }

    pub fn name(&self) -> &MaterializationProfileName {
        &self.name
    }
}

impl Default for MaterializationProfile {
    fn default() -> Self {
        Self::default_profile()
    }
}

fn validate_profile_name(value: String) -> Result<String, CompositionError> {
    if value.trim() != value || value.is_empty() {
        return Err(CompositionError::InvalidIdentifier {
            kind: "MaterializationProfileName",
            value,
        });
    }
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        Ok(value)
    } else {
        Err(CompositionError::InvalidIdentifier {
            kind: "MaterializationProfileName",
            value,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializationPlanProvenance {
    composition_id: CompositionId,
    profile: MaterializationProfile,
    host: Option<HostDescriptor>,
}

impl MaterializationPlanProvenance {
    pub(crate) fn new(
        composition_id: CompositionId,
        profile: MaterializationProfile,
        host: Option<HostDescriptor>,
    ) -> Self {
        Self {
            composition_id,
            profile,
            host,
        }
    }

    pub fn composition_id(&self) -> &CompositionId {
        &self.composition_id
    }

    pub fn materialization_profile(&self) -> &MaterializationProfile {
        &self.profile
    }

    pub fn host(&self) -> Option<&HostDescriptor> {
        self.host.as_ref()
    }
}

/// Frozen non-live materialization truth prepared from a Composition, Profile,
/// and Host context before an Instance generation exists.
pub struct MaterializationPlan<'a> {
    composition: &'a Composition,
    provenance: MaterializationPlanProvenance,
}

impl std::fmt::Debug for MaterializationPlan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaterializationPlan")
            .field("composition_id", self.composition_id())
            .field("materialization_profile", self.materialization_profile())
            .field("host", &self.host())
            .finish()
    }
}

impl<'a> MaterializationPlan<'a> {
    pub(crate) fn new(
        composition: &'a Composition,
        profile: MaterializationProfile,
        host: Option<HostDescriptor>,
    ) -> Result<Self, CompositionError> {
        validate_host_context(composition.manifest(), host.as_ref())?;
        Ok(Self {
            composition,
            provenance: MaterializationPlanProvenance::new(composition.id().clone(), profile, host),
        })
    }

    pub fn composition_id(&self) -> &CompositionId {
        self.provenance.composition_id()
    }

    pub fn materialization_profile(&self) -> &MaterializationProfile {
        self.provenance.materialization_profile()
    }

    pub fn host(&self) -> Option<&HostDescriptor> {
        self.provenance.host()
    }

    pub fn provenance(&self) -> &MaterializationPlanProvenance {
        &self.provenance
    }

    pub fn resources(&self) -> impl Iterator<Item = ResourceInspection<'_>> + '_ {
        self.composition.resources()
    }

    pub fn systems(&self) -> impl Iterator<Item = SystemInspection<'_>> + '_ {
        self.composition.systems()
    }

    pub fn components(&self) -> impl Iterator<Item = ComponentInspection<'_>> + '_ {
        self.composition.components()
    }

    pub fn relations(&self) -> &[SemanticRelationBindingManifestEntry] {
        self.composition.relations()
    }

    pub fn materialize<I>(&self, instance_id: I) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        let instance_id = instance_id.into_instance_id()?;
        let core = match self.host() {
            Some(host) => self.composition.core().materialize_on(instance_id, host)?,
            None => self.composition.core().materialize(instance_id)?,
        };
        let components = component_host(self.composition.component_host_export(), &core);
        Ok(Instance::from_materialization(
            core,
            self.composition.semantic_context(),
            self.provenance.clone(),
            components,
        ))
    }

    pub fn materialize_with_input<I, T>(
        &self,
        instance_id: I,
        input: &T,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
        T: Send + Sync + 'static,
    {
        let instance_id = instance_id.into_instance_id()?;
        let core = match self.host() {
            Some(host) => {
                self.composition
                    .core()
                    .materialize_on_with_input(instance_id, host, input)?
            }
            None => self
                .composition
                .core()
                .materialize_with_input(instance_id, input)?,
        };
        let components = component_host(self.composition.component_host_export(), &core);
        Ok(Instance::from_materialization(
            core,
            self.composition.semantic_context(),
            self.provenance.clone(),
            components,
        ))
    }
}

impl Composition {
    pub fn plan(&self) -> Result<MaterializationPlan<'_>, CompositionError> {
        self.plan_with_profile(&MaterializationProfile::default_profile())
    }

    pub fn plan_on(
        &self,
        host: &HostDescriptor,
    ) -> Result<MaterializationPlan<'_>, CompositionError> {
        self.plan_with_profile_on(&MaterializationProfile::default_profile(), host)
    }

    pub fn plan_with_profile(
        &self,
        profile: &MaterializationProfile,
    ) -> Result<MaterializationPlan<'_>, CompositionError> {
        MaterializationPlan::new(self, profile.clone(), None)
    }

    pub fn plan_with_profile_on(
        &self,
        profile: &MaterializationProfile,
        host: &HostDescriptor,
    ) -> Result<MaterializationPlan<'_>, CompositionError> {
        MaterializationPlan::new(self, profile.clone(), Some(host.clone()))
    }

    pub fn materialize<I>(&self, instance_id: I) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.plan()?.materialize(instance_id)
    }

    pub fn materialize_with_input<I, T>(
        &self,
        instance_id: I,
        input: &T,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
        T: Send + Sync + 'static,
    {
        self.plan()?.materialize_with_input(instance_id, input)
    }

    pub fn materialize_on<I>(
        &self,
        instance_id: I,
        host: &HostDescriptor,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.plan_on(host)?.materialize(instance_id)
    }

    pub fn materialize_on_with_input<I, T>(
        &self,
        instance_id: I,
        host: &HostDescriptor,
        input: &T,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
        T: Send + Sync + 'static,
    {
        self.plan_on(host)?
            .materialize_with_input(instance_id, input)
    }

    pub fn materialize_with_profile<I>(
        &self,
        instance_id: I,
        profile: &MaterializationProfile,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.plan_with_profile(profile)?.materialize(instance_id)
    }

    pub fn materialize_with_profile_on<I>(
        &self,
        instance_id: I,
        profile: &MaterializationProfile,
        host: &HostDescriptor,
    ) -> Result<Instance, CompositionError>
    where
        I: IntoInstanceId,
    {
        self.plan_with_profile_on(profile, host)?
            .materialize(instance_id)
    }
}

fn component_host(
    export: Option<&CompositionExport<ComponentHostHandle>>,
    core: &fabric_core::Instance,
) -> Option<InstanceComponents> {
    export.map(|export| InstanceComponents {
        handle: core
            .export(export)
            .expect("Composition component export must be retained by its Instance"),
    })
}

fn validate_host_context(
    manifest: &FabricManifest,
    host: Option<&HostDescriptor>,
) -> Result<(), CompositionError> {
    let mut requirements = manifest
        .diagnostics()
        .module_declarations()
        .iter()
        .filter_map(|declaration| declaration.host_requirement())
        .collect::<Vec<_>>();
    requirements.sort_by(|left, right| left.module_id().cmp(right.module_id()));
    match host {
        Some(host) => {
            for requirement in requirements {
                requirement.requirement().evaluate(host).map_err(|source| {
                    CompositionError::HostIncompatible {
                        module_id: requirement.module_id().clone(),
                        source,
                    }
                })?;
            }
            Ok(())
        }
        None if requirements.is_empty() => Ok(()),
        None => Err(CompositionError::HostDescriptorRequired {
            module_ids: requirements
                .into_iter()
                .map(|requirement| requirement.module_id().clone())
                .collect::<Vec<ModuleId>>(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Poll, Waker};

    use fabric_component::ComponentRelationName;
    use fabric_core::{
        BlockBuilder, CompositionBuilder, CompositionError, Health, Module, ModuleBindings,
        ModuleContract, ModuleDeclaration, ModuleError, ModuleId, ModuleMaterializationContext,
        ModuleRuntime,
    };
    use fabric_host::HostDescriptor;
    use futures::executor::block_on;

    use crate::authoring::{
        AdapterBridgeMode, AdapterDefinition, CanonicalAdapterSupport,
        ComponentResourceRequirement, Requires,
    };
    use crate::ids::{block, composition, instance, module};
    use crate::resource::ResourceFuture;
    use crate::{Fabric, HostFacilityId, HostRequirement};

    #[derive(Clone)]
    struct TestEnvironment {
        prefix: String,
    }

    struct WrongEnvironment;

    #[derive(Clone)]
    struct LiveValueAdapter {
        suffix: String,
        cleanup: Option<Arc<AtomicUsize>>,
        host_requirement: HostRequirement,
    }

    #[derive(Clone)]
    struct LiveValueRuntime {
        module_id: ModuleId,
        value: String,
        cleanup: Option<Arc<AtomicUsize>>,
    }

    crate::resource! {
        LiveValue {
            id: "fabric.test.live-materialization.value";

            api {
                async fn read(&self) -> String;
            }
        }
    }

    crate::adapter! {
        StaticLiveValueAdapter for LiveValue {
            id: "fabric.test.live-materialization.static-adapter";

            runtime {
                async fn read(&self) -> String {
                    "static".to_owned()
                }
            }
        }
    }

    crate::component! {
        LiveValueConsumer {
            id: "fabric.test.live-materialization.consumer";

            relations {
                requires {
                    value: LiveValue;
                }
            }

            api {
                fn read(&self) -> String;
            }

            runtime {
                fn read(&self) -> String {
                    expect_resource_ready(self.relations().value.read())
                }
            }
        }
    }

    fn expect_resource_ready<T>(mut future: ResourceFuture<'_, T>) -> T {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        match Future::poll(future.as_mut(), &mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("test Resource operation unexpectedly required async progress"),
        }
    }

    impl LiveValueAdapter {
        fn new(suffix: impl Into<String>) -> Self {
            Self {
                suffix: suffix.into(),
                cleanup: None,
                host_requirement: HostRequirement::new(),
            }
        }

        fn with_cleanup(mut self, cleanup: Arc<AtomicUsize>) -> Self {
            self.cleanup = Some(cleanup);
            self
        }

        fn with_host_requirement(mut self, host_requirement: HostRequirement) -> Self {
            self.host_requirement = host_requirement;
            self
        }
    }

    impl LiveValueRuntime {
        fn read<'a>(&'a self) -> ResourceFuture<'a, String> {
            Box::pin(async move { self.value.clone() })
        }
    }

    impl ModuleRuntime for LiveValueRuntime {
        fn id(&self) -> &ModuleId {
            &self.module_id
        }

        fn provided_contract_declarations(&self) -> Vec<fabric_core::ProvidedContractDeclaration> {
            vec![LiveValue::__fabric_canonical_adapter_contract_key().declaration()]
        }

        fn export_contracts(&self) -> Result<Vec<ModuleContract>, ModuleError> {
            let contract = LiveValue::__fabric_canonical_adapter_builder()
                .read(Self::read)
                .build(Arc::new(self.clone()));
            Ok(vec![ModuleContract::new(
                &LiveValue::__fabric_canonical_adapter_contract_key(),
                Arc::new(contract),
            )])
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

        fn stop(&mut self) -> Result<(), ModuleError> {
            if let Some(cleanup) = &self.cleanup {
                cleanup.fetch_add(1, Ordering::SeqCst);
            }
            Ok(())
        }

        fn health(&self) -> Health {
            Health::Healthy
        }
    }

    impl AdapterDefinition for LiveValueAdapter {
        type Target = LiveValue;
        type Compatibility = CanonicalAdapterSupport<LiveValue>;

        fn adapter_definition_id(&self) -> crate::authoring::AdapterDefinitionId {
            crate::authoring::AdapterDefinitionId::new("fabric.test.live-materialization.adapter")
                .expect("adapter id")
        }

        fn compatibility(&self) -> Self::Compatibility {
            CanonicalAdapterSupport::<LiveValue>::inferred()
        }

        fn bridge_mode(&self) -> AdapterBridgeMode {
            LiveValue::__fabric_canonical_adapter_bridge_mode()
        }

        fn host_requirement(&self) -> HostRequirement {
            self.host_requirement.clone()
        }

        fn declaration(&self, provider_module_id: ModuleId) -> ModuleDeclaration {
            ModuleDeclaration::new(provider_module_id).with_provided_contracts(vec![
                LiveValue::__fabric_canonical_adapter_contract_key().declaration(),
            ])
        }

        fn materialize_provider_in(
            &self,
            provider_module_id: ModuleId,
            context: &ModuleMaterializationContext<'_>,
        ) -> Result<Option<Box<dyn ModuleRuntime>>, ModuleError> {
            let environment = context.require_input::<TestEnvironment>("TestEnvironment")?;
            Ok(Some(Box::new(LiveValueRuntime {
                module_id: provider_module_id,
                value: format!("{}{}", environment.prefix, self.suffix),
                cleanup: self.cleanup.clone(),
            })))
        }
    }

    #[derive(Clone)]
    struct CleanupModule {
        module_id: ModuleId,
        cleanup: Arc<AtomicUsize>,
    }

    impl ModuleRuntime for CleanupModule {
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

        fn stop(&mut self) -> Result<(), ModuleError> {
            self.cleanup.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn health(&self) -> Health {
            Health::Healthy
        }
    }

    fn live_value_fabric(adapter: LiveValueAdapter) -> Fabric {
        let selected = LiveValue::select("value").expect("selection");
        Fabric::new("fabric.test.live-materialization")
            .expect("fabric")
            .resource(
                selected
                    .clone()
                    .using(adapter)
                    .expect("adapter supports resource"),
            )
            .component(LiveValueConsumer::define().select_named_resource_provider(
                &ComponentResourceRequirement::new(
                    ComponentRelationName::new("value").expect("relation name"),
                    Requires::<LiveValue>::provisional(),
                ),
                &selected,
            ))
    }

    fn read_live_value(composition: &crate::Composition, input: &TestEnvironment) -> String {
        let mut instance = composition
            .materialize_on_with_input(
                "fabric.test.live-materialization.instance",
                &HostDescriptor::native(),
                input,
            )
            .expect("materialize");
        read_live_value_from_instance(&mut instance)
    }

    fn read_live_value_without_input(composition: &crate::Composition) -> String {
        let mut instance = composition
            .materialize_on(
                "fabric.test.live-materialization.instance",
                &HostDescriptor::native(),
            )
            .expect("materialize");
        read_live_value_from_instance(&mut instance)
    }

    fn read_live_value_from_instance(instance: &mut crate::Instance) -> String {
        instance.start().expect("start");
        let components = instance.components().expect("component host");
        components
            .materialize::<LiveValueConsumer>()
            .expect("materialize consumer");
        let value = block_on(components.invoke_external(&live_value_consumer::api::read(), ()))
            .expect("invoke consumer");
        components
            .dematerialize::<LiveValueConsumer>()
            .expect("dematerialize consumer");
        instance.stop().expect("stop");
        value
    }

    #[test]
    fn existing_module_implementations_materialize_unchanged() {
        let cleanup = Arc::new(AtomicUsize::new(0));
        let composition = CompositionBuilder::new(
            composition("fabric.test.live-materialization.old-module").expect("composition"),
        )
        .register_block(
            BlockBuilder::new(block("fabric.test.live-materialization.old-module").expect("block"))
                .register_module(CleanupModule {
                    module_id: module("fabric.test.live-materialization.old-module")
                        .expect("module"),
                    cleanup: Arc::clone(&cleanup),
                })
                .build(),
        )
        .build()
        .expect("composition");

        let mut instance = composition
            .materialize(instance("fabric.test.live-materialization.old-module").expect("instance"))
            .expect("materialize");
        instance.start().expect("start");
        instance.stop().expect("stop");
        assert_eq!(cleanup.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn existing_config_only_adapters_materialize_unchanged() {
        let selected = LiveValue::select("value").expect("selection");
        let composition = Fabric::new("fabric.test.live-materialization.static")
            .expect("fabric")
            .resource(
                selected
                    .clone()
                    .using(StaticLiveValueAdapter::new())
                    .expect("static adapter supports resource"),
            )
            .component(LiveValueConsumer::define().select_named_resource_provider(
                &ComponentResourceRequirement::new(
                    ComponentRelationName::new("value").expect("relation name"),
                    Requires::<LiveValue>::provisional(),
                ),
                &selected,
            ))
            .build()
            .expect("composition");

        assert_eq!(read_live_value_without_input(&composition), "static");
    }

    #[test]
    fn live_aware_adapter_uses_config_and_typed_input_for_state() {
        let composition = live_value_fabric(LiveValueAdapter::new("-suffix"))
            .build()
            .expect("composition");

        assert_eq!(
            read_live_value(
                &composition,
                &TestEnvironment {
                    prefix: "alpha".to_owned(),
                },
            ),
            "alpha-suffix"
        );
    }

    #[test]
    fn missing_input_fails_during_materialization() {
        let composition = live_value_fabric(LiveValueAdapter::new("-suffix"))
            .build()
            .expect("composition");
        let error = composition
            .materialize_on(
                "fabric.test.live-materialization.missing",
                &HostDescriptor::native(),
            )
            .expect_err("missing input must fail");

        assert!(matches!(
            error,
            CompositionError::ModuleFailure {
                phase: "materialize",
                ..
            }
        ));
    }

    #[test]
    fn wrong_input_type_fails_during_materialization() {
        let composition = live_value_fabric(LiveValueAdapter::new("-suffix"))
            .build()
            .expect("composition");
        let error = composition
            .materialize_on_with_input(
                "fabric.test.live-materialization.wrong",
                &HostDescriptor::native(),
                &WrongEnvironment,
            )
            .expect_err("wrong input must fail");

        assert!(matches!(
            error,
            CompositionError::ModuleFailure {
                phase: "materialize",
                ..
            }
        ));
    }

    #[test]
    fn host_materialization_path_accepts_live_input_without_changing_host() {
        let facility =
            HostFacilityId::new("fabric.test.live-materialization.facility").expect("facility");
        let host_requirement = HostRequirement::new().require_facility(facility.clone());
        let composition = live_value_fabric(
            LiveValueAdapter::new("-host").with_host_requirement(host_requirement),
        )
        .build()
        .expect("composition");
        let host = HostDescriptor::native().with_facility(facility);
        let mut instance = composition
            .materialize_on_with_input(
                "fabric.test.live-materialization.host",
                &host,
                &TestEnvironment {
                    prefix: "ok".to_owned(),
                },
            )
            .expect("host materialize");
        instance.start().expect("start");
        instance.stop().expect("stop");
    }

    #[test]
    fn materialization_failure_cleans_previously_constructed_runtimes() {
        let cleanup = Arc::new(AtomicUsize::new(0));
        let core_composition =
            live_value_fabric(LiveValueAdapter::new("-suffix").with_cleanup(Arc::clone(&cleanup)))
                .build()
                .expect("composition");

        let error = core_composition
            .materialize_on_with_input(
                "fabric.test.live-materialization.cleanup",
                &HostDescriptor::native(),
                &WrongEnvironment,
            )
            .expect_err("wrong input must fail");
        assert!(matches!(
            error,
            CompositionError::ModuleFailure {
                phase: "materialize",
                ..
            }
        ));
        assert_eq!(cleanup.load(Ordering::SeqCst), 0);

        let core_cleanup = Arc::new(AtomicUsize::new(0));
        let core = CompositionBuilder::new(
            crate::ids::composition("fabric.test.live-materialization.cleanup-core")
                .expect("composition"),
        )
        .register_block(
            BlockBuilder::new(
                block("fabric.test.live-materialization.cleanup-core").expect("block"),
            )
            .register_module(CleanupModule {
                module_id: module("fabric.test.live-materialization.cleanup-first")
                    .expect("module"),
                cleanup: Arc::clone(&core_cleanup),
            })
            .register_module(FailingInputModule {
                module_id: module("fabric.test.live-materialization.cleanup-failing")
                    .expect("module"),
            })
            .build(),
        )
        .build()
        .expect("composition");
        let error = core
            .materialize(instance("fabric.test.live-materialization.cleanup-core").expect("id"))
            .expect_err("missing input must fail");
        assert!(matches!(
            error,
            CompositionError::ModuleFailure {
                phase: "materialize",
                ..
            }
        ));
        assert_eq!(core_cleanup.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn rematerialization_uses_separate_generation_local_state() {
        let composition = live_value_fabric(LiveValueAdapter::new("-state"))
            .build()
            .expect("composition");
        assert_eq!(
            read_live_value(
                &composition,
                &TestEnvironment {
                    prefix: "alpha".to_owned(),
                },
            ),
            "alpha-state"
        );
        assert_eq!(
            read_live_value(
                &composition,
                &TestEnvironment {
                    prefix: "beta".to_owned(),
                },
            ),
            "beta-state"
        );
    }

    #[test]
    fn input_is_not_retained_in_plan_or_instance_metadata() {
        let composition = live_value_fabric(LiveValueAdapter::new("-metadata"))
            .build()
            .expect("composition");
        let host = HostDescriptor::native();
        let plan = composition.plan_on(&host).expect("plan");
        assert_eq!(plan.host(), Some(&host));

        let instance = plan
            .materialize_with_input(
                "fabric.test.live-materialization.metadata",
                &TestEnvironment {
                    prefix: "meta".to_owned(),
                },
            )
            .expect("materialize");
        let observation = instance.observe();
        assert_eq!(observation.materialization_plan().host(), Some(&host));
        assert_eq!(
            observation.materialization_plan().materialization_profile(),
            plan.materialization_profile()
        );
    }

    struct FailingInputModule {
        module_id: ModuleId,
    }

    impl Module for FailingInputModule {
        fn declaration(&self) -> ModuleDeclaration {
            ModuleDeclaration::new(self.module_id.clone())
        }

        fn materialize_in(
            &self,
            context: &ModuleMaterializationContext<'_>,
        ) -> Result<Option<Box<dyn ModuleRuntime>>, ModuleError> {
            let _ = context.require_input::<TestEnvironment>("TestEnvironment")?;
            Ok(None)
        }
    }
}
