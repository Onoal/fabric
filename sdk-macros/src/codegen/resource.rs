use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::ast::{
    ApiDefinition, ContractMethod, DifferentialRealizationDefinition, RelationDefinition,
    ResourceInput,
};

use super::common::{
    CanonicalAdapterBridgeTokens, PrimaryContractTokens, SubjectKind, api_contract_id_expr,
    canonical_adapter_bridge_tokens, canonical_adapter_bridge_tokens_named, config_type_tokens,
    fabric_path, has_config, inline_config_definition_tokens, method_call_args,
    primary_contract_tokens, runtime_method_tokens, to_snake_case, version_literal_expr,
};

pub fn expand_resource(input: &ResourceInput) -> TokenStream {
    let sdk = fabric_path();
    let visibility = &input.visibility;
    let resource_name = &input.name;
    let config_name = format_ident!("{}Config", resource_name);
    let config_ty = config_type_tokens(&input.config, &config_name);
    let config_definition =
        inline_config_definition_tokens(&input.config, visibility, &config_name);
    let select_method = if has_config(&input.config) {
        quote! {
            pub fn select(
                name: impl #sdk::authoring::IntoResourceName,
                config: #config_ty,
            ) -> ::std::result::Result<
                #sdk::authoring::ResourceSelection<Self>,
                #sdk::resource::ResourceError,
            > {
                <Self as #sdk::authoring::ResourceDefinition>::select(name, config)
            }
        }
    } else {
        quote! {
            pub fn select(
                name: impl #sdk::authoring::IntoResourceName,
            ) -> ::std::result::Result<
                #sdk::authoring::ResourceSelection<Self>,
                #sdk::resource::ResourceError,
            > {
                <Self as #sdk::authoring::ResourceDefinition>::select(name, ())
            }
        }
    };
    let resource_mod = format_ident!("{}", to_snake_case(resource_name));
    let raw_impl_mod = format_ident!("__fabric_raw_{}", to_snake_case(resource_name));
    let api = &input.api;
    let contract_tokens = primary_contract_tokens(&sdk, api, quote!(primary_contract_id()));
    let PrimaryContractTokens {
        service_name,
        contract_name,
        service_methods,
        contract_methods,
        contract_key_expr,
    } = contract_tokens;
    let CanonicalAdapterBridgeTokens {
        definition: canonical_adapter_bridge_definition,
        builder_name: canonical_adapter_builder_name,
    } = canonical_adapter_bridge_tokens(api, &service_name, &contract_name);
    let differential = input.differential_realization.as_ref();
    let effective_api = differential.map(|differential| effective_api(api, differential));
    let effective_contract_tokens = effective_api.as_ref().map(|effective_api| {
        primary_contract_tokens(
            &sdk,
            effective_api,
            quote!(effective_realization_contract_id()),
        )
    });
    let differential_contract_definition = effective_contract_tokens.as_ref().map(|tokens| {
        let effective_service = &tokens.service_name;
        let effective_contract = &tokens.contract_name;
        let effective_service_methods = &tokens.service_methods;
        let effective_contract_methods = &tokens.contract_methods;
        quote! {
            pub trait #effective_service: Send + Sync { #(#effective_service_methods)* }
            #[derive(Clone)]
            pub struct #effective_contract { inner: ::std::sync::Arc<dyn #effective_service> }
            impl #effective_contract {
                pub fn new(inner: ::std::sync::Arc<dyn #effective_service>) -> Self { Self { inner } }
                #(#effective_contract_methods)*
            }
        }
    });
    let differential_effective_key_definition = effective_contract_tokens.as_ref().map(|tokens| {
        let contract = &tokens.contract_name;
        let key = &tokens.contract_key_expr;
        quote! {
            pub fn effective_realization_contract_key() -> #sdk::core::ContractKey<#contract> {
                #key
            }
        }
    });
    let differential_bridge_definition = effective_contract_tokens.as_ref().map(|tokens| {
        let effective_api = effective_api
            .as_ref()
            .expect("effective API accompanies tokens");
        canonical_adapter_bridge_tokens_named(
            effective_api,
            &tokens.service_name,
            &tokens.contract_name,
            "Effective",
        )
        .definition
    });
    let differential_builder_name = effective_contract_tokens.as_ref().map(|tokens| {
        let effective_api = effective_api
            .as_ref()
            .expect("effective API accompanies tokens");
        canonical_adapter_bridge_tokens_named(
            effective_api,
            &tokens.service_name,
            &tokens.contract_name,
            "Effective",
        )
        .builder_name
    });
    let target_builder_name = differential_builder_name
        .as_ref()
        .unwrap_or(&canonical_adapter_builder_name);
    let target_builder_type = quote!(#resource_mod::raw::#target_builder_name);
    let target_contract_type = if let Some(tokens) = effective_contract_tokens.as_ref() {
        let name = &tokens.contract_name;
        quote!(#resource_mod::raw::#name)
    } else {
        quote!(#resource_mod::raw::#contract_name)
    };
    let target_contract_key = if let Some(_tokens) = effective_contract_tokens.as_ref() {
        quote!(#resource_mod::raw::effective_realization_contract_key())
    } else {
        quote!(#resource_mod::raw::primary_contract_key())
    };
    let target_bridge_mode = if differential.is_some() {
        quote!(#sdk::authoring::AdapterBridgeMode::DifferentialSemanticApi)
    } else {
        quote!(#sdk::authoring::AdapterBridgeMode::SemanticApi)
    };
    let differential_raw_reexports = effective_contract_tokens.as_ref().map(|tokens| {
        let contract = &tokens.contract_name;
        let service = &tokens.service_name;
        let builder = differential_builder_name
            .as_ref()
            .expect("differential builder");
        quote!(#builder, #contract, #service, effective_realization_contract_key,)
    });
    let resource_id = &input.resource_id;
    let api_contract_id = api_contract_id_expr(&sdk, resource_id, SubjectKind::Resource);
    let schema_expr =
        version_literal_expr(&sdk, &resource_mod, &input.schema, SubjectKind::Resource);
    let dependency_fields = input.relations.iter().map(|requirement| {
        let field = &requirement.field;
        let contract_ty = requirement_contract_type(&sdk, requirement);
        quote!(#field: #sdk::authoring::ContractDependency<#contract_ty>,)
    });
    let dependency_initializers = input.relations.iter().map(|requirement| {
        let field = &requirement.field;
        let requirement_expr = resource_requirement_expr(&sdk, requirement);
        quote! {
            #field: #sdk::authoring::ContractDependency::new(
                #requirement_expr
            ),
        }
    });
    let dependency_declarations = input
        .relations
        .iter()
        .map(|requirement| {
            let field = &requirement.field;
            quote!(self.#field.declaration().clone())
        })
        .collect::<Vec<_>>();
    let declaration_requirements = input
        .relations
        .iter()
        .map(|requirement| {
            let requirement = resource_requirement_expr(&sdk, requirement);
            quote!(#requirement.declaration().clone())
        })
        .collect::<Vec<_>>();
    let relation_metadata = input
        .relations
        .iter()
        .map(|relation| {
            let name = relation.field.to_string();
            let target = &relation.target;
            let requirement = resource_requirement_expr(&sdk, relation);
            quote!((
                #sdk::component::ComponentRelationName::new(#name)
                    .expect("resource! generated a non-empty relation role"),
                <#target as #sdk::authoring::RelationTarget>::relation_target_descriptor(),
                #requirement.declaration().clone(),
            ))
        })
        .collect::<Vec<_>>();
    let dependency_bindings = input.relations.iter().map(|requirement| {
        let field = &requirement.field;
        quote! {
            self.#field
                .bind(bindings)
                .map_err(|error| #sdk::core::ModuleError::new(error.to_string()))?;
        }
    });

    let differential_field = effective_contract_tokens.as_ref().map(|tokens| {
        let contract = &tokens.contract_name;
        quote!(realization: #sdk::authoring::ContractDependency<#resource_mod::raw::#contract>,)
    });
    let differential_initializer = effective_contract_tokens.as_ref().map(|_| {
        quote!(realization: #sdk::authoring::ContractDependency::new(
            <#resource_name as #sdk::authoring::AdaptableResourceDefinition>::realization_requirement(),
        ),)
    });
    let differential_declaration = effective_contract_tokens
        .as_ref()
        .map(|_| quote!(declarations.push(self.realization.declaration().clone());));
    let differential_requirement_declaration = effective_contract_tokens.as_ref().map(|_| {
        quote!(required.push(
            <#resource_name as #sdk::authoring::AdaptableResourceDefinition>::realization_requirement()
                .declaration().clone(),
        );)
    });
    let differential_binding = effective_contract_tokens.as_ref().map(|_| quote!(
        self.realization.bind(bindings).map_err(|error| #sdk::core::ModuleError::new(error.to_string()))?;
    ));
    let adaptable_impl = if let Some(tokens) = effective_contract_tokens.as_ref() {
        let effective_contract = &tokens.contract_name;
        let effective_key = quote!(#resource_mod::raw::effective_realization_contract_key());
        quote! {
            impl #sdk::authoring::AdaptableResourceDefinition for #resource_name {
                type RealizationContract = #resource_mod::raw::#effective_contract;
                fn realization_requirement() -> #sdk::core::ContractRequirement<Self::RealizationContract> {
                    match #effective_key.identity() {
                        #sdk::core::ContractIdentity::Provisional => #sdk::core::ContractRequirement::provisional(#effective_key.id().clone()),
                        #sdk::core::ContractIdentity::Versioned(version) => #sdk::core::ContractRequirement::versioned(#effective_key.id().clone(), #sdk::core::ContractVersionRequirement::parse(format!("={version}")).expect("resource! generated an exact effective realization requirement")),
                    }
                }
            }
        }
    } else {
        quote! {
            impl #sdk::authoring::AdaptableResourceDefinition for #resource_name {
                type RealizationContract = #resource_mod::raw::#contract_name;

                fn realization_requirement() -> #sdk::core::ContractRequirement<Self::RealizationContract> {
                    let key = <Self as #sdk::authoring::PrimaryResourceContract>::primary_contract_key();
                    match key.identity() {
                        #sdk::core::ContractIdentity::Provisional => {
                            #sdk::core::ContractRequirement::provisional(key.id().clone())
                        }
                        #sdk::core::ContractIdentity::Versioned(version) => {
                            #sdk::core::ContractRequirement::versioned(
                                key.id().clone(),
                                #sdk::core::ContractVersionRequirement::parse(format!("={version}"))
                                    .expect("resource! generated a static exact API requirement"),
                            )
                        }
                    }
                }

                fn supports_semantic_api_adapter() -> bool {
                    true
                }
            }
        }
    };

    let runtime_methods = input.runtime_methods.as_deref().unwrap_or(&[]);
    let runtime_inherent_methods = runtime_methods.iter().map(runtime_method_tokens);
    let runtime_trait_methods = if let Some(differential) = differential {
        api.methods
            .iter()
            .map(|method| {
                let signature = &method.signature;
                let name = &signature.ident;
                let args = method_call_args(signature);
                if differential
                    .mediated
                    .iter()
                    .any(|mediated| mediated == name)
                {
                    quote!(#signature { Self::#name(self, #(#args),*) })
                } else {
                    quote!(#signature { self.realization.#name(#(#args),*) })
                }
            })
            .collect::<Vec<_>>()
    } else {
        runtime_methods
            .iter()
            .map(|method| {
                let signature = &method.signature;
                let name = &signature.ident;
                let args = method_call_args(signature);
                quote!(#signature { Self::#name(self, #(#args),*) })
            })
            .collect::<Vec<_>>()
    };
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
    let has_self_realization = input.runtime_methods.is_some();

    let materialize_self_runtime = if input.runtime_methods.is_some() {
        quote! {
            ::std::option::Option::Some(::std::boxed::Box::new(
                #resource_mod::raw::Runtime::new(
                    selection.module_id().clone(),
                    selection.config().clone(),
                ),
            ))
        }
    } else {
        quote!(::std::option::Option::None)
    };
    let raw_runtime_reexport = input.runtime_methods.is_some().then(|| quote!(Runtime,));
    let self_runtime_definition = input.runtime_methods.is_some().then(|| {
        quote! {
            #[derive(Clone)]
            pub struct Runtime {
                module_id: #sdk::core::ModuleId,
                config: #config_ty,
                runtime_context: #sdk::authoring::RuntimeContext,
                #runtime_state_field
                #(#dependency_fields)*
                #differential_field
            }

            impl Runtime {
                pub fn new(
                    module_id: #sdk::core::ModuleId,
                    config: #config_ty,
                ) -> Self {
                    #runtime_state_initializer
                    Self {
                        module_id,
                        config,
                        runtime_context: #sdk::authoring::RuntimeContext::default(),
                        #runtime_state_value
                        #(#dependency_initializers)*
                        #differential_initializer
                    }
                }

                pub fn config(&self) -> &#config_ty {
                    &self.config
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
                    #differential_declaration
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
                    #differential_binding
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
    });

    quote! {
        #config_definition

        #visibility struct #resource_name;

        impl #resource_name {
            #select_method

            #[doc(hidden)]
            pub fn __fabric_canonical_adapter_builder() -> #target_builder_type {
                #target_builder_type::new()
            }

            #[doc(hidden)]
            pub fn __fabric_canonical_adapter_contract_key() -> #sdk::core::ContractKey<#target_contract_type> {
                #target_contract_key
            }

            #[doc(hidden)]
            pub fn __fabric_canonical_adapter_bridge_mode() -> #sdk::authoring::AdapterBridgeMode {
                #target_bridge_mode
            }
        }

        impl #sdk::authoring::PrimaryResourceContract for #resource_name {
            type Contract = #resource_mod::raw::#contract_name;

            fn primary_contract_key() -> #sdk::core::ContractKey<Self::Contract> {
                #resource_mod::raw::primary_contract_key()
            }
        }

        impl #sdk::authoring::RelationTarget for #resource_name {
            type Contract = #resource_mod::raw::#contract_name;

            fn relation_target_descriptor() -> #sdk::authoring::RelationTargetDescriptor {
                #sdk::authoring::RelationTargetDescriptor::Resource(
                    <Self as #sdk::authoring::ResourceDefinition>::resource_id(),
                )
            }

            fn relation_requirement() -> #sdk::core::ContractRequirement<Self::Contract> {
                let key = <Self as #sdk::authoring::PrimaryResourceContract>::primary_contract_key();
                match key.identity() {
                    #sdk::core::ContractIdentity::Provisional => #sdk::core::ContractRequirement::provisional(key.id().clone()),
                    #sdk::core::ContractIdentity::Versioned(version) => #sdk::core::ContractRequirement::versioned(
                        key.id().clone(),
                        #sdk::core::ContractVersionRequirement::parse(format!("={version}"))
                            .expect("resource! generated a static exact relation requirement"),
                    ),
                }
            }

            fn relation_requirement_versioned(requirement: #sdk::core::ContractVersionRequirement) -> #sdk::core::ContractRequirement<Self::Contract> {
                #sdk::core::ContractRequirement::versioned(
                    <Self as #sdk::authoring::PrimaryResourceContract>::primary_contract_key().id().clone(),
                    requirement,
                )
            }
        }

        impl #sdk::authoring::ComponentAdapterTarget for #resource_name {
            type ComponentConfig = ();
            type ComponentRelations = ();

            fn component_adapter_context(
                _config: &Self::ComponentConfig,
                _scope: &#sdk::component::ComponentParticipationScope,
            ) -> ::std::result::Result<(Self::ComponentConfig, Self::ComponentRelations), #sdk::component::ComponentError> {
                Ok(((), ()))
            }

            fn is_component_participation_target() -> bool { false }
        }

        impl #sdk::authoring::ComponentRelationTarget for #resource_name {
            fn add_component_relation<C>(
                spec: #sdk::authoring::ComponentSpec<C>,
                name: #sdk::component::ComponentRelationName,
                compatibility: #sdk::authoring::ComponentRelationCompatibility,
            ) -> #sdk::authoring::ComponentSpec<C>
            where C: #sdk::authoring::ComponentDefinition {
                let requirement = match compatibility {
                    #sdk::authoring::ComponentRelationCompatibility::Provisional => #sdk::authoring::Requires::<Self>::provisional(),
                    #sdk::authoring::ComponentRelationCompatibility::Versioned(requirement) => #sdk::authoring::Requires::<Self>::versioned(requirement),
                };
                spec.requires_named_resource(#sdk::authoring::ComponentResourceRequirement::new(
                    name,
                    requirement,
                ))
            }

            fn resolve_component_relation(
                scope: &#sdk::component::ComponentParticipationScope,
                name: &#sdk::component::ComponentRelationName,
                compatibility: &#sdk::authoring::ComponentRelationCompatibility,
            ) -> ::std::result::Result<::std::sync::Arc<Self::Contract>, #sdk::component::ComponentError> {
                let requirement = match compatibility {
                    #sdk::authoring::ComponentRelationCompatibility::Provisional => <Self as #sdk::authoring::RelationTarget>::relation_requirement(),
                    #sdk::authoring::ComponentRelationCompatibility::Versioned(requirement) => <Self as #sdk::authoring::RelationTarget>::relation_requirement_versioned(requirement.clone()),
                };
                scope.named_resource_dependency(name, &requirement)
            }
        }

        impl #sdk::authoring::ResourceDefinition for #resource_name {
            type Config = #config_ty;

            fn resource_id() -> #sdk::resource::ResourceId {
                #resource_mod::raw::resource_id()
            }

            fn schema() -> #sdk::resource::ResourceSchemaDescriptor {
                #schema_expr
            }

            fn declaration(
                selection: &#sdk::authoring::ResourceSelection<Self>,
            ) -> #sdk::core::ModuleDeclaration {
                let mut required = ::std::vec![#(#declaration_requirements),*];
                #differential_requirement_declaration
                #sdk::core::ModuleDeclaration::new(selection.module_id().clone())
                    .with_provided_contracts(::std::vec![#resource_mod::raw::primary_contract_key().declaration()])
                    .with_required_contracts(required)
            }

            fn relation_declarations(
                _consumer_module_id: #sdk::core::ModuleId,
            ) -> ::std::vec::Vec<(
                #sdk::component::ComponentRelationName,
                #sdk::authoring::RelationTargetDescriptor,
                #sdk::core::ContractRequirementDeclaration,
            )> {
                ::std::vec![#(#relation_metadata),*]
            }

            fn materialize(
                selection: &#sdk::authoring::ResourceSelection<Self>,
            ) -> ::std::option::Option<::std::boxed::Box<dyn #sdk::core::ModuleRuntime>> {
                #materialize_self_runtime
            }

            fn has_self_realization() -> bool {
                #has_self_realization
            }
        }

        #adaptable_impl

        #[allow(non_snake_case)]
        mod #raw_impl_mod {
            use super::*;

            pub fn resource_id() -> #sdk::resource::ResourceId {
                #sdk::resource::ResourceId::new(#resource_id)
                    .expect("resource! generated a static resource id")
            }

            pub fn primary_contract_id() -> #sdk::core::ContractId {
                #api_contract_id
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

            #canonical_adapter_bridge_definition

            pub fn effective_realization_contract_id() -> #sdk::core::ContractId {
                #sdk::core::ContractId::new(concat!("fabric.resource.realization.", #resource_id))
                    .expect("resource! generated a static effective realization contract id")
            }

            #differential_effective_key_definition

            #differential_contract_definition
            #differential_bridge_definition

            #self_runtime_definition
        }

        #visibility mod #resource_mod {
            #[doc(hidden)]
            pub mod raw {
                pub use super::super::#raw_impl_mod::{
                    #canonical_adapter_builder_name, #contract_name, #service_name, #raw_runtime_reexport
                    primary_contract_id, primary_contract_key, resource_id, #differential_raw_reexports
                };
            }

        }
    }
}

fn requirement_contract_type(sdk: &TokenStream, requirement: &RelationDefinition) -> TokenStream {
    let target = &requirement.target;
    quote!(<#target as #sdk::authoring::RelationTarget>::Contract)
}

fn resource_requirement_expr(sdk: &TokenStream, requirement: &RelationDefinition) -> TokenStream {
    let target = &requirement.target;
    match &requirement.compatibility {
        None => quote!(<#target as #sdk::authoring::RelationTarget>::relation_requirement()),
        Some(crate::ast::RequirementLiteral::Provisional) => {
            quote!(#sdk::core::ContractRequirement::<<#target as #sdk::authoring::RelationTarget>::Contract>::provisional(<#target as #sdk::authoring::RelationTarget>::relation_requirement().id().clone()))
        }
        Some(crate::ast::RequirementLiteral::Versioned(version)) => quote!(
            <#target as #sdk::authoring::RelationTarget>::relation_requirement_versioned(
                #sdk::core::ContractVersionRequirement::parse(#version)
                    .expect("resource! generated a static relation version requirement"),
            )
        ),
    }
}

fn effective_api(
    api: &ApiDefinition,
    differential: &DifferentialRealizationDefinition,
) -> ApiDefinition {
    let mut methods = api
        .methods
        .iter()
        .filter(|method| !differential.mediated.contains(&method.signature.ident))
        .map(|method| ContractMethod {
            signature: method.signature.clone(),
        })
        .collect::<Vec<_>>();
    methods.extend(differential.operations.iter().map(|method| ContractMethod {
        signature: method.signature.clone(),
    }));
    ApiDefinition {
        name: format_ident!("EffectiveRealization"),
        version: api.version.clone(),
        methods,
    }
}
