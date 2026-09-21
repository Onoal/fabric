use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

use crate::ast::{AdapterInput, AdapterTargetKind, SystemDependencyDefinition};

use super::common::{fabric_path, method_call_args, runtime_method_tokens, to_snake_case};

pub fn expand_adapter(input: &AdapterInput) -> TokenStream {
    let sdk = fabric_path();
    let visibility = &input.visibility;
    let adapter_name = &input.name;
    let config_name = format_ident!("{}Config", adapter_name);
    let adapter_mod = format_ident!("{}", to_snake_case(adapter_name));
    let raw_impl_mod = format_ident!("__fabric_adapter_raw_{}", to_snake_case(adapter_name));
    let target = &input.target;
    let interface = &input.realization_interface;
    let interface_assertion = quote! {
        let _: fn() -> ::std::marker::PhantomData<#target> =
            <#raw_impl_mod::Runtime as #interface>::realization_target;
    };
    let target_service = quote!(#interface);
    let schema_support_ty = schema_support_type(&sdk, input.target_kind);
    let schema_support_expr = schema_support_expr(&sdk, input.target_kind, target, &input.schema);
    let realization_key_expr =
        realization_key_expr_interface(&sdk, interface, &raw_impl_mod, &input.realization);
    let realization_contract_expr = quote!(<#raw_impl_mod::Runtime as #interface>::realization_contract(
        ::std::sync::Arc::new(self.clone())
    ));
    let config_fields = input.config_fields.iter().map(|field| {
        let name = &field.name;
        let ty = &field.ty;
        quote!(pub #name: #ty,)
    });
    let dependency_fields = input.systems.iter().map(|dependency| {
        let field = &dependency.field;
        let contract_ty = dependency_contract_type(&sdk, dependency);
        quote!(#field: #sdk::authoring::ContractDependency<#contract_ty>,)
    });
    let dependency_initializers = input.systems.iter().map(|dependency| {
        let field = &dependency.field;
        let requirement_expr = system_requirement_expr(&sdk, dependency);
        quote! {
            #field: #sdk::authoring::ContractDependency::new(
                #requirement_expr.as_contract_requirement().clone()
            ),
        }
    });
    let dependency_declarations = input
        .systems
        .iter()
        .map(|dependency| {
            let field = &dependency.field;
            quote!(self.#field.declaration().clone())
        })
        .collect::<Vec<_>>();
    let declaration_requirements = input
        .systems
        .iter()
        .map(|dependency| {
            let requirement = system_requirement_expr(&sdk, dependency);
            quote!(#requirement.declaration().clone())
        })
        .collect::<Vec<_>>();
    let dependency_bindings = input.systems.iter().map(|dependency| {
        let field = &dependency.field;
        quote! {
            self.#field
                .bind(bindings)
                .map_err(|error| #sdk::core::ModuleError::new(error.to_string()))?;
        }
    });
    let runtime_inherent_methods = input.runtime_methods.iter().map(runtime_method_tokens);
    let runtime_trait_methods = input.runtime_methods.iter().map(|method| {
        let signature = &method.signature;
        let name = &signature.ident;
        let args = method_call_args(signature);
        quote! {
            #signature {
                Self::#name(self, #(#args),*)
            }
        }
    });
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
        #[derive(Clone)]
        #visibility struct #config_name {
            #(#config_fields)*
        }

        #[derive(Clone)]
        #visibility struct #adapter_name {
            config: #config_name,
        }

        impl #adapter_name {
            pub fn new(config: #config_name) -> Self {
                Self { config }
            }
        }

        impl #sdk::authoring::AdapterDefinition for #adapter_name {
            type Target = #target;
            type Compatibility = #schema_support_ty;

            fn compatibility(&self) -> Self::Compatibility {
                #schema_support_expr
            }

            fn host_requirement(&self) -> #sdk::host::HostRequirement {
                #host_requirement_expr
            }

            fn declaration(
                &self,
                provider_module_id: #sdk::core::ModuleId,
            ) -> #sdk::core::ModuleDeclaration {
                #interface_assertion
                let key = #realization_key_expr;
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
                config: #config_name,
                runtime_context: #sdk::authoring::RuntimeContext,
                #runtime_state_field
                #(#dependency_fields)*
            }

            impl Runtime {
                pub fn new(module_id: #sdk::core::ModuleId, config: #config_name) -> Self {
                    #runtime_state_initializer
                    Self {
                        module_id,
                        config,
                        runtime_context: #sdk::authoring::RuntimeContext::default(),
                        #runtime_state_value
                        #(#dependency_initializers)*
                    }
                }

                #(#runtime_inherent_methods)*
            }

            impl #target_service for Runtime {
                #(#runtime_trait_methods)*
            }

            impl #sdk::core::ModuleRuntime for Runtime {
                fn id(&self) -> &#sdk::core::ModuleId {
                    &self.module_id
                }

                fn provided_contract_declarations(
                    &self,
                ) -> ::std::vec::Vec<#sdk::core::ProvidedContractDeclaration> {
                    let key = #realization_key_expr;
                    ::std::vec![key.declaration()]
                }

                fn required_contract_declarations(
                    &self,
                ) -> ::std::vec::Vec<#sdk::core::ContractRequirementDeclaration> {
                    ::std::vec![#(#dependency_declarations),*]
                }

                fn export_contracts(
                    &self,
                ) -> ::std::result::Result<
                    ::std::vec::Vec<#sdk::core::ModuleContract>,
                    #sdk::core::ModuleError,
                > {
                    let key = #realization_key_expr;
                    Ok(::std::vec![#sdk::core::ModuleContract::new(
                        &key,
                        ::std::sync::Arc::new(#realization_contract_expr),
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

                fn initialize(&mut self) -> ::std::result::Result<(), #sdk::core::ModuleError> {
                    #initialize_hook
                }

                fn start(&mut self) -> ::std::result::Result<(), #sdk::core::ModuleError> {
                    #start_hook
                }

                fn stop(&mut self) -> ::std::result::Result<(), #sdk::core::ModuleError> {
                    #stop_hook
                }

                fn health(&self) -> #sdk::core::Health {
                    #health_hook
                }
            }
        }

        #visibility mod #adapter_mod {
            pub mod raw {
                pub use super::super::#raw_impl_mod::Runtime;
            }
        }
    }
}

fn schema_support_type(sdk: &TokenStream, target_kind: AdapterTargetKind) -> TokenStream {
    match target_kind {
        AdapterTargetKind::Resource => quote!(#sdk::resource::AdapterResourceSchemaSupport),
        AdapterTargetKind::System => quote!(#sdk::system::AdapterSystemSchemaSupport),
    }
}

fn schema_support_expr(
    sdk: &TokenStream,
    target_kind: AdapterTargetKind,
    target: &Path,
    schema: &crate::ast::RequirementLiteral,
) -> TokenStream {
    match target_kind {
        AdapterTargetKind::Resource => match schema {
            crate::ast::RequirementLiteral::Provisional => quote! {
                #sdk::resource::AdapterResourceSchemaSupport::provisional(
                    <#target as #sdk::authoring::ResourceDefinition>::resource_id()
                )
            },
            crate::ast::RequirementLiteral::Versioned(version) => quote! {
                #sdk::resource::AdapterResourceSchemaSupport::versioned(
                    <#target as #sdk::authoring::ResourceDefinition>::resource_id(),
                    #sdk::resource::ResourceSchemaRequirement::parse(#version)
                        .expect("adapter! generated a static resource schema requirement"),
                )
            },
        },
        AdapterTargetKind::System => match schema {
            crate::ast::RequirementLiteral::Provisional => quote! {
                #sdk::system::AdapterSystemSchemaSupport::provisional(
                    <#target as #sdk::authoring::SystemDefinition>::system_id()
                )
            },
            crate::ast::RequirementLiteral::Versioned(version) => quote! {
                #sdk::system::AdapterSystemSchemaSupport::versioned(
                    <#target as #sdk::authoring::SystemDefinition>::system_id(),
                    #sdk::system::SystemSchemaRequirement::parse(#version)
                        .expect("adapter! generated a static system schema requirement"),
                )
            },
        },
    }
}

fn realization_key_expr_interface(
    sdk: &TokenStream,
    interface: &syn::Path,
    runtime_mod: &proc_macro2::Ident,
    version: &crate::ast::VersionLiteral,
) -> TokenStream {
    match version {
        crate::ast::VersionLiteral::Provisional => quote!(
            <#runtime_mod::Runtime as #interface>::provisional_realization_contract_key()
        ),
        crate::ast::VersionLiteral::Versioned(version) => quote!(
            <#runtime_mod::Runtime as #interface>::realization_contract_key(
                #sdk::core::ContractVersion::parse(#version)
                    .expect("adapter! generated a static realization version"),
            )
        ),
    }
}

fn dependency_contract_type(
    sdk: &TokenStream,
    dependency: &SystemDependencyDefinition,
) -> TokenStream {
    let system = &dependency.system;
    quote!(<#system as #sdk::authoring::PrimarySystemContract>::Contract)
}

fn system_requirement_expr(
    sdk: &TokenStream,
    dependency: &SystemDependencyDefinition,
) -> TokenStream {
    let system = &dependency.system;
    match &dependency.compatibility {
        crate::ast::RequirementLiteral::Provisional => {
            quote!(#sdk::authoring::SystemRequires::<#system>::provisional())
        }
        crate::ast::RequirementLiteral::Versioned(version) => quote!(
            #sdk::authoring::SystemRequires::<#system>::versioned(
                #sdk::core::ContractVersionRequirement::parse(#version)
                    .expect("adapter! generated a static dependency version requirement"),
            )
        ),
    }
}
