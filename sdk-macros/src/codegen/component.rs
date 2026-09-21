use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, PatType, parse_quote};

use crate::ast::{
    ComponentInput, ComponentOperationContext, ComponentOperationDefinition, RequirementDefinition,
    RequirementLiteral, SystemDependencyDefinition,
};

use super::common::{fabric_path, to_snake_case};

pub fn expand_component(input: &ComponentInput) -> TokenStream {
    let sdk = fabric_path();
    let visibility = &input.visibility;
    let component_name = &input.name;
    let config_name = format_ident!("{}Config", component_name);
    let dependencies_name = format_ident!("{}Dependencies", component_name);
    let component_mod = format_ident!("{}", to_snake_case(component_name));
    let component_id = &input.component_id;
    let config_fields = input.config_fields.iter().map(|field| {
        let name = &field.name;
        let ty = &field.ty;
        quote!(pub #name: #ty,)
    });
    let resource_requirements = input
        .requires
        .iter()
        .map(|requirement| {
            let field = &requirement.field;
            quote!(#component_mod::requirements::#field())
        })
        .collect::<Vec<_>>();
    let requirement_accessors = input
        .requires
        .iter()
        .map(|requirement| {
            let field = &requirement.field;
            let resource = &requirement.resource;
            let requirement = component_resource_requirement_tokens(&sdk, requirement);
            let name = field.to_string();
            quote! {
                pub fn #field() -> #sdk::authoring::ComponentResourceRequirement<#resource> {
                    #sdk::authoring::ComponentResourceRequirement::new(
                        #sdk::component::ComponentResourceRequirementName::new(#name)
                            .expect("component! generated a non-empty local requirement name"),
                        #requirement,
                    )
                }
            }
        })
        .collect::<Vec<_>>();
    let system_requirements = input
        .systems
        .iter()
        .map(|dependency| component_system_requirement_tokens(&sdk, dependency))
        .collect::<Vec<_>>();
    let dependency_fields = input.requires.iter().map(|requirement| {
        let field = &requirement.field;
        let resource = &requirement.resource;
        quote!(pub #field: ::std::sync::Arc<<#resource as #sdk::authoring::PrimaryResourceContract>::Contract>,)
    }).chain(input.systems.iter().map(|dependency| {
        let field = &dependency.field;
        let system = &dependency.system;
        quote!(pub #field: ::std::sync::Arc<<#system as #sdk::authoring::PrimarySystemContract>::Contract>,)
    })).collect::<Vec<_>>();
    let dependency_initializers = input.requires.iter().map(|requirement| {
        let field = &requirement.field;
        let field_accessor = &requirement.field;
        quote!(
            #field: <#sdk::component::ComponentRuntimeScope as #sdk::authoring::ComponentResourceScope>::named_resource(
                scope,
                &#component_mod::requirements::#field_accessor(),
            )?,
        )
    }).chain(input.systems.iter().map(|dependency| {
        let field = &dependency.field;
        let requirement_tokens = component_system_requirement_tokens(&sdk, dependency);
        quote!(
            #field: <#sdk::component::ComponentRuntimeScope as #sdk::authoring::ComponentSystemScope>::system(
                scope,
                &#requirement_tokens,
            )?,
        )
    })).collect::<Vec<_>>();
    let has_dependencies = !input.requires.is_empty() || !input.systems.is_empty();
    let operation_tokens = input
        .operations
        .iter()
        .map(|operation| operation_tokens(&sdk, operation))
        .collect::<Vec<_>>();
    let operation_names = input.operations.iter().map(|operation| &operation.name);
    let operation_registrations = input
        .operations
        .iter()
        .enumerate()
        .map(|(index, operation)| {
            let operation_name = &operation.name;
            let input_ty = &operation.input_ty;
            let has_context = operation.context == ComponentOperationContext::Invocation;
            let handler = component_handler_tokens(
                &operation.handler,
                &sdk,
                &dependencies_name,
                has_context,
                has_dependencies,
            );
            let config_binding = format_ident!("__fabric_component_config_{index}");
            let dependency_binding = format_ident!("__fabric_component_dependencies_{index}");
            let handler_call = match (has_context, has_dependencies) {
                (false, false) => quote!((#handler)(input)),
                (false, true) => quote! {
                    let dependencies: #dependencies_name = #dependency_binding.clone();
                    (#handler)(dependencies, input)
                },
                (true, false) => quote!((#handler)(context, input)),
                (true, true) => quote! {
                    let dependencies: #dependencies_name = #dependency_binding.clone();
                    (#handler)(context, dependencies, input)
                },
            };
            let dependency_capture = if has_dependencies {
                quote!(let #dependency_binding = dependencies.clone();)
            } else {
                quote!()
            };
            if has_context {
                quote! {
                    {
                        let #config_binding = config.clone();
                        #dependency_capture
                        scope.operation_with_context(
                            #component_mod::operations::#operation_name(),
                            move |context, input: #input_ty| {
                                let config = #config_binding.clone();
                                let _ = &config;
                                #handler_call
                            },
                        )?;
                    }
                }
            } else {
                quote! {
                    {
                        let #config_binding = config.clone();
                        #dependency_capture
                        scope.operation(
                            #component_mod::operations::#operation_name(),
                            move |input: #input_ty| {
                                let config = #config_binding.clone();
                                let _ = &config;
                                #handler_call
                            },
                        )?;
                    }
                }
            }
        });
    let self_realization = if let Some(teardown) = &input.teardown {
        quote! {
            #sdk::component::ComponentRuntimeDefinition::new_with_teardown(
                <Self as #sdk::authoring::ComponentDefinition>::component_id(),
                move |scope: &#sdk::component::ComponentRuntimeScope| -> ::std::result::Result<
                    #sdk::component::ComponentRuntimePreparation,
                    #sdk::component::ComponentError,
                > {
                    let dependencies = #dependencies_name {
                        #(#dependency_initializers)*
                    };
                    #(#operation_registrations)*
                    let teardown_scope = scope.clone();
                    let teardown_config = config.clone();
                    let teardown_dependencies = dependencies.clone();
                    ::std::result::Result::Ok(
                        #sdk::component::ComponentRuntimePreparation::with_teardown(
                            #sdk::core::Health::Healthy,
                            move || {
                                let scope = teardown_scope;
                                let config = teardown_config;
                                let dependencies = teardown_dependencies;
                                let _ = (&scope, &config, &dependencies);
                                #teardown
                            },
                        ),
                    )
                },
            )
        }
    } else {
        quote! {
            #sdk::component::ComponentRuntimeDefinition::new(
                <Self as #sdk::authoring::ComponentDefinition>::component_id(),
                move |scope: &#sdk::component::ComponentRuntimeScope| -> ::std::result::Result<
                    #sdk::core::Health,
                    #sdk::component::ComponentError,
                > {
                    let dependencies = #dependencies_name {
                        #(#dependency_initializers)*
                    };
                    #(#operation_registrations)*
                    ::std::result::Result::Ok(#sdk::core::Health::Healthy)
                },
            )
        }
    };

    quote! {
        #[derive(Clone)]
        #visibility struct #config_name {
            #(#config_fields)*
        }

        #[derive(Clone)]
        #visibility struct #dependencies_name {
            #(#dependency_fields)*
        }

        #visibility struct #component_name;

        impl #component_name {
            pub fn define(config: #config_name) -> #sdk::authoring::ComponentSpec<Self> {
                #sdk::authoring::ComponentSpec::<Self>::self_realizing(config)
                    .expect("component! generated a matching self realization")
                    #(.requires_named_resource(#resource_requirements))*
                    #(.requires_system(#system_requirements))*
            }
        }

        impl #sdk::authoring::ComponentDefinition for #component_name {
            type Config = #config_name;

            fn component_id() -> #sdk::component::ComponentId {
                #component_mod::component_id()
            }

            fn declaration() -> #sdk::component::ComponentDeclaration {
                #sdk::component::ComponentDeclaration::new(
                    #component_mod::component_id(),
                    ::std::vec![
                        #(#component_mod::operations::#operation_names().definition().clone()),*
                    ],
                )
            }

        }

        impl #sdk::authoring::SelfRealizingComponentDefinition for #component_name {
            fn self_realization(
                config: &Self::Config,
            ) -> #sdk::component::ComponentRuntimeDefinition {
                let config = config.clone();
                #self_realization
            }
        }

        #visibility mod #component_mod {
            use super::*;

            pub fn component_id() -> #sdk::component::ComponentId {
                #sdk::component::ComponentId::new(#component_id)
                    .expect("component! generated a static component id")
            }

            pub mod operations {
                use super::*;

                #(#operation_tokens)*
            }

            pub mod requirements {
                use super::*;
                #(#requirement_accessors)*
            }
        }
    }
}

fn component_handler_tokens(
    handler: &Expr,
    sdk: &TokenStream,
    dependencies_name: &syn::Ident,
    has_context: bool,
    has_dependencies: bool,
) -> TokenStream {
    let Expr::Closure(mut closure) = handler.clone() else {
        unreachable!("component handler validation requires a closure");
    };
    let mut index = 0;
    if has_context {
        type_handler_input(
            &mut closure,
            index,
            parse_quote!(#sdk::component::InvocationContext),
        );
        index += 1;
    }
    if has_dependencies {
        type_handler_input(&mut closure, index, parse_quote!(#dependencies_name));
    }
    quote!(#closure)
}

fn type_handler_input(closure: &mut syn::ExprClosure, index: usize, ty: syn::Type) {
    let binding = closure
        .inputs
        .iter()
        .nth(index)
        .expect("component handler validation requires a generated parameter")
        .clone();
    let typed_binding = syn::Pat::Type(PatType {
        attrs: Vec::new(),
        pat: Box::new(binding),
        colon_token: Default::default(),
        ty: Box::new(ty),
    });
    *closure
        .inputs
        .iter_mut()
        .nth(index)
        .expect("component handler validation requires a generated parameter") = typed_binding;
}

fn component_resource_requirement_tokens(
    sdk: &TokenStream,
    requirement: &RequirementDefinition,
) -> TokenStream {
    let resource = &requirement.resource;
    match &requirement.compatibility {
        RequirementLiteral::Provisional => {
            quote!(#sdk::authoring::Requires::<#resource>::provisional())
        }
        RequirementLiteral::Versioned(version) => quote!(
            #sdk::authoring::Requires::<#resource>::versioned(
                #sdk::core::ContractVersionRequirement::parse(#version)
                    .expect("component! generated a static Resource version requirement"),
            )
        ),
    }
}

fn component_system_requirement_tokens(
    sdk: &TokenStream,
    dependency: &SystemDependencyDefinition,
) -> TokenStream {
    let system = &dependency.system;
    match &dependency.compatibility {
        RequirementLiteral::Provisional => {
            quote!(#sdk::authoring::SystemRequires::<#system>::provisional())
        }
        RequirementLiteral::Versioned(version) => quote!(
            #sdk::authoring::SystemRequires::<#system>::versioned(
                #sdk::core::ContractVersionRequirement::parse(#version)
                    .expect("component! generated a static System version requirement"),
            )
        ),
    }
}

fn operation_tokens(sdk: &TokenStream, operation: &ComponentOperationDefinition) -> TokenStream {
    let operation_name = &operation.name;
    let operation_id = &operation.operation_id;
    let input_ty = &operation.input_ty;
    let input_type_id = &operation.input_type_id;
    let output_ty = &operation.output_ty;
    let output_type_id = &operation.output_type_id;
    let operation_id_fn = format_ident!("{}_id", operation_name);
    let input_type_id_fn = format_ident!("{}_input_type_id", operation_name);
    let output_type_id_fn = format_ident!("{}_output_type_id", operation_name);

    quote! {
        pub fn #operation_id_fn() -> #sdk::component::OperationId {
            #sdk::component::OperationId::new(#operation_id)
                .expect("component! generated a static operation id")
        }

        pub fn #input_type_id_fn() -> #sdk::component::OperationTypeId {
            #sdk::component::OperationTypeId::new(#input_type_id)
                .expect("component! generated a static input operation type id")
        }

        pub fn #output_type_id_fn() -> #sdk::component::OperationTypeId {
            #sdk::component::OperationTypeId::new(#output_type_id)
                .expect("component! generated a static output operation type id")
        }

        pub fn #operation_name() -> #sdk::component::OperationKey<#input_ty, #output_ty> {
            #sdk::component::OperationKey::new(
                #operation_id_fn(),
                #input_type_id_fn(),
                #output_type_id_fn(),
            )
        }
    }
}
