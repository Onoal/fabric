use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{FnArg, Pat, PatIdent, ReturnType};

use crate::ast::{
    ApiDefinition, ApiIdentity, ConfigDefinition, ContractMethod, RequirementLiteral,
    RuntimeMethod, VersionLiteral,
};

pub struct PrimaryContractTokens {
    pub service_name: Ident,
    pub contract_name: Ident,
    pub service_methods: Vec<TokenStream>,
    pub contract_methods: Vec<TokenStream>,
    pub contract_key_expr: TokenStream,
}

/// Target-owned machinery for canonical `adapter! { Name for Target }`
/// lowering.  The adapter macro calls the generated inherent factory on the
/// Rust-resolved target type; it never reconstructs this module from the
/// spelling of that type.
pub struct CanonicalAdapterBridgeTokens {
    pub definition: TokenStream,
    pub builder_name: Ident,
}

pub fn canonical_adapter_bridge_tokens(
    api: &ApiDefinition,
    service_name: &Ident,
    contract_name: &Ident,
) -> CanonicalAdapterBridgeTokens {
    canonical_adapter_bridge_tokens_named(api, service_name, contract_name, "")
}

pub fn canonical_adapter_bridge_tokens_named(
    api: &ApiDefinition,
    service_name: &Ident,
    contract_name: &Ident,
    prefix: &str,
) -> CanonicalAdapterBridgeTokens {
    let builder_name = if prefix.is_empty() {
        format_ident!("CanonicalAdapterBuilder")
    } else {
        format_ident!("{prefix}CanonicalAdapterBuilder")
    };
    let service_name_bridge = if prefix.is_empty() {
        format_ident!("CanonicalAdapterService")
    } else {
        format_ident!("{prefix}CanonicalAdapterService")
    };
    let method_names = api
        .methods
        .iter()
        .map(|method| method.signature.ident.clone())
        .collect::<Vec<_>>();
    let function_names = method_names
        .iter()
        .map(|name| format_ident!("F{}", to_upper_camel_case(name)))
        .collect::<Vec<_>>();
    let missing_names = method_names
        .iter()
        .map(|name| {
            format_ident!(
                "{prefix}CanonicalAdapterMethodMissing{}",
                to_upper_camel_case(name)
            )
        })
        .collect::<Vec<_>>();

    let builder_ty = |types: &[TokenStream]| {
        if types.is_empty() {
            quote!(#builder_name)
        } else {
            quote!(#builder_name<#(#types),*>)
        }
    };
    let service_ty = |types: &[TokenStream]| {
        if types.is_empty() {
            quote!(#service_name_bridge<R>)
        } else {
            quote!(#service_name_bridge<R, #(#types),*>)
        }
    };
    let function_type_tokens = function_names
        .iter()
        .map(|name| quote!(#name))
        .collect::<Vec<_>>();
    let missing_type_tokens = method_names
        .iter()
        .zip(missing_names.iter())
        .map(|(_, name)| quote!(#name))
        .collect::<Vec<_>>();
    let builder_type = builder_ty(&function_type_tokens);
    let missing_builder_type = builder_ty(&missing_type_tokens);
    let service_type = service_ty(&function_type_tokens);

    let builder_generics =
        (!function_names.is_empty()).then(|| quote!(<#(#function_names = #missing_names),*>));
    let builder_impl_generics =
        (!function_names.is_empty()).then(|| quote!(<#(#function_names),*>));
    let service_generics = if function_names.is_empty() {
        quote!(<R>)
    } else {
        quote!(<R, #(#function_names),*>)
    };
    let builder_fields = method_names
        .iter()
        .zip(function_names.iter())
        .map(|(name, function)| quote!(#name: #function,))
        .collect::<Vec<_>>();
    let missing_fields = method_names
        .iter()
        .zip(missing_names.iter())
        .map(|(name, missing)| quote!(#name: #missing,));
    let service_fields = method_names.iter().map(|name| quote!(#name: self.#name,));

    let setters = method_names.iter().enumerate().map(|(index, name)| {
        let replacement = function_names
            .iter()
            .enumerate()
            .map(|(candidate_index, candidate)| {
                if candidate_index == index {
                    quote!(Next)
                } else {
                    quote!(#candidate)
                }
            })
            .collect::<Vec<_>>();
        let return_type = builder_ty(&replacement);
        let fields = method_names.iter().map(|field| {
            if field == name {
                quote!(#field: #field,)
            } else {
                quote!(#field: self.#field,)
            }
        });
        quote! {
            pub fn #name<Next>(self, #name: Next) -> #return_type {
                #builder_name { #(#fields)* }
            }
        }
    });

    let function_bounds = api
        .methods
        .iter()
        .zip(function_names.iter())
        .map(|(method, function)| {
            let inputs = method
                .signature
                .inputs
                .iter()
                .filter_map(|argument| match argument {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(argument) => Some(&argument.ty),
                })
                .collect::<Vec<_>>();
            let output = match &method.signature.output {
                ReturnType::Default => quote!(()),
                ReturnType::Type(_, ty) => quote!(#ty),
            };
            quote!(#function: ::std::ops::Fn(&R, #(#inputs),*) -> #output + Send + Sync + 'static,)
        })
        .collect::<Vec<_>>();
    let service_methods = api.methods.iter().map(|method| {
        let signature = &method.signature;
        let name = &signature.ident;
        let args = method_call_args(signature);
        quote! {
            #signature {
                (self.#name)(&self.runtime, #(#args),*)
            }
        }
    });

    let definition = quote! {
        #(
            #[doc(hidden)]
            pub struct #missing_names;
        )*

        #[doc(hidden)]
        pub struct #builder_name #builder_generics {
            #(#builder_fields)*
        }

        impl #missing_builder_type {
            #[doc(hidden)]
            pub fn new() -> Self {
                Self { #(#missing_fields)* }
            }
        }

        impl #builder_impl_generics #builder_type {
            #(#setters)*
        }

        #[doc(hidden)]
        pub struct #service_name_bridge #service_generics {
            runtime: ::std::sync::Arc<R>,
            #(#builder_fields)*
        }

        impl #builder_impl_generics #builder_type {
            #[doc(hidden)]
            pub fn build<R>(self, runtime: ::std::sync::Arc<R>) -> #contract_name
            where
                R: Send + Sync + 'static,
                #(#function_bounds)*
            {
                #contract_name::new(::std::sync::Arc::new(#service_name_bridge {
                    runtime,
                    #(#service_fields)*
                }))
            }
        }

        impl #service_generics #service_name for #service_type
        where
            R: Send + Sync + 'static,
            #(#function_bounds)*
        {
            #(#service_methods)*
        }
    };

    CanonicalAdapterBridgeTokens {
        definition,
        builder_name,
    }
}

fn to_upper_camel_case(identifier: &Ident) -> String {
    let mut upper_next = true;
    identifier
        .to_string()
        .chars()
        .filter_map(|character| {
            if character == '_' {
                upper_next = true;
                None
            } else if upper_next {
                upper_next = false;
                Some(character.to_ascii_uppercase())
            } else {
                Some(character)
            }
        })
        .collect()
}

pub fn config_type_tokens(config: &ConfigDefinition, generated_name: &Ident) -> TokenStream {
    match config {
        ConfigDefinition::None => quote!(()),
        ConfigDefinition::Inline(_) => quote!(#generated_name),
        ConfigDefinition::Type(ty) => quote!(#ty),
    }
}

pub fn inline_config_definition_tokens(
    config: &ConfigDefinition,
    visibility: &syn::Visibility,
    generated_name: &Ident,
) -> TokenStream {
    let ConfigDefinition::Inline(fields) = config else {
        return TokenStream::new();
    };
    let fields = fields.iter().map(|field| {
        let name = &field.name;
        let ty = &field.ty;
        quote!(pub #name: #ty,)
    });
    quote! {
        #[derive(Clone)]
        #visibility struct #generated_name {
            #(#fields)*
        }
    }
}

pub fn has_config(config: &ConfigDefinition) -> bool {
    !matches!(config, ConfigDefinition::None)
}

pub fn primary_contract_tokens(
    sdk: &TokenStream,
    api: &ApiDefinition,
    contract_id_expr: TokenStream,
) -> PrimaryContractTokens {
    let service_name = format_ident!("{}Service", api.name);
    let contract_name = format_ident!("{}Contract", api.name);
    let service_methods = api.methods.iter().map(service_method_tokens).collect();
    let contract_methods = api
        .methods
        .iter()
        .map(contract_wrapper_method_tokens)
        .collect();
    let contract_key_expr = contract_key_expr(sdk, &api.version, &contract_name, contract_id_expr);

    PrimaryContractTokens {
        service_name,
        contract_name,
        service_methods,
        contract_methods,
        contract_key_expr,
    }
}

pub fn api_contract_id_expr(
    sdk: &TokenStream,
    identity: &ApiIdentity,
    owner_id: &syn::LitStr,
    subject_kind: SubjectKind,
) -> TokenStream {
    match identity {
        ApiIdentity::OwnerDerived => match subject_kind {
            SubjectKind::Resource => quote!(
                #sdk::core::ContractId::new(concat!("fabric.resource.api.", #owner_id))
                    .expect("resource! generated a static owner-derived API contract id")
            ),
            SubjectKind::System => quote!(
                #sdk::core::ContractId::new(concat!("fabric.system.api.", #owner_id))
                    .expect("system! generated a static owner-derived API contract id")
            ),
        },
        ApiIdentity::LegacyExplicit(id) => quote!(
            #sdk::core::ContractId::new(#id)
                .expect("legacy contracts syntax generated a static contract id")
        ),
    }
}

pub fn service_method_tokens(method: &ContractMethod) -> TokenStream {
    let signature = &method.signature;
    quote!(#signature;)
}

pub fn contract_wrapper_method_tokens(method: &ContractMethod) -> TokenStream {
    let signature = &method.signature;
    let name = &signature.ident;
    let args = method_call_args(signature);
    quote! {
        pub #signature {
            self.inner.#name(#(#args),*)
        }
    }
}

pub fn fabric_path() -> TokenStream {
    match crate_name("onoal-fabric") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::fabric),
    }
}

pub fn version_literal_expr(
    sdk: &TokenStream,
    subject_mod: &Ident,
    version: &VersionLiteral,
    subject_kind: SubjectKind,
) -> TokenStream {
    match (subject_kind, version) {
        (SubjectKind::Resource, VersionLiteral::Provisional) => quote!(
            #sdk::resource::ResourceSchemaDescriptor::provisional(#subject_mod::raw::resource_id())
        ),
        (SubjectKind::Resource, VersionLiteral::Versioned(value)) => quote!(
            #sdk::resource::ResourceSchemaDescriptor::versioned(
                #subject_mod::raw::resource_id(),
                #sdk::resource::ResourceSchemaVersion::parse(#value)
                    .expect("resource! generated a static schema version"),
            )
        ),
        (SubjectKind::System, VersionLiteral::Provisional) => quote!(
            #sdk::system::SystemSchemaDescriptor::provisional(#subject_mod::raw::system_id())
        ),
        (SubjectKind::System, VersionLiteral::Versioned(value)) => quote!(
            #sdk::system::SystemSchemaDescriptor::versioned(
                #subject_mod::raw::system_id(),
                #sdk::system::SystemSchemaVersion::parse(#value)
                    .expect("system! generated a static schema version"),
            )
        ),
    }
}

pub fn requirement_literal_expr(
    sdk: &TokenStream,
    requirement: &RequirementLiteral,
    subject_name: &'static str,
) -> TokenStream {
    match requirement {
        RequirementLiteral::Provisional => {
            quote!(#sdk::core::ContractCompatibilityRequirement::provisional())
        }
        RequirementLiteral::Versioned(version) => {
            let expect = format!("{subject_name}! generated a static version requirement");
            quote!(
                #sdk::core::ContractCompatibilityRequirement::versioned(
                    #sdk::core::ContractVersionRequirement::parse(#version)
                        .expect(#expect),
                )
            )
        }
    }
}

pub fn contract_key_expr(
    sdk: &TokenStream,
    version: &VersionLiteral,
    contract_name: &Ident,
    contract_id_expr: TokenStream,
) -> TokenStream {
    match version {
        VersionLiteral::Provisional => quote!(
            #sdk::core::ContractKey::<#contract_name>::provisional(#contract_id_expr)
        ),
        VersionLiteral::Versioned(value) => quote!(
            #sdk::core::ContractKey::<#contract_name>::versioned(
                #contract_id_expr,
                #sdk::core::ContractVersion::parse(#value)
                    .expect("generated a static contract version"),
            )
        ),
    }
}

pub fn runtime_method_tokens(method: &RuntimeMethod) -> TokenStream {
    let signature = &method.signature;
    let body = &method.body;
    quote!(
        #[allow(dead_code)]
        #signature #body
    )
}

pub fn method_call_args(signature: &syn::Signature) -> Vec<TokenStream> {
    signature
        .inputs
        .iter()
        .skip(1)
        .map(|arg| match arg {
            FnArg::Typed(arg) => match arg.pat.as_ref() {
                Pat::Ident(PatIdent {
                    ident,
                    by_ref: None,
                    mutability: None,
                    subpat: None,
                    ..
                }) => quote!(#ident),
                _ => unreachable!("macro validation guarantees simple argument bindings"),
            },
            FnArg::Receiver(_) => {
                unreachable!("macro validation guarantees exactly one leading &self receiver")
            }
        })
        .collect()
}

pub fn to_snake_case(name: &Ident) -> String {
    let input = name.to_string();
    let mut result = String::new();
    for (index, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if index != 0 {
                result.push('_');
            }
            for lower in ch.to_lowercase() {
                result.push(lower);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[derive(Clone, Copy)]
pub enum SubjectKind {
    Resource,
    System,
}
