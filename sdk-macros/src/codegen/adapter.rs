use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::ast::{AdapterInput, RelationDefinition};

use super::common::{
    config_type_tokens, fabric_path, has_config, inline_config_definition_tokens,
    runtime_method_tokens, to_snake_case,
};

pub fn expand_adapter(input: &AdapterInput) -> TokenStream {
    expand_canonical_adapter(input)
}

/// Lowers the normal `adapter! { Name for Target { ... } }` form.  The
/// target's generated primary API service is intentionally referenced only by
/// generated code: the author declares the target once and never names an
/// implementation trait or duplicate realization contract.
fn expand_canonical_adapter(input: &AdapterInput) -> TokenStream {
    let sdk = fabric_path();
    let visibility = &input.visibility;
    let adapter_name = &input.name;
    let adapter_id = &input.adapter_id;
    let config_name = format_ident!("{}Config", adapter_name);
    let config_ty = config_type_tokens(&input.config, &config_name);
    let config_definition =
        inline_config_definition_tokens(&input.config, visibility, &config_name);
    let constructor = if has_config(&input.config) {
        quote! {
            pub fn new(config: #config_ty) -> Self {
                Self { config }
            }
        }
    } else {
        quote! {
            pub fn new() -> Self {
                Self { config: () }
            }
        }
    };
    let adapter_mod = format_ident!("{}", to_snake_case(adapter_name));
    let adapter_relations_name = format_ident!("{}Relations", adapter_name);
    let raw_impl_mod = format_ident!("__fabric_adapter_raw_{}", to_snake_case(adapter_name));
    let target = &input.target;
    let target_key = quote!(<#target>::__fabric_canonical_adapter_contract_key());
    let canonical_service_steps = input.runtime_methods.iter().map(|method| {
        let name = &method.signature.ident;
        quote!(.#name(Self::#name))
    });
    let canonical_contract = quote! {
        <#target>::__fabric_canonical_adapter_builder()
            #(#canonical_service_steps)*
            .build(::std::sync::Arc::new(self.clone()))
    };
    let support = match input.schema.as_ref() {
        None => quote!(#sdk::authoring::CanonicalAdapterSupport::<#target>::inferred()),
        Some(crate::ast::RequirementLiteral::Provisional) => {
            quote!(#sdk::authoring::CanonicalAdapterSupport::<#target>::provisional())
        }
        Some(crate::ast::RequirementLiteral::Versioned(requirement)) => {
            quote!(#sdk::authoring::CanonicalAdapterSupport::<#target>::versioned(#requirement))
        }
    };
    let dependency_fields = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        let contract_ty = dependency_contract_type(&sdk, dependency);
        quote!(#field: #sdk::authoring::ContractDependency<#contract_ty>,)
    });
    let dependency_initializers = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        let requirement_expr = system_requirement_expr(&sdk, dependency);
        quote! {
            #field: #sdk::authoring::ContractDependency::new(#requirement_expr),
        }
    });
    let dependency_declarations = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        quote!(self.#field.declaration().clone())
    });
    let declaration_requirements = input.relations.iter().map(|dependency| {
        let requirement = system_requirement_expr(&sdk, dependency);
        quote!(#requirement.declaration().clone())
    });
    let relation_metadata = input
        .relations
        .iter()
        .map(|relation| {
            let name = relation.field.to_string();
            let target = &relation.target;
            let requirement = system_requirement_expr(&sdk, relation);
            quote!((
                #sdk::component::ComponentRelationName::new(#name)
                    .expect("adapter! generated a non-empty relation role"),
                <#target as #sdk::authoring::RelationTarget>::relation_target_descriptor(),
                #requirement.declaration().clone(),
            ))
        })
        .collect::<Vec<_>>();
    let dependency_bindings = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        quote! {
            self.#field
                .bind(bindings)
                .map_err(|error| #sdk::core::ModuleError::new(error.to_string()))?;
        }
    });
    let dependency_clones = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        quote!(#field: self.#field.clone(),)
    });
    let runtime_inherent_methods = input.runtime_methods.iter().map(runtime_method_tokens);
    let runtime_state_field = input.runtime_state.as_ref().map(|state| {
        let ty = &state.ty;
        quote!(state: #sdk::authoring::AdapterRuntimeState<#ty>,)
    });
    let participation_runtime_state_initializer = input.runtime_state.as_ref().map(|state| {
        let initializer = &state.initializer;
        quote! {
            let state = #sdk::authoring::AdapterRuntimeState::new({
                let config = &config;
                #initializer
            });
        }
    });
    let participation_runtime_state_value = input.runtime_state.as_ref().map(|_| quote!(state,));
    let provider_runtime_state_initializer = input.runtime_state.as_ref().map(|state| {
        let initializer = &state.initializer;
        quote! {
            let state = if <#target as #sdk::authoring::ComponentAdapterTarget>::is_component_participation_target() {
                #sdk::authoring::AdapterRuntimeState::absent()
            } else {
                #sdk::authoring::AdapterRuntimeState::new({
                    let config = &config;
                    #initializer
                })
            };
        }
    });
    let runtime_provider_state = input.runtime_state.as_ref().map(|_| quote!(state,));
    let runtime_state_accessor = input.runtime_state.as_ref().map(|state| {
        let ty = &state.ty;
        quote!(pub fn state(&self) -> &#sdk::authoring::RuntimeState<#ty> { self.state.as_runtime_state() })
    });
    let adapter_relation_fields = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        let contract_ty = dependency_contract_type(&sdk, dependency);
        quote!(pub #field: ::std::sync::Arc<#contract_ty>,)
    });
    let adapter_relation_values = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        quote!(#field: self.#field.value(),)
    });
    let component_prepare_hook = input
        .component_prepare
        .as_ref()
        .map(|body| {
            let statements = &body.stmts;
            quote!(#(#statements)*)
        })
        .unwrap_or_else(|| quote!(Ok(())));
    let component_teardown_hook = input
        .component_teardown
        .as_ref()
        .map(|body| {
            let statements = &body.stmts;
            quote!(#(#statements)*)
        })
        .unwrap_or_else(|| quote!(Ok(())));
    let initialize_hook = input
        .lifecycle
        .initialize
        .as_ref()
        .map(|body| quote!(#body))
        .unwrap_or_else(|| quote!(Ok(())));
    let start_hook = input
        .lifecycle
        .start
        .as_ref()
        .map(|body| quote!(#body))
        .unwrap_or_else(|| quote!(Ok(())));
    let stop_hook = input
        .lifecycle
        .stop
        .as_ref()
        .map(|body| quote!(#body))
        .unwrap_or_else(|| quote!(Ok(())));
    let health_hook = input
        .lifecycle
        .health
        .as_ref()
        .map(|expr| quote!(#expr))
        .unwrap_or_else(|| quote!(#sdk::core::Health::Healthy));
    let host_requirement_expr = input
        .host_requirement
        .as_ref()
        .map(|expr| quote!(#expr))
        .unwrap_or_else(|| quote!(#sdk::host::HostRequirement::new()));

    quote! {
        #config_definition

        #[derive(Clone)]
        #visibility struct #adapter_name {
            config: #config_ty,
        }

        impl #adapter_name {
            #constructor

            pub fn config(&self) -> &#config_ty {
                &self.config
            }
        }

        impl #sdk::authoring::AdapterDefinition for #adapter_name {
            type Target = #target;
            type Compatibility = #sdk::authoring::CanonicalAdapterSupport<#target>;

            fn adapter_definition_id(&self) -> #sdk::authoring::AdapterDefinitionId {
                #sdk::authoring::AdapterDefinitionId::new(#adapter_id)
                    .expect("adapter! generated a valid static Adapter definition ID")
            }

            fn compatibility(&self) -> Self::Compatibility {
                #support
            }

            fn bridge_mode(&self) -> #sdk::authoring::AdapterBridgeMode {
                <#target>::__fabric_canonical_adapter_bridge_mode()
            }

            fn host_requirement(&self) -> #sdk::host::HostRequirement {
                #host_requirement_expr
            }

            fn declaration(
                &self,
                provider_module_id: #sdk::core::ModuleId,
            ) -> #sdk::core::ModuleDeclaration {
                let key = #target_key;
                #sdk::core::ModuleDeclaration::new(provider_module_id)
                    .with_provided_contracts(::std::vec![key.declaration()])
                    .with_required_contracts(::std::vec![#(#declaration_requirements),*])
            }

            fn relation_declarations(
                &self,
                _provider_module_id: #sdk::core::ModuleId,
            ) -> ::std::vec::Vec<(
                #sdk::component::ComponentRelationName,
                #sdk::authoring::RelationTargetDescriptor,
                #sdk::core::ContractRequirementDeclaration,
            )> {
                ::std::vec![#(#relation_metadata),*]
            }

            fn materialize_provider(
                &self,
                provider_module_id: #sdk::core::ModuleId,
            ) -> ::std::option::Option<::std::boxed::Box<dyn #sdk::core::ModuleRuntime>> {
                ::std::option::Option::Some(::std::boxed::Box::new(#adapter_mod::raw::Runtime::new(
                    provider_module_id,
                    self.config.clone(),
                )))
            }
        }

        #[allow(non_snake_case)]
        mod #raw_impl_mod {
            use super::*;

            #[derive(Clone)]
            pub struct #adapter_relations_name {
                #(#adapter_relation_fields)*
            }

            #[derive(Clone)]
            pub struct Runtime {
                module_id: #sdk::core::ModuleId,
                config: #config_ty,
                runtime_context: #sdk::authoring::RuntimeContext,
                component_config: ::std::option::Option<<#target as #sdk::authoring::ComponentAdapterTarget>::ComponentConfig>,
                component_relations: ::std::option::Option<<#target as #sdk::authoring::ComponentAdapterTarget>::ComponentRelations>,
                #runtime_state_field
                #(#dependency_fields)*
            }

            impl Runtime {
                pub fn new(module_id: #sdk::core::ModuleId, config: #config_ty) -> Self {
                    #provider_runtime_state_initializer
                    Self {
                        module_id,
                        config,
                        runtime_context: #sdk::authoring::RuntimeContext::default(),
                        component_config: ::std::option::Option::None,
                        component_relations: ::std::option::Option::None,
                        #runtime_provider_state
                        #(#dependency_initializers)*
                    }
                }

                pub fn config(&self) -> &#config_ty {
                    &self.config
                }

                pub fn relations(&self) -> #adapter_relations_name {
                    #adapter_relations_name { #(#adapter_relation_values)* }
                }

                pub fn component_config(&self) -> &<#target as #sdk::authoring::ComponentAdapterTarget>::ComponentConfig {
                    self.component_config.as_ref().expect("Component config exists only for a prepared Component participation")
                }

                pub fn component_relations(&self) -> &<#target as #sdk::authoring::ComponentAdapterTarget>::ComponentRelations {
                    self.component_relations.as_ref().expect("Component relations exist only for a prepared Component participation")
                }

                #runtime_state_accessor

                #(#runtime_inherent_methods)*

                fn __fabric_component_prepare(&self) -> ::std::result::Result<(), #sdk::component::ComponentError> {
                    #component_prepare_hook
                }

                fn __fabric_component_teardown(&self) -> ::std::result::Result<(), #sdk::component::ComponentError> {
                    #component_teardown_hook
                }
            }

            impl #sdk::authoring::CanonicalComponentAdapterRuntime<#target> for Runtime {
                type ParticipationRuntime = Runtime;

                fn provider_runtime(&self) -> ::std::sync::Arc<Self::ParticipationRuntime> {
                    ::std::sync::Arc::new(self.clone())
                }

                fn prepare_component_runtime(
                    &self,
                    component_config: <#target as #sdk::authoring::ComponentAdapterTarget>::ComponentConfig,
                    component_relations: <#target as #sdk::authoring::ComponentAdapterTarget>::ComponentRelations,
                ) -> ::std::result::Result<Self::ParticipationRuntime, #sdk::component::ComponentError> {
                    let config = &self.config;
                    #participation_runtime_state_initializer
                    Ok(Self {
                        module_id: self.module_id.clone(),
                        config: self.config.clone(),
                        runtime_context: self.runtime_context.clone(),
                        component_config: ::std::option::Option::Some(component_config),
                        component_relations: ::std::option::Option::Some(component_relations),
                        #participation_runtime_state_value
                        #(#dependency_clones)*
                    })
                }

                fn component_prepare(&self, runtime: &Self::ParticipationRuntime) -> ::std::result::Result<(), #sdk::component::ComponentError> {
                    let _ = self;
                    runtime.__fabric_component_prepare()
                }

                fn component_teardown(&self, runtime: &Self::ParticipationRuntime) -> ::std::result::Result<(), #sdk::component::ComponentError> {
                    let _ = self;
                    runtime.__fabric_component_teardown()
                }
            }

            impl #sdk::core::ModuleRuntime for Runtime {
                fn id(&self) -> &#sdk::core::ModuleId { &self.module_id }

                fn provided_contract_declarations(
                    &self,
                ) -> ::std::vec::Vec<#sdk::core::ProvidedContractDeclaration> {
                    let key = #target_key;
                    ::std::vec![key.declaration()]
                }

                fn required_contract_declarations(
                    &self,
                ) -> ::std::vec::Vec<#sdk::core::ContractRequirementDeclaration> {
                    ::std::vec![#(#dependency_declarations),*]
                }

                fn export_contracts(
                    &self,
                ) -> ::std::result::Result<::std::vec::Vec<#sdk::core::ModuleContract>, #sdk::core::ModuleError> {
                    let key = #target_key;
                    let contract = #canonical_contract;
                    Ok(::std::vec![#sdk::core::ModuleContract::new(
                        &key,
                        ::std::sync::Arc::new(contract),
                    )])
                }

                fn bind(
                    &mut self,
                    bindings: &#sdk::core::ModuleBindings,
                ) -> ::std::result::Result<(), #sdk::core::ModuleError> {
                    #(#dependency_bindings)*
                    Ok(())
                }

                fn bind_instance_context(
                    &mut self,
                    context: &#sdk::core::InstanceRuntimeContext,
                ) -> ::std::result::Result<(), #sdk::core::ModuleError> {
                    self.runtime_context.bind(context.clone());
                    Ok(())
                }

                fn initialize(&mut self) -> ::std::result::Result<(), #sdk::core::ModuleError> { #initialize_hook }
                fn start(&mut self) -> ::std::result::Result<(), #sdk::core::ModuleError> { #start_hook }
                fn stop(&mut self) -> ::std::result::Result<(), #sdk::core::ModuleError> { #stop_hook }
                fn health(&self) -> #sdk::core::Health { #health_hook }
            }
        }

        // The runtime bridge is generated implementation detail. The public
        // Adapter type is the author-facing surface; exposing this raw module
        // would leak the provider machinery canonical authoring removes.
        #[doc(hidden)]
        mod #adapter_mod {
            #[doc(hidden)]
            pub mod raw {
                pub use super::super::#raw_impl_mod::Runtime;
            }
        }
    }
}

fn dependency_contract_type(sdk: &TokenStream, dependency: &RelationDefinition) -> TokenStream {
    let target = &dependency.target;
    quote!(<#target as #sdk::authoring::RelationTarget>::Contract)
}

fn system_requirement_expr(sdk: &TokenStream, dependency: &RelationDefinition) -> TokenStream {
    let target = &dependency.target;
    match &dependency.compatibility {
        None => quote!(<#target as #sdk::authoring::RelationTarget>::relation_requirement()),
        Some(crate::ast::RequirementLiteral::Provisional) => {
            quote!(#sdk::core::ContractRequirement::<<#target as #sdk::authoring::RelationTarget>::Contract>::provisional(<#target as #sdk::authoring::RelationTarget>::relation_requirement().id().clone()))
        }
        Some(crate::ast::RequirementLiteral::Versioned(version)) => quote!(
            <#target as #sdk::authoring::RelationTarget>::relation_requirement_versioned(
                #sdk::core::ContractVersionRequirement::parse(#version)
                    .expect("adapter! generated a static relation version requirement"),
            )
        ),
    }
}
