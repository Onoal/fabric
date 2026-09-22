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
    let dependency_bindings = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        quote! {
            self.#field
                .bind(bindings)
                .map_err(|error| #sdk::core::ModuleError::new(error.to_string()))?;
        }
    });
    let runtime_inherent_methods = input.runtime_methods.iter().map(runtime_method_tokens);
    let runtime_state_field = input.runtime_state.as_ref().map(|state| {
        let ty = &state.ty;
        quote!(state: #sdk::authoring::RuntimeState<#ty>,)
    });
    let runtime_state_initializer = input.runtime_state.as_ref().map(|state| {
        let initializer = &state.initializer;
        quote! {
            let state = #sdk::authoring::RuntimeState::new({
                let config = &config;
                #initializer
            });
        }
    });
    let runtime_state_value = input.runtime_state.as_ref().map(|_| quote!(state,));
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
            pub struct Runtime {
                module_id: #sdk::core::ModuleId,
                config: #config_ty,
                runtime_context: #sdk::authoring::RuntimeContext,
                #runtime_state_field
                #(#dependency_fields)*
            }

            impl Runtime {
                pub fn new(module_id: #sdk::core::ModuleId, config: #config_ty) -> Self {
                    #runtime_state_initializer
                    Self {
                        module_id,
                        config,
                        runtime_context: #sdk::authoring::RuntimeContext::default(),
                        #runtime_state_value
                        #(#dependency_initializers)*
                    }
                }

                pub fn config(&self) -> &#config_ty {
                    &self.config
                }

                #(#runtime_inherent_methods)*
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
