use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{FnArg, Pat, PatIdent};

use crate::ast::{
    ConfigDefinition, ContractDefinition, ContractMethod, RequirementLiteral, RuntimeMethod,
    VersionLiteral,
};

pub struct PrimaryContractTokens {
    pub service_name: Ident,
    pub contract_name: Ident,
    pub contract_id: syn::LitStr,
    pub service_methods: Vec<TokenStream>,
    pub contract_methods: Vec<TokenStream>,
    pub contract_key_expr: TokenStream,
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
    contract: &ContractDefinition,
    contract_id_expr: TokenStream,
) -> PrimaryContractTokens {
    let service_name = format_ident!("{}Service", contract.name);
    let contract_name = format_ident!("{}Contract", contract.name);
    let contract_id = contract.contract_id.clone();
    let service_methods = contract.methods.iter().map(service_method_tokens).collect();
    let contract_methods = contract
        .methods
        .iter()
        .map(contract_wrapper_method_tokens)
        .collect();
    let contract_key_expr =
        contract_key_expr(sdk, &contract.version, &contract_name, contract_id_expr);

    PrimaryContractTokens {
        service_name,
        contract_name,
        contract_id,
        service_methods,
        contract_methods,
        contract_key_expr,
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
