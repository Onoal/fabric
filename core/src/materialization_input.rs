use std::any::Any;

use crate::error::ModuleError;

/// One typed live root supplied only while constructing a fresh Instance
/// generation.
///
/// This is construction-time machinery. It is not Composition truth, Host
/// truth, Plan truth, RuntimeContext, an Instance Facility, or retained
/// Instance state.
#[derive(Clone, Copy)]
pub struct MaterializationInput<'a> {
    root: &'a (dyn Any + Send + Sync),
}

impl<'a> MaterializationInput<'a> {
    pub fn new<T>(root: &'a T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self { root }
    }

    pub fn downcast_ref<T>(&self) -> Option<&'a T>
    where
        T: 'static,
    {
        self.root.downcast_ref::<T>()
    }
}

#[derive(Clone, Copy, Default)]
pub struct ModuleMaterializationContext<'a> {
    input: Option<MaterializationInput<'a>>,
}

impl<'a> ModuleMaterializationContext<'a> {
    pub fn empty() -> Self {
        Self { input: None }
    }

    pub fn with_input<T>(input: &'a T) -> Self
    where
        T: Send + Sync + 'static,
    {
        Self {
            input: Some(MaterializationInput::new(input)),
        }
    }

    pub fn input(&self) -> Option<MaterializationInput<'a>> {
        self.input
    }

    pub fn input_as<T>(&self) -> Option<&'a T>
    where
        T: 'static,
    {
        self.input.and_then(|input| input.downcast_ref::<T>())
    }

    pub fn require_input<T>(&self, expected: &'static str) -> Result<&'a T, ModuleError>
    where
        T: 'static,
    {
        self.input_as::<T>().ok_or_else(|| {
            ModuleError::new(format!(
                "materialization input `{expected}` was not supplied or had the wrong type"
            ))
        })
    }
}
