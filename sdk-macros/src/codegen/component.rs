use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, PatType, parse_quote};

use crate::ast::{
    ComponentInput, ComponentOperationContext, ComponentOperationDefinition, ContractMethod,
    RelationDefinition, RequirementDefinition, RequirementLiteral, SystemDependencyDefinition,
};

use super::common::{
    config_type_tokens, fabric_path, has_config, inline_config_definition_tokens, to_snake_case,
};

pub fn expand_component(input: &ComponentInput) -> TokenStream {
    let sdk = fabric_path();
    let visibility = &input.visibility;
    let component_name = &input.name;
    let config_name = format_ident!("{}Config", component_name);
    let dependencies_name = format_ident!("{}Dependencies", component_name);
    let component_mod = format_ident!("{}", to_snake_case(component_name));
    let component_id = &input.component_id;
    let legacy_self_realization = input.legacy_operations.is_some();
    // Preserve the old operations frontend's empty Config type temporarily.
    // Canonical declaration authoring never takes this branch.
    let config_ty = if legacy_self_realization && !has_config(&input.config) {
        quote!(#config_name)
    } else {
        config_type_tokens(&input.config, &config_name)
    };
    let config_definition = if legacy_self_realization && !has_config(&input.config) {
        quote!(#[derive(Clone)] #visibility struct #config_name {})
    } else {
        inline_config_definition_tokens(&input.config, visibility, &config_name)
    };
    let legacy_operations = input.legacy_operations.as_deref().unwrap_or(&[]);
    let canonical_operations = input
        .api
        .as_ref()
        .map(|api| api.methods.as_slice())
        .unwrap_or(&[]);
    let operation_tokens = if input.api.is_some() {
        canonical_operations
            .iter()
            .map(|method| canonical_operation_tokens(&sdk, &input.component_id, method))
            .collect::<Vec<_>>()
    } else {
        legacy_operations
            .iter()
            .map(|operation| operation_tokens(&sdk, operation))
            .collect::<Vec<_>>()
    };
    let operation_names = if input.api.is_some() {
        canonical_operations
            .iter()
            .map(|method| method.signature.ident.clone())
            .collect::<Vec<_>>()
    } else {
        legacy_operations
            .iter()
            .map(|operation| operation.name.clone())
            .collect::<Vec<_>>()
    };
    let relation_declarations = input.relations.iter().map(|relation| {
        let field = &relation.field;
        let requirement = relation_requirement_tokens(&sdk, relation);
        let name = field.to_string();
        quote!(#sdk::component::ComponentRelationDeclaration::new(
            #sdk::component::ComponentRelationName::new(#name)
                .expect("component! generated a non-empty relation role"),
            #requirement.declaration().clone(),
        ))
    });
    let declaration_relations = if input.relations.is_empty() {
        quote!()
    } else {
        quote!(.with_relations(::std::vec![#(#relation_declarations),*]))
    };
    let resource_requirements = input
        .legacy_requires
        .iter()
        .map(|requirement| {
            let field = &requirement.field;
            quote!(#component_mod::requirements::#field())
        })
        .collect::<Vec<_>>();
    let requirement_accessors = input
        .legacy_requires
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
        .legacy_systems
        .iter()
        .map(|dependency| component_system_requirement_tokens(&sdk, dependency))
        .collect::<Vec<_>>();
    let dependency_fields = input.legacy_requires.iter().map(|requirement| {
        let field = &requirement.field;
        let resource = &requirement.resource;
        quote!(pub #field: ::std::sync::Arc<<#resource as #sdk::authoring::PrimaryResourceContract>::Contract>,)
    }).chain(input.legacy_systems.iter().map(|dependency| {
        let field = &dependency.field;
        let system = &dependency.system;
        quote!(pub #field: ::std::sync::Arc<<#system as #sdk::authoring::PrimarySystemContract>::Contract>,)
    })).collect::<Vec<_>>();
    let dependency_initializers = input.legacy_requires.iter().map(|requirement| {
        let field = &requirement.field;
        let field_accessor = &requirement.field;
        quote!(
            #field: <#sdk::component::ComponentRuntimeScope as #sdk::authoring::ComponentResourceScope>::named_resource(
                scope,
                &#component_mod::requirements::#field_accessor(),
            )?,
        )
    }).chain(input.legacy_systems.iter().map(|dependency| {
        let field = &dependency.field;
        let requirement_tokens = component_system_requirement_tokens(&sdk, dependency);
        quote!(
            #field: <#sdk::component::ComponentRuntimeScope as #sdk::authoring::ComponentSystemScope>::system(
                scope,
                &#requirement_tokens,
            )?,
        )
    })).collect::<Vec<_>>();
    let has_dependencies = !input.legacy_requires.is_empty() || !input.legacy_systems.is_empty();
    let operation_registrations = legacy_operations
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

    let define_method = if legacy_self_realization {
        quote! {
            pub fn define(config: #config_ty) -> #sdk::authoring::ComponentSpec<Self> {
                Self::self_realizing(config)
                    .expect("component! generated a matching self realization")
                    #(.requires_named_resource(#resource_requirements))*
                    #(.requires_system(#system_requirements))*
            }
        }
    } else if has_config(&input.config) {
        quote!(pub fn define(config: #config_ty) -> #sdk::authoring::ComponentSpec<Self> { #sdk::authoring::ComponentSpec::<Self>::new(config) })
    } else {
        quote!(pub fn define() -> #sdk::authoring::ComponentSpec<Self> { #sdk::authoring::ComponentSpec::<Self>::new(()) })
    };
    let self_realizing_impl = legacy_self_realization.then(|| quote! {
        impl #sdk::authoring::SelfRealizingComponentDefinition for #component_name {
            fn self_realization(config: &Self::Config) -> #sdk::component::ComponentRuntimeDefinition {
                let config = config.clone();
                #self_realization
            }
        }
    });
    let legacy_define = legacy_self_realization.then(|| quote! {
        pub fn self_realizing(config: #config_ty) -> ::std::result::Result<#sdk::authoring::ComponentSpec<Self>, #sdk::component::ComponentError> {
            #sdk::authoring::ComponentSpec::<Self>::self_realizing(config)
        }
    });

    quote! {
        #config_definition

        #[derive(Clone)]
        #visibility struct #dependencies_name {
            #(#dependency_fields)*
        }

        #visibility struct #component_name;

        impl #component_name {
            #define_method
            #legacy_define
        }

        impl #sdk::authoring::ComponentDefinition for #component_name {
            type Config = #config_ty;

            fn component_id() -> #sdk::component::ComponentId {
                #component_mod::component_id()
            }

            fn declaration() -> #sdk::component::ComponentDeclaration {
                #sdk::component::ComponentDeclaration::new(
                    #component_mod::component_id(),
                    ::std::vec![
                        #(#component_mod::operations::#operation_names().definition().clone()),*
                    ],
                ) #declaration_relations
            }

        }

        #self_realizing_impl

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

/// Lowers the shared declaration-only `api` grammar into the existing
/// Component invocation representation. Operation and slot identities are
/// deterministic lowering details; authors declare neither.
fn canonical_operation_tokens(
    sdk: &TokenStream,
    component_id: &syn::LitStr,
    method: &ContractMethod,
) -> TokenStream {
    let method_name = &method.signature.ident;
    let operation_id_fn = format_ident!("{}_id", method_name);
    let input_type_id_fn = format_ident!("{}_input_type_id", method_name);
    let output_type_id_fn = format_ident!("{}_output_type_id", method_name);
    let argument_types = method
        .signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(argument) => Some(&argument.ty),
        })
        .collect::<Vec<_>>();
    let input_ty = match argument_types.as_slice() {
        [] => quote!(()),
        [only] => quote!(#only),
        many => quote!((#(#many),*)),
    };
    let output_ty = match &method.signature.output {
        syn::ReturnType::Default => quote!(()),
        syn::ReturnType::Type(_, ty) => quote!(#ty),
    };
    let operation_id = format!("{}.api.{}", component_id.value(), method_name);
    let input_type_id = format!("{}.input", operation_id);
    let output_type_id = format!("{}.output", operation_id);
    quote! {
        pub fn #operation_id_fn() -> #sdk::component::OperationId {
            #sdk::component::OperationId::new(#operation_id)
                .expect("component! generated a deterministic API operation id")
        }
        pub fn #input_type_id_fn() -> #sdk::component::OperationTypeId {
            #sdk::component::OperationTypeId::new(#input_type_id)
                .expect("component! generated a deterministic API input slot id")
        }
        pub fn #output_type_id_fn() -> #sdk::component::OperationTypeId {
            #sdk::component::OperationTypeId::new(#output_type_id)
                .expect("component! generated a deterministic API output slot id")
        }
        pub fn #method_name() -> #sdk::component::OperationKey<#input_ty, #output_ty> {
            #sdk::component::OperationKey::new(
                #operation_id_fn(),
                #input_type_id_fn(),
                #output_type_id_fn(),
            )
        }
    }
}

fn relation_requirement_tokens(sdk: &TokenStream, relation: &RelationDefinition) -> TokenStream {
    let target = &relation.target;
    match &relation.compatibility {
        None => quote!(<#target as #sdk::authoring::RelationTarget>::relation_requirement()),
        Some(RequirementLiteral::Provisional) => quote!(
            <#target as #sdk::authoring::RelationTarget>::relation_requirement()
        ),
        Some(RequirementLiteral::Versioned(version)) => quote!(
            <#target as #sdk::authoring::RelationTarget>::relation_requirement_versioned(
                #sdk::core::ContractVersionRequirement::parse(#version)
                    .expect("component! generated a static relation version requirement"),
            )
        ),
    }
}
