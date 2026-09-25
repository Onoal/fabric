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
