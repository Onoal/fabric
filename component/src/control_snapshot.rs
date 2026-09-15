use fabric_core::InstanceId;

use crate::{ComponentControl, ComponentDesiredState, ComponentError, ComponentId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentControlSnapshot {
    instance_id: InstanceId,
    entries: Vec<ComponentControlSnapshotEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentControlSnapshotEntry {
    component_id: ComponentId,
    desired: ComponentDesiredState,
}

impl ComponentControlSnapshot {
    pub fn new(
        instance_id: InstanceId,
        entries: impl IntoIterator<Item = ComponentControlSnapshotEntry>,
    ) -> Result<Self, ComponentError> {
        let mut entries = entries.into_iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.component_id().cmp(right.component_id()));
        for pair in entries.windows(2) {
            if pair[0].component_id() == pair[1].component_id() {
                return Err(ComponentError::DuplicateComponentControlSnapshotEntry(
                    pair[0].component_id().clone(),
                ));
            }
        }
        Ok(Self {
            instance_id,
            entries,
        })
    }

    pub(crate) fn from_controls(
        instance_id: InstanceId,
        controls: impl IntoIterator<Item = ComponentControl>,
    ) -> Self {
        Self::new(
            instance_id,
            controls
                .into_iter()
                .map(ComponentControlSnapshotEntry::from_control),
        )
        .expect("component control map must stay duplicate-free")
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }

    pub fn entries(&self) -> &[ComponentControlSnapshotEntry] {
        &self.entries
    }
}

impl ComponentControlSnapshotEntry {
    pub fn new(component_id: ComponentId, desired: ComponentDesiredState) -> Self {
        Self {
            component_id,
            desired,
        }
    }

    pub(crate) fn from_control(control: ComponentControl) -> Self {
        Self {
            component_id: control.component().component_id().clone(),
            desired: control.desired(),
        }
    }

    pub fn component_id(&self) -> &ComponentId {
        &self.component_id
    }

    pub fn desired(&self) -> ComponentDesiredState {
        self.desired
    }
}
