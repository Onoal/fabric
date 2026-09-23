use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::FnArg;

use crate::ast::{ComponentInput, ContractMethod, RelationDefinition, RequirementLiteral};

use super::common::{
    config_type_tokens, fabric_path, has_config, inline_config_definition_tokens, method_call_args,
    to_snake_case,
};

pub fn expand_component(input: &ComponentInput) -> TokenStream {
    let sdk = fabric_path();
    let visibility = &input.visibility;
    let component_name = &input.name;
    let config_name = format_ident!("{}Config", component_name);
    let component_mod = format_ident!("{}", to_snake_case(component_name));
    let endpoint_module = format_ident!("api");
    let component_id = &input.component_id;
    let canonical_runtime = input.runtime.as_ref();
    let config_ty = config_type_tokens(&input.config, &config_name);
    let config_definition =
        inline_config_definition_tokens(&input.config, visibility, &config_name);
    let canonical_operations = input
        .api
        .as_ref()
        .map(|api| api.methods.as_slice())
        .unwrap_or(&[]);
    let operation_tokens = canonical_operations
        .iter()
        .map(|method| canonical_operation_tokens(&sdk, &input.component_id, method))
        .collect::<Vec<_>>();
    let operation_names = canonical_operations
        .iter()
        .map(|method| method.signature.ident.clone())
        .collect::<Vec<_>>();
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
    let canonical_relation_spec_additions = input
        .relations
        .iter()
        .map(|relation| {
            let target = &relation.target;
            let compatibility = component_relation_compatibility_tokens(&sdk, relation);
            let name = relation.field.to_string();
            quote!(
                .requires_relation::<#target>(
                    #sdk::component::ComponentRelationName::new(#name)
                        .expect("component! generated a non-empty relation role"),
                    #compatibility,
                )
            )
        })
        .collect::<Vec<_>>();
    let canonical_relations_name = format_ident!("{}Relations", component_name);
    let canonical_relation_fields = input.relations.iter().map(|relation| {
        let field = &relation.field;
        let target = &relation.target;
        quote!(pub #field: ::std::sync::Arc<<#target as #sdk::authoring::RelationTarget>::Contract>,)
    }).collect::<Vec<_>>();
    let canonical_relation_initializers = input.relations.iter().map(|relation| {
        let field = &relation.field;
        let target = &relation.target;
        let compatibility = component_relation_compatibility_tokens(&sdk, relation);
        let name = relation.field.to_string();
        quote!(
            #field: <#target as #sdk::authoring::ComponentRelationTarget>::resolve_component_relation(
                scope,
                &#sdk::component::ComponentRelationName::new(#name)
                    .expect("component! generated a non-empty relation role"),
                &#compatibility,
            )?,
        )
    }).collect::<Vec<_>>();
    let canonical_self_realization = canonical_runtime.map(|runtime| {
        let runtime_name = format_ident!("__Fabric{}Runtime", component_name);
        let runtime_methods = runtime.methods.iter().map(|method| {
            let signature = &method.signature;
            let body = &method.body;
            quote!(#signature #body)
        }).collect::<Vec<_>>();
        let state_field = runtime.state.as_ref().map(|state| {
            let ty = &state.ty;
            quote!(state: #sdk::authoring::RuntimeState<#ty>,)
        });
        let state_initializer = runtime.state.as_ref().map(|state| {
            let initializer = &state.initializer;
            quote! {
                let state = #sdk::authoring::RuntimeState::new({
                    let config = &config;
                    #initializer
                });
            }
        });
        let state_value = runtime.state.as_ref().map(|_| quote!(state,));
        let state_accessor = runtime.state.as_ref().map(|state| {
            let ty = &state.ty;
            quote!(pub fn state(&self) -> &#sdk::authoring::RuntimeState<#ty> { &self.state })
        });
        let prepare_body = runtime.prepare.as_ref().map(|body| {
            let statements = &body.stmts;
            quote!(#(#statements)*)
        }).unwrap_or_else(|| quote!(Ok(())));
        let teardown_body = runtime.teardown.as_ref().map(|body| {
            let statements = &body.stmts;
            quote!(#(#statements)*)
        }).unwrap_or_else(|| quote!(Ok(())));
        let operation_registrations = input.api.as_ref().map(|api| {
            api.methods.iter().map(|method| {
                let signature = &method.signature;
                let name = &signature.ident;
                let operation = quote!(#component_mod::#endpoint_module::#name());
                let arguments = method_call_args(signature);
                let argument_types = signature.inputs.iter().filter_map(|input| match input {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(argument) => Some(&argument.ty),
                }).collect::<Vec<_>>();
                match argument_types.as_slice() {
                    [] => quote! {
                        {
                            let runtime = ::std::sync::Arc::clone(&runtime);
                            scope.operation(#operation, move |(): ()| {
                                let runtime = ::std::sync::Arc::clone(&runtime);
                                async move { Ok(runtime.#name()) }
                            })?;
                        }
                    },
                    [only] => quote! {
                        {
                            let runtime = ::std::sync::Arc::clone(&runtime);
                            scope.operation(#operation, move |input: #only| {
                                let runtime = ::std::sync::Arc::clone(&runtime);
                                async move { Ok(runtime.#name(input)) }
                            })?;
                        }
                    },
                    many => quote! {
                        {
                            let runtime = ::std::sync::Arc::clone(&runtime);
                            scope.operation(#operation, move |input: (#(#many),*)| {
                                let runtime = ::std::sync::Arc::clone(&runtime);
                                async move {
                                    let (#(#arguments),*) = input;
                                    Ok(runtime.#name(#(#arguments),*))
                                }
                            })?;
                        }
                    },
                }
            }).collect::<Vec<_>>()
        }).unwrap_or_default();
        quote! {
            #[derive(Clone)]
            struct #runtime_name {
                config: #config_ty,
                relations: #canonical_relations_name,
                #state_field
            }

            impl #runtime_name {
                pub fn config(&self) -> &#config_ty { &self.config }
                pub fn relations(&self) -> &#canonical_relations_name { &self.relations }
                #state_accessor
                #(#runtime_methods)*
                fn __fabric_prepare(&self) -> ::std::result::Result<(), #sdk::component::ComponentError> {
                    #prepare_body
                }
                fn __fabric_teardown(&self) -> ::std::result::Result<(), #sdk::component::ComponentError> {
                    #teardown_body
                }
            }

            impl #sdk::authoring::SelfRealizingComponentDefinition for #component_name {
                fn self_realization(config: &Self::Config) -> #sdk::component::ComponentParticipationRealization {
                    let config = config.clone();
                    #sdk::component::ComponentParticipationRealization::new_with_teardown(
                        <Self as #sdk::authoring::ComponentDefinition>::component_id(),
                        move |scope: &#sdk::component::ComponentParticipationScope| -> ::std::result::Result<
                            #sdk::component::ComponentParticipationPreparation,
                            #sdk::component::ComponentError,
                        > {
                            let relations = #canonical_relations_name {
                                #(#canonical_relation_initializers)*
                            };
                            #state_initializer
                            let runtime = ::std::sync::Arc::new(#runtime_name {
                                config: config.clone(),
                                relations,
                                #state_value
                            });
                            #(#operation_registrations)*
                            runtime.__fabric_prepare()?;
                            let teardown_runtime = ::std::sync::Arc::clone(&runtime);
                            Ok(#sdk::component::ComponentParticipationPreparation::with_teardown(
                                #sdk::core::Health::Healthy,
                                move || teardown_runtime.__fabric_teardown(),
                            ))
                        },
                    )
                }
            }
        }
    });
    let component_adapter_methods = input
        .api
        .as_ref()
        .map(|api| api.methods.as_slice())
        .unwrap_or(&[]);
    let component_adapter_bridge = component_adapter_bridge_tokens(
        &sdk,
        component_name,
        &component_mod,
        &endpoint_module,
        component_adapter_methods,
    );
    let component_adapter_builder_method = quote!(
        #[doc(hidden)]
        pub fn __fabric_canonical_adapter_builder() -> #component_mod::ComponentAdapterBuilder {
            #component_mod::ComponentAdapterBuilder::new()
        }
    );
    let component_realization_contract_id = quote!(#sdk::core::ContractId::new(format!("{}.realization", #component_id)).expect("component! generated a static realization contract id"));
    let adaptable_component_impl = quote!(
        impl #sdk::authoring::AdaptableComponentDefinition for #component_name {
            fn realization_requirement() -> #sdk::core::ContractRequirement<#sdk::authoring::ComponentRealizationContract<Self>> {
                #sdk::core::ContractRequirement::provisional(#component_realization_contract_id)
            }
        }
    );

    let define_method = if canonical_runtime.is_some() && has_config(&input.config) {
        quote!(pub fn define(config: #config_ty) -> #sdk::authoring::ComponentSpec<Self> {
            #sdk::authoring::ComponentSpec::<Self>::self_realizing(config)
                .expect("component! generated a matching self realization")
                #(#canonical_relation_spec_additions)*
        })
    } else if canonical_runtime.is_some() {
        quote!(pub fn define() -> #sdk::authoring::ComponentSpec<Self> {
            #sdk::authoring::ComponentSpec::<Self>::self_realizing(())
                .expect("component! generated a matching self realization")
                #(#canonical_relation_spec_additions)*
        })
    } else if has_config(&input.config) {
        quote!(pub fn define(config: #config_ty) -> #sdk::authoring::ComponentSpec<Self> {
            #sdk::authoring::ComponentSpec::<Self>::new(config)
                #(#canonical_relation_spec_additions)*
        })
    } else {
        quote!(pub fn define() -> #sdk::authoring::ComponentSpec<Self> {
            #sdk::authoring::ComponentSpec::<Self>::new(())
                #(#canonical_relation_spec_additions)*
        })
    };
    quote! {
        #config_definition

        #[derive(Clone)]
        #visibility struct #canonical_relations_name {
            #(#canonical_relation_fields)*
        }

        #visibility struct #component_name;

        impl #component_name {
            #define_method
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
                        #(#component_mod::#endpoint_module::#operation_names().definition().clone()),*
                    ],
                ) #declaration_relations
            }

        }

        impl #sdk::authoring::ComponentAdapterTarget for #component_name {
            type ComponentConfig = #config_ty;
            type ComponentRelations = #canonical_relations_name;

            fn component_adapter_context(
                config: &Self::ComponentConfig,
                scope: &#sdk::component::ComponentParticipationScope,
            ) -> ::std::result::Result<(Self::ComponentConfig, Self::ComponentRelations), #sdk::component::ComponentError> {
                Ok((config.clone(), #canonical_relations_name { #(#canonical_relation_initializers)* }))
            }

            fn is_component_participation_target() -> bool { true }
        }

        #adaptable_component_impl

        impl #component_name {
            #[doc(hidden)]
            pub fn __fabric_canonical_adapter_contract_key() -> #sdk::core::ContractKey<#sdk::authoring::ComponentRealizationContract<Self>> {
                #sdk::core::ContractKey::provisional(#component_realization_contract_id)
            }

            #[doc(hidden)]
            pub fn __fabric_canonical_adapter_bridge_mode() -> #sdk::authoring::AdapterBridgeMode {
                #sdk::authoring::AdapterBridgeMode::ExplicitContract
            }

            #component_adapter_builder_method
        }

        #canonical_self_realization

        #visibility mod #component_mod {
            use super::*;

            pub fn component_id() -> #sdk::component::ComponentId {
                #sdk::component::ComponentId::new(#component_id)
                    .expect("component! generated a static component id")
            }

            pub mod #endpoint_module {
                use super::*;

                #(#operation_tokens)*
            }

            #component_adapter_bridge
        }
    }
}

fn component_adapter_bridge_tokens(
    sdk: &TokenStream,
    component: &syn::Ident,
    component_mod: &syn::Ident,
    endpoint_module: &syn::Ident,
    methods: &[ContractMethod],
) -> TokenStream {
    let method_names = methods
        .iter()
        .map(|method| method.signature.ident.clone())
        .collect::<Vec<_>>();
    let function_names = method_names
        .iter()
        .map(|name| format_ident!("F{}", upper_camel(name)))
        .collect::<Vec<_>>();
    let missing_names = method_names
        .iter()
        .map(|name| format_ident!("ComponentAdapterMissingApiMethod{}", upper_camel(name)))
        .collect::<Vec<_>>();
    let builder_ty = |types: &[TokenStream]| {
        if types.is_empty() {
            quote!(ComponentAdapterBuilder)
        } else {
            quote!(ComponentAdapterBuilder<#(#types),*>)
        }
    };
    let builder_type = builder_ty(
        &function_names
            .iter()
            .map(|name| quote!(#name))
            .collect::<Vec<_>>(),
    );
    let missing_type = builder_ty(
        &missing_names
            .iter()
            .map(|name| quote!(#name))
            .collect::<Vec<_>>(),
    );
    let builder_generics =
        (!function_names.is_empty()).then(|| quote!(<#(#function_names = #missing_names),*>));
    let builder_impl_generics =
        (!function_names.is_empty()).then(|| quote!(<#(#function_names),*>));
    let fields = method_names
        .iter()
        .zip(function_names.iter())
        .map(|(name, ty)| quote!(#name: #ty,))
        .collect::<Vec<_>>();
    let missing_fields = method_names
        .iter()
        .zip(missing_names.iter())
        .map(|(name, ty)| quote!(#name: #ty,))
        .collect::<Vec<_>>();
    let setters = method_names.iter().enumerate().map(|(index, name)| {
        let replacement = function_names.iter().enumerate().map(|(candidate, ty)| if candidate == index { quote!(Next) } else { quote!(#ty) }).collect::<Vec<_>>();
        let ret = builder_ty(&replacement);
        let values = method_names.iter().map(|field| if field == name { quote!(#field: #field,) } else { quote!(#field: self.#field,) });
        quote!(pub fn #name<Next>(self, #name: Next) -> #ret { ComponentAdapterBuilder { #(#values)* } })
    }).collect::<Vec<_>>();
    let bounds = methods.iter().zip(function_names.iter()).map(|(method, function)| {
        let inputs = method.signature.inputs.iter().filter_map(|arg| match arg { FnArg::Receiver(_) => None, FnArg::Typed(arg) => Some(&arg.ty) }).collect::<Vec<_>>();
        let output = match &method.signature.output { syn::ReturnType::Default => quote!(()), syn::ReturnType::Type(_, ty) => quote!(#ty) };
        quote!(#function: ::std::ops::Fn(&R::ParticipationRuntime, #(#inputs),*) -> #output + Send + Sync + 'static,)
    }).collect::<Vec<_>>();
    let registrations = methods.iter().map(|method| {
        let signature = &method.signature;
        let name = &signature.ident;
        let args = method_call_args(signature);
        let arg_types = signature.inputs.iter().filter_map(|arg| match arg { FnArg::Receiver(_) => None, FnArg::Typed(arg) => Some(&arg.ty) }).collect::<Vec<_>>();
        match arg_types.as_slice() {
            [] => quote!({ let runtime = ::std::sync::Arc::clone(&runtime); let call = ::std::sync::Arc::clone(&#name); scope.operation(#component_mod::#endpoint_module::#name(), move |(): ()| { let runtime = ::std::sync::Arc::clone(&runtime); let call = ::std::sync::Arc::clone(&call); async move { Ok(call(&runtime)) } })?; }),
            [only] => quote!({ let runtime = ::std::sync::Arc::clone(&runtime); let call = ::std::sync::Arc::clone(&#name); scope.operation(#component_mod::#endpoint_module::#name(), move |input: #only| { let runtime = ::std::sync::Arc::clone(&runtime); let call = ::std::sync::Arc::clone(&call); async move { Ok(call(&runtime, input)) } })?; }),
            many => quote!({ let runtime = ::std::sync::Arc::clone(&runtime); let call = ::std::sync::Arc::clone(&#name); scope.operation(#component_mod::#endpoint_module::#name(), move |input: (#(#many),*)| { let runtime = ::std::sync::Arc::clone(&runtime); let call = ::std::sync::Arc::clone(&call); async move { let (#(#args),*) = input; Ok(call(&runtime, #(#args),*)) } })?; }),
        }
    }).collect::<Vec<_>>();
    quote! {
        #[doc(hidden)] pub struct ComponentAdapterBuilder #builder_generics { #(#fields)* }
        #(#[doc(hidden)] pub struct #missing_names;)*
        impl #missing_type { #[doc(hidden)] pub fn new() -> Self { Self { #(#missing_fields)* } } }
        impl #builder_impl_generics #builder_type { #(#setters)*
            #[doc(hidden)] pub fn build<R>(self, provider: ::std::sync::Arc<R>) -> #sdk::authoring::ComponentRealizationContract<#component>
            where R: #sdk::authoring::CanonicalComponentAdapterRuntime<#component>, #(#bounds)* {
                let ComponentAdapterBuilder { #(#method_names),* } = self;
                #(let #method_names = ::std::sync::Arc::new(#method_names);)*
                #sdk::authoring::ComponentRealizationContract::new_with_teardown(move |config, scope| {
                    let (component_config, component_relations) = <#component as #sdk::authoring::ComponentAdapterTarget>::component_adapter_context(config, scope)?;
                    let runtime = ::std::sync::Arc::new(provider.prepare_component_runtime(component_config, component_relations)?);
                    #(#registrations)*
                    provider.component_prepare(&runtime)?;
                    let provider = ::std::sync::Arc::clone(&provider); let teardown_runtime = ::std::sync::Arc::clone(&runtime);
                    Ok(#sdk::component::ComponentParticipationPreparation::with_teardown(#sdk::core::Health::Healthy, move || provider.component_teardown(&teardown_runtime)))
                })
            }
        }
    }
}

fn upper_camel(identifier: &syn::Ident) -> String {
    let mut upper = true;
    identifier
        .to_string()
        .chars()
        .filter_map(|ch| {
            if ch == '_' {
                upper = true;
                None
            } else if upper {
                upper = false;
                Some(ch.to_ascii_uppercase())
            } else {
                Some(ch)
            }
        })
        .collect()
}

/// Lowers the shared declaration-only `api` grammar into the existing
/// ComponentInstanceBinding invocation representation. Operation and slot identities are
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

fn component_relation_compatibility_tokens(
    sdk: &TokenStream,
    relation: &RelationDefinition,
) -> TokenStream {
    match &relation.compatibility {
        None | Some(RequirementLiteral::Provisional) => {
            quote!(#sdk::authoring::ComponentRelationCompatibility::Provisional)
        }
        Some(RequirementLiteral::Versioned(version)) => quote!(
            #sdk::authoring::ComponentRelationCompatibility::Versioned(
                #sdk::core::ContractVersionRequirement::parse(#version)
                    .expect("component! generated a static relation version requirement"),
            )
        ),
    }
}
