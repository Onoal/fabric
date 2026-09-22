use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::ast::{RealizationDefinition, SystemDependencyDefinition, SystemInput};

use super::common::{
    PrimaryContractTokens, SubjectKind, fabric_path, method_call_args, primary_contract_tokens,
    requirement_literal_expr, runtime_method_tokens, to_snake_case, version_literal_expr,
};

pub fn expand_system(input: &SystemInput) -> TokenStream {
    let sdk = fabric_path();
    let visibility = &input.visibility;
    let system_name = &input.name;
    let config_name = format_ident!("{}Config", system_name);
    let system_mod = format_ident!("{}", to_snake_case(system_name));
    let raw_impl_mod = format_ident!("__fabric_raw_{}", to_snake_case(system_name));
    let contract = input
        .contracts
        .iter()
        .find(|contract| contract.is_primary)
        .expect("validated primary contract");
    let contract_tokens = primary_contract_tokens(&sdk, contract, quote!(primary_contract_id()));
    let PrimaryContractTokens {
        service_name,
        contract_name,
        contract_id,
        service_methods,
        contract_methods,
        contract_key_expr,
    } = contract_tokens;
    let system_id = &input.system_id;
    let schema_expr = version_literal_expr(&sdk, &system_mod, &input.schema, SubjectKind::System);
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

    let realization_tokens = input
        .realization
        .as_ref()
        .map(|realization| realization_tokens(&sdk, realization, system_name));
    let realization_interface_tokens = input.realization.as_ref().map(|realization| {
        realization_interface_tokens(&sdk, realization, system_name, &system_mod)
    });
    let realization_field = input.realization.as_ref().map(|realization| {
        let field = format_ident!("{}", to_snake_case(&realization.name));
        let contract_name = format_ident!("{}Contract", realization.name);
        quote!(#field: #sdk::authoring::ContractDependency<#system_mod::realization::raw::#contract_name>,)
    });
    let realization_initializer = input.realization.as_ref().map(|realization| {
        let field = format_ident!("{}", to_snake_case(&realization.name));
        quote! {
            #field: #sdk::authoring::ContractDependency::new(
                #system_mod::realization::raw::requirement()
            ),
        }
    });
    let realization_declaration = input.realization.as_ref().map(|realization| {
        let field = format_ident!("{}", to_snake_case(&realization.name));
        quote!(declarations.push(self.#field.declaration().clone());)
    });
    let realization_requirement_declaration = input.realization.as_ref().map(|realization| {
        let _ = realization;
        quote!(required.push(#system_mod::realization::raw::requirement().declaration().clone());)
    });
    let realization_binding = input.realization.as_ref().map(|realization| {
        let field = format_ident!("{}", to_snake_case(&realization.name));
        quote! {
            self.#field
                .bind(bindings)
                .map_err(|error| #sdk::core::ModuleError::new(error.to_string()))?;
        }
    });
    let adaptable_impl = input.realization.as_ref().map(|realization| {
        let contract_name = format_ident!("{}Contract", realization.name);
        quote! {
            impl #sdk::authoring::AdaptableSystemDefinition for #system_name {
                type RealizationContract = #system_mod::realization::raw::#contract_name;

                fn realization_requirement() -> #sdk::core::ContractRequirement<Self::RealizationContract> {
                    #system_mod::realization::raw::requirement()
                }
            }
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

    quote! {
        #[derive(Clone)]
        #visibility struct #config_name {
            #(#config_fields)*
        }

        #visibility struct #system_name;

        impl #system_name {
            pub fn select(
                config: #config_name,
            ) -> ::std::result::Result<
                #sdk::authoring::SystemSelection<Self>,
                #sdk::system::SystemCompatibilityError,
            > {
                <Self as #sdk::authoring::SystemDefinition>::select(config)
            }
        }

        impl #sdk::authoring::PrimarySystemContract for #system_name {
            type Contract = #system_mod::raw::#contract_name;

            fn primary_contract_key() -> #sdk::core::ContractKey<Self::Contract> {
                #system_mod::raw::primary_contract_key()
            }
        }

        impl #sdk::authoring::SystemDefinition for #system_name {
            type Config = #config_name;

            fn system_id() -> #sdk::system::SystemId {
                #system_mod::raw::system_id()
            }

            fn schema() -> #sdk::system::SystemSchemaDescriptor {
                #schema_expr
            }

            fn declaration(
                selection: &#sdk::authoring::SystemSelection<Self>,
            ) -> #sdk::core::ModuleDeclaration {
                let mut required = ::std::vec![#(#declaration_requirements),*];
                #realization_requirement_declaration
                #sdk::core::ModuleDeclaration::new(selection.module_id().clone())
                    .with_provided_contracts(::std::vec![#system_mod::raw::primary_contract_key().declaration()])
                    .with_required_contracts(required)
            }

            fn materialize(
                selection: &#sdk::authoring::SystemSelection<Self>,
            ) -> ::std::option::Option<::std::boxed::Box<dyn #sdk::core::ModuleRuntime>> {
                ::std::option::Option::Some(::std::boxed::Box::new(#system_mod::raw::Runtime::new(
                    selection.module_id().clone(),
                    selection.config().clone(),
                )))
            }
        }

        #adaptable_impl

        #[allow(non_snake_case)]
        mod #raw_impl_mod {
            use super::*;

            pub fn system_id() -> #sdk::system::SystemId {
                #sdk::system::SystemId::new(#system_id)
                    .expect("system! generated a static system id")
            }

            pub fn primary_contract_id() -> #sdk::core::ContractId {
                #sdk::core::ContractId::new(#contract_id)
                    .expect("system! generated a static contract id")
            }

            pub fn primary_contract_key() -> #sdk::core::ContractKey<#contract_name> {
                #contract_key_expr
            }

            pub trait #service_name: Send + Sync {
                #(#service_methods)*
            }

            #[derive(Clone)]
            pub struct #contract_name {
                inner: ::std::sync::Arc<dyn #service_name>,
            }

            impl #contract_name {
                pub fn new(inner: ::std::sync::Arc<dyn #service_name>) -> Self {
                    Self { inner }
                }

                #(#contract_methods)*
            }

            #[derive(Clone)]
            pub struct Runtime {
                module_id: #sdk::core::ModuleId,
                config: #config_name,
                runtime_context: #sdk::authoring::RuntimeContext,
                #runtime_state_field
                #(#dependency_fields)*
                #realization_field
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
                        #realization_initializer
                    }
                }

                #(#runtime_inherent_methods)*
            }

            impl #service_name for Runtime {
                #(#runtime_trait_methods)*
            }

            impl #sdk::core::ModuleRuntime for Runtime {
                fn id(&self) -> &#sdk::core::ModuleId {
                    &self.module_id
                }

                fn provided_contract_declarations(
                    &self,
                ) -> ::std::vec::Vec<#sdk::core::ProvidedContractDeclaration> {
                    ::std::vec![primary_contract_key().declaration()]
                }

                fn required_contract_declarations(
                    &self,
                ) -> ::std::vec::Vec<#sdk::core::ContractRequirementDeclaration> {
                    let mut declarations = ::std::vec![
                        #(#dependency_declarations),*
                    ];
                    #realization_declaration
                    declarations
                }

                fn export_contracts(
                    &self,
                ) -> ::std::result::Result<
                    ::std::vec::Vec<#sdk::core::ModuleContract>,
                    #sdk::core::ModuleError,
                > {
                    let service = ::std::sync::Arc::new(self.clone())
                        as ::std::sync::Arc<dyn #service_name>;
                    Ok(::std::vec![#sdk::core::ModuleContract::new(
                        &primary_contract_key(),
                        ::std::sync::Arc::new(#contract_name::new(service)),
                    )])
                }

                fn bind(
                    &mut self,
                    bindings: &#sdk::core::ModuleBindings,
                ) -> ::std::result::Result<(), #sdk::core::ModuleError> {
                    #(#dependency_bindings)*
                    #realization_binding
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

        #visibility mod #system_mod {
            pub mod raw {
                pub use super::super::#raw_impl_mod::{
                    #contract_name, #service_name, Runtime,
                    primary_contract_id, primary_contract_key, system_id,
                };
            }

            #realization_tokens
        }

        #realization_interface_tokens
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
                    .expect("system! generated a static dependency version requirement"),
            )
        ),
    }
}

fn realization_tokens(
    sdk: &TokenStream,
    realization: &RealizationDefinition,
    system_name: &Ident,
) -> TokenStream {
    let realization_mod = format_ident!("realization");
    let raw_impl_mod = format_ident!("__fabric_realization_raw_{}", to_snake_case(system_name));
    let service_name = format_ident!("{}Service", realization.name);
    let contract_name = format_ident!("{}Contract", realization.name);
    let contract_id = &realization.contract_id;
    let compatibility_expr = requirement_literal_expr(sdk, &realization.compatibility, "system");
    let service_methods = realization
        .methods
        .iter()
        .map(super::common::service_method_tokens);
    let contract_methods = realization
        .methods
        .iter()
        .map(super::common::contract_wrapper_method_tokens);

    quote! {
        #[allow(non_snake_case)]
        mod #raw_impl_mod {
            // This helper is emitted directly inside the generated system
            // module. Its grandparent is therefore always the caller's
            // invocation scope, regardless of whether that scope is the
            // crate root or an arbitrarily nested Rust module.
            use super::super::*;

            pub fn contract_id() -> #sdk::core::ContractId {
                #sdk::core::ContractId::new(#contract_id)
                    .expect("system! generated a static realization contract id")
            }

            pub fn requirement() -> #sdk::core::ContractRequirement<#contract_name> {
                match #compatibility_expr {
                    #sdk::core::ContractCompatibilityRequirement::Provisional => {
                        #sdk::core::ContractRequirement::provisional(contract_id())
                    }
                    #sdk::core::ContractCompatibilityRequirement::Versioned(requirement) => {
                        #sdk::core::ContractRequirement::versioned(contract_id(), requirement)
                    }
                }
            }

            pub fn provisional_contract_key() -> #sdk::core::ContractKey<#contract_name> {
                #sdk::core::ContractKey::provisional(contract_id())
            }

            pub fn contract_key(
                version: #sdk::core::ContractVersion,
            ) -> #sdk::core::ContractKey<#contract_name> {
                #sdk::core::ContractKey::versioned(contract_id(), version)
            }

            pub trait #service_name: Send + Sync {
                #(#service_methods)*
            }

            #[derive(Clone)]
            pub struct #contract_name {
                inner: ::std::sync::Arc<dyn #service_name>,
            }

            impl #contract_name {
                pub fn new(inner: ::std::sync::Arc<dyn #service_name>) -> Self {
                    Self { inner }
                }

                #(#contract_methods)*
            }
        }

        pub mod #realization_mod {
            pub mod raw {
                pub use super::super::#raw_impl_mod::{
                    #contract_name, #service_name, contract_id, contract_key,
                    provisional_contract_key, requirement,
                };
                pub use super::super::#raw_impl_mod::{
                    #contract_name as Contract,
                    #service_name as Service,
                };
            }
        }
    }
}

fn realization_interface_tokens(
    sdk: &TokenStream,
    realization: &RealizationDefinition,
    system_name: &Ident,
    system_mod: &Ident,
) -> TokenStream {
    let interface_name = format_ident!("{}Realization", system_name);
    let contract_name = format_ident!("{}Contract", realization.name);
    let service_name = format_ident!("{}Service", realization.name);
    let service_methods = realization
        .methods
        .iter()
        .map(super::common::service_method_tokens);
    let forwarding_methods = realization.methods.iter().map(|method| {
        let signature = &method.signature;
        let name = &signature.ident;
        let args = method_call_args(signature);
        quote!(#signature { <T as #interface_name>::#name(self, #(#args),*) })
    });

    quote! {
        pub trait #interface_name: Send + Sync + 'static {
            #(#service_methods)*

            fn realization_target() -> ::std::marker::PhantomData<#system_name>
            where Self: Sized {
                ::std::marker::PhantomData
            }

            fn realization_contract_key(
                version: #sdk::core::ContractVersion,
            ) -> #sdk::core::ContractKey<#system_mod::realization::raw::#contract_name>
            where Self: Sized {
                #system_mod::realization::raw::contract_key(version)
            }

            fn provisional_realization_contract_key() -> #sdk::core::ContractKey<#system_mod::realization::raw::#contract_name>
            where Self: Sized {
                #system_mod::realization::raw::provisional_contract_key()
            }

            fn realization_contract(
                service: ::std::sync::Arc<Self>,
            ) -> #system_mod::realization::raw::#contract_name
            where Self: Sized {
                #system_mod::realization::raw::#contract_name::new(service)
            }
        }

        impl<T: #interface_name + ?Sized> #system_mod::realization::raw::#service_name for T {
            #(#forwarding_methods)*
        }
    }
}
