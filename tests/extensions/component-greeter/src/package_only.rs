//! Package-style Components defined entirely through public
//! authoring APIs with no runtime preparation or native runtime adoption.
//!
//! `PackageComponent` carries stable identity plus one declarative behavior
//! endpoint. `EmptyComponent` carries stable identity with zero endpoints,
//! proving that a ComponentInstanceBinding without invocation-based behavior is valid.
//!
//! Both intentionally define none of: `prepare`, `ComponentParticipationScope`,
//! `ComponentParticipationRealization`, `Health`, handlers, `ModuleRuntime`,
//! `Instance`, or any optional rail (registry, control, readiness,
//! surface, reconstruction).
//!
//! A `src/tests.rs` source guard pins this negative space so the package
//! cannot silently regain a runtime obligation.

use fabric::authoring::component::ComponentDefinition;
use fabric_component::declaration::{ComponentDeclaration, ComponentId};
use fabric_component::invocation::{OperationDefinition, OperationId, OperationTypeId};

#[derive(Clone)]
pub struct PackageComponentConfig {
    pub prefix: String,
}

pub struct PackageComponent;

impl ComponentDefinition for PackageComponent {
    type Config = PackageComponentConfig;

    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.package-component").expect("static component id")
    }

    fn declaration() -> ComponentDeclaration {
        ComponentDeclaration::new(
            Self::component_id(),
            vec![OperationDefinition::new(
                OperationId::new("fabric.test.package-component.describe")
                    .expect("static operation id"),
                OperationTypeId::new("fabric.test.package-component.describe.input")
                    .expect("static input type id"),
                OperationTypeId::new("fabric.test.package-component.describe.output")
                    .expect("static output type id"),
            )],
        )
    }
}

#[derive(Clone)]
pub struct EmptyComponentConfig;

pub struct EmptyComponent;

impl ComponentDefinition for EmptyComponent {
    type Config = EmptyComponentConfig;

    fn component_id() -> ComponentId {
        ComponentId::new("fabric.test.empty-component").expect("static component id")
    }

    fn declaration() -> ComponentDeclaration {
        ComponentDeclaration::new(Self::component_id(), Vec::new())
    }
}
