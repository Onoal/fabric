use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::ast::{
    ApiDefinition, ContractMethod, DifferentialRealizationDefinition, RelationDefinition,
    SystemInput,
};

use super::common::{
    CanonicalAdapterBridgeTokens, PrimaryContractTokens, SubjectKind, api_contract_id_expr,
    canonical_adapter_bridge_tokens, canonical_adapter_bridge_tokens_named, config_type_tokens,
    fabric_path, has_config, inline_config_definition_tokens, method_call_args,
    primary_contract_tokens, runtime_method_tokens, to_snake_case, version_literal_expr,
};

pub fn expand_system(input: &SystemInput) -> TokenStream {
    let sdk = fabric_path();
    let visibility = &input.visibility;
    let system_name = &input.name;
    let config_name = format_ident!("{}Config", system_name);
    let config_ty = config_type_tokens(&input.config, &config_name);
    let config_definition =
        inline_config_definition_tokens(&input.config, visibility, &config_name);
    let select_method = if has_config(&input.config) {
        quote! {
            pub fn select(
                config: #config_ty,
            ) -> ::std::result::Result<
                #sdk::authoring::SystemSelection<Self>,
                #sdk::system::SystemCompatibilityError,
            > {
                <Self as #sdk::authoring::SystemDefinition>::select(config)
            }
        }
    } else {
        quote! {
            pub fn select() -> ::std::result::Result<
                #sdk::authoring::SystemSelection<Self>,
                #sdk::system::SystemCompatibilityError,
            > {
                <Self as #sdk::authoring::SystemDefinition>::select(())
            }
        }
    };
    let system_mod = format_ident!("{}", to_snake_case(system_name));
    let raw_impl_mod = format_ident!("__fabric_raw_{}", to_snake_case(system_name));
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
        let service = &tokens.service_name;
        let contract = &tokens.contract_name;
        let service_methods = &tokens.service_methods;
        let contract_methods = &tokens.contract_methods;
        quote! {
            pub trait #service: Send + Sync { #(#service_methods)* }
            #[derive(Clone)] pub struct #contract { inner: ::std::sync::Arc<dyn #service> }
            impl #contract { pub fn new(inner: ::std::sync::Arc<dyn #service>) -> Self { Self { inner } } #(#contract_methods)* }
        }
    });
    let differential_effective_key_definition = effective_contract_tokens.as_ref().map(|tokens| {
        let contract = &tokens.contract_name;
        let key = &tokens.contract_key_expr;
        quote!(pub fn effective_realization_contract_key() -> #sdk::core::ContractKey<#contract> { #key })
    });
    let differential_bridge = effective_contract_tokens.as_ref().map(|tokens| {
        let effective_api = effective_api
            .as_ref()
            .expect("effective API accompanies tokens");
        canonical_adapter_bridge_tokens_named(
            effective_api,
            &tokens.service_name,
            &tokens.contract_name,
            "Effective",
        )
    });
    let differential_builder_name = differential_bridge
        .as_ref()
        .map(|bridge| bridge.builder_name.clone());
    let differential_bridge_definition = differential_bridge
        .as_ref()
        .map(|bridge| bridge.definition.clone());
    let target_builder_name = differential_builder_name
        .as_ref()
        .unwrap_or(&canonical_adapter_builder_name);
    let target_builder_type = quote!(#system_mod::raw::#target_builder_name);
    let target_contract_type = if let Some(tokens) = effective_contract_tokens.as_ref() {
        let contract = &tokens.contract_name;
        quote!(#system_mod::raw::#contract)
    } else {
        quote!(#system_mod::raw::#contract_name)
    };
    let target_contract_key = if differential.is_some() {
        quote!(#system_mod::raw::effective_realization_contract_key())
    } else {
        quote!(#system_mod::raw::primary_contract_key())
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
    let system_id = &input.system_id;
    let api_contract_id = api_contract_id_expr(&sdk, system_id, SubjectKind::System);
    let schema_expr = version_literal_expr(&sdk, &system_mod, &input.schema, SubjectKind::System);
    let dependency_fields = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        let contract_ty = dependency_contract_type(&sdk, dependency);
        quote!(#field: #sdk::authoring::ContractDependency<#contract_ty>,)
    });
    let dependency_initializers = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        let requirement_expr = system_requirement_expr(&sdk, dependency);
        quote! {
            #field: #sdk::authoring::ContractDependency::new(
                #requirement_expr
            ),
        }
    });
    let dependency_declarations = input
        .relations
        .iter()
        .map(|dependency| {
            let field = &dependency.field;
            quote!(self.#field.declaration().clone())
        })
        .collect::<Vec<_>>();
    let declaration_requirements = input
        .relations
        .iter()
        .map(|dependency| {
            let requirement = system_requirement_expr(&sdk, dependency);
            quote!(#requirement.declaration().clone())
        })
        .collect::<Vec<_>>();
    let relation_metadata = input
        .relations
        .iter()
        .map(|relation| {
            let name = relation.field.to_string();
            let target = &relation.target;
            let requirement = system_requirement_expr(&sdk, relation);
            quote!((
                #sdk::component::ComponentRelationName::new(#name)
                    .expect("system! generated a non-empty relation role"),
                <#target as #sdk::authoring::RelationTarget>::relation_target_descriptor(),
                #requirement.declaration().clone(),
            ))
        })
        .collect::<Vec<_>>();
    let api_endpoint_names = api
        .methods
        .iter()
        .map(|method| method.signature.ident.to_string())
        .collect::<Vec<_>>();
    let dependency_bindings = input.relations.iter().map(|dependency| {
        let field = &dependency.field;
        quote! {
            self.#field
                .bind(bindings)
                .map_err(|error| #sdk::core::ModuleError::new(error.to_string()))?;
        }
    });

    let differential_field = effective_contract_tokens.as_ref().map(|tokens| {
        let contract = &tokens.contract_name;
        quote!(realization: #sdk::authoring::ContractDependency<#system_mod::raw::#contract>,)
    });
    let differential_initializer = effective_contract_tokens.as_ref().map(|_| quote!(
        realization: #sdk::authoring::ContractDependency::new(
            <#system_name as #sdk::authoring::AdaptableSystemDefinition>::realization_requirement(),
        ),
    ));
    let differential_declaration = effective_contract_tokens
        .as_ref()
        .map(|_| quote!(declarations.push(self.realization.declaration().clone());));
    let differential_requirement_declaration = effective_contract_tokens.as_ref().map(|_| quote!(
        required.push(<#system_name as #sdk::authoring::AdaptableSystemDefinition>::realization_requirement().declaration().clone());
    ));
    let differential_binding = effective_contract_tokens.as_ref().map(|_| quote!(
        self.realization.bind(bindings).map_err(|error| #sdk::core::ModuleError::new(error.to_string()))?;
    ));
    let adaptable_impl = if let Some(tokens) = effective_contract_tokens.as_ref() {
        let contract = &tokens.contract_name;
        quote! {
            impl #sdk::authoring::AdaptableSystemDefinition for #system_name {
                type RealizationContract = #system_mod::raw::#contract;
                fn realization_requirement() -> #sdk::core::ContractRequirement<Self::RealizationContract> {
                    let key = #system_mod::raw::effective_realization_contract_key();
                    match key.identity() {
                        #sdk::core::ContractIdentity::Provisional => #sdk::core::ContractRequirement::provisional(key.id().clone()),
                        #sdk::core::ContractIdentity::Versioned(version) => #sdk::core::ContractRequirement::versioned(key.id().clone(), #sdk::core::ContractVersionRequirement::parse(format!("={version}")).expect("system! generated an exact effective realization requirement")),
                    }
                }
            }
        }
    } else {
        quote! {
            impl #sdk::authoring::AdaptableSystemDefinition for #system_name {
                type RealizationContract = #system_mod::raw::#contract_name;

                fn realization_requirement() -> #sdk::core::ContractRequirement<Self::RealizationContract> {
                    let key = <Self as #sdk::authoring::PrimarySystemContract>::primary_contract_key();
                    match key.identity() {
                        #sdk::core::ContractIdentity::Provisional => {
                            #sdk::core::ContractRequirement::provisional(key.id().clone())
                        }
                        #sdk::core::ContractIdentity::Versioned(version) => {
                            #sdk::core::ContractRequirement::versioned(
                                key.id().clone(),
                                #sdk::core::ContractVersionRequirement::parse(format!("={version}"))
                                    .expect("system! generated a static exact API requirement"),
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
                quote! {
                    #signature {
                        Self::#name(self, #(#args),*)
                    }
                }
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
                #system_mod::raw::Runtime::new(
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

        #visibility struct #system_name;

        impl #system_name {
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

        impl #sdk::authoring::PrimarySystemContract for #system_name {
            type Contract = #system_mod::raw::#contract_name;

            fn primary_contract_key() -> #sdk::core::ContractKey<Self::Contract> {
                #system_mod::raw::primary_contract_key()
            }
        }

        impl #sdk::authoring::RelationTarget for #system_name {
            type Contract = #system_mod::raw::#contract_name;
            fn relation_target_descriptor() -> #sdk::authoring::RelationTargetDescriptor {
                #sdk::authoring::RelationTargetDescriptor::System(
                    <Self as #sdk::authoring::SystemDefinition>::system_id(),
                )
            }
            fn relation_requirement() -> #sdk::core::ContractRequirement<Self::Contract> {
                let key = <Self as #sdk::authoring::PrimarySystemContract>::primary_contract_key();
                match key.identity() {
                    #sdk::core::ContractIdentity::Provisional => #sdk::core::ContractRequirement::provisional(key.id().clone()),
                    #sdk::core::ContractIdentity::Versioned(version) => #sdk::core::ContractRequirement::versioned(key.id().clone(), #sdk::core::ContractVersionRequirement::parse(format!("={version}")).expect("system! generated a static exact relation requirement")),
                }
            }
            fn relation_requirement_versioned(requirement: #sdk::core::ContractVersionRequirement) -> #sdk::core::ContractRequirement<Self::Contract> {
                #sdk::core::ContractRequirement::versioned(<Self as #sdk::authoring::PrimarySystemContract>::primary_contract_key().id().clone(), requirement)
            }
        }

        impl #sdk::authoring::ComponentAdapterTarget for #system_name {
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

        impl #sdk::authoring::ComponentRelationTarget for #system_name {
            fn add_component_relation<C>(
                spec: #sdk::authoring::ComponentSpec<C>,
                name: #sdk::component::ComponentRelationName,
                compatibility: #sdk::authoring::ComponentRelationCompatibility,
            ) -> #sdk::authoring::ComponentSpec<C>
            where C: #sdk::authoring::ComponentDefinition {
                let requirement = match compatibility {
                    #sdk::authoring::ComponentRelationCompatibility::Provisional => #sdk::authoring::SystemRequires::<Self>::provisional(),
                    #sdk::authoring::ComponentRelationCompatibility::Versioned(requirement) => #sdk::authoring::SystemRequires::<Self>::versioned(requirement),
                };
                spec.requires_named_system(#sdk::authoring::ComponentSystemRequirement::new(
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
                scope.named_system_dependency(name, &requirement)
            }
        }

        impl #sdk::authoring::SystemDefinition for #system_name {
            type Config = #config_ty;

            fn system_id() -> #sdk::system::SystemId {
                #system_mod::raw::system_id()
            }

            fn schema() -> #sdk::system::SystemSchemaDescriptor {
                #schema_expr
            }

            fn api_metadata() -> #sdk::SemanticApiMetadata {
                let key = #system_mod::raw::primary_contract_key();
                #sdk::SemanticApiMetadata::new(
                    key.id().clone(),
                    key.identity().clone(),
                    ::std::vec![
                        #(#sdk::SemanticApiEndpoint::new(#api_endpoint_names)),*
                    ],
                )
            }

            fn declaration(
                selection: &#sdk::authoring::SystemSelection<Self>,
            ) -> #sdk::core::ModuleDeclaration {
                let mut required = ::std::vec![#(#declaration_requirements),*];
                #differential_requirement_declaration
                #sdk::core::ModuleDeclaration::new(selection.module_id().clone())
                    .with_provided_contracts(::std::vec![#system_mod::raw::primary_contract_key().declaration()])
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
                selection: &#sdk::authoring::SystemSelection<Self>,
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

            pub fn system_id() -> #sdk::system::SystemId {
                #sdk::system::SystemId::new(#system_id)
                    .expect("system! generated a static system id")
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
                #sdk::core::ContractId::new(concat!("fabric.system.realization.", #system_id))
                    .expect("system! generated a static effective realization contract id")
            }
            #differential_effective_key_definition
            #differential_contract_definition
            #differential_bridge_definition

            #self_runtime_definition
        }

        #visibility mod #system_mod {
            #[doc(hidden)]
            pub mod raw {
                pub use super::super::#raw_impl_mod::{
                    #canonical_adapter_builder_name, #contract_name, #service_name, #raw_runtime_reexport
                    primary_contract_id, primary_contract_key, system_id, #differential_raw_reexports
                };
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
                    .expect("system! generated a static relation version requirement"),
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
