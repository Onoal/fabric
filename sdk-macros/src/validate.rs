use std::collections::BTreeMap;

use quote::ToTokens;
use semver::{Version, VersionReq};
use syn::spanned::Spanned;
use syn::{Error, Expr, ExprClosure, FnArg, Pat, PatIdent, Result, Signature};

use crate::ast::{
    AdapterInput, ApiDefinition, ComponentInput, ComponentOperationContext, RealizationDefinition,
    RequirementLiteral, ResourceInput, SystemInput, VersionLiteral,
};

struct ApiValidation<'a> {
    async_message: &'a str,
    method_kind: &'a str,
    generic_message: &'a str,
}

pub fn validate_resource(input: &ResourceInput) -> Result<()> {
    let mut errors = None;

    validate_version_literal(
        &input.schema,
        "invalid resource schema version",
        &mut errors,
    );
    validate_resource_dependency_fields(input, &mut errors);

    validate_api(
        &input.api,
        &ApiValidation {
            async_message: "async resource API methods are not supported",
            method_kind: "resource API methods",
            generic_message: "generic resource API methods are not supported",
        },
        &mut errors,
    );

    for requirement in &input.relations {
        if let Some(compatibility) = &requirement.compatibility {
            validate_requirement_literal(
                compatibility,
                "invalid resource dependency version requirement",
                &mut errors,
            );
        }
    }

    if let Some(realization) = &input.realization {
        validate_realization(
            realization,
            &mut errors,
            "resource",
            "invalid resource realization version requirement",
        );
    }

    for method in input.runtime_methods.iter().flatten() {
        if method.signature.asyncness.is_some() {
            push_error(
                &mut errors,
                Error::new(
                    method.signature.ident.span(),
                    "async resource API methods are not supported",
                ),
            );
        }
        validate_supported_signature(
            &method.signature,
            "resource runtime methods",
            "generic resource runtime methods are not supported",
            &mut errors,
        );
    }

    if let Some(runtime_methods) = &input.runtime_methods {
        let required_methods = input.differential_realization.as_ref().map(|differential| {
            input
                .api
                .methods
                .iter()
                .filter(|method| {
                    differential
                        .mediated
                        .iter()
                        .any(|name| name == &method.signature.ident)
                })
                .cloned()
                .collect::<Vec<_>>()
        });
        validate_runtime_method_integrity(
            required_methods.as_ref().or(Some(&input.api.methods)),
            runtime_methods,
            &mut errors,
        );
    }
    validate_self_realization_sections(
        input.runtime_methods.is_some(),
        input.runtime_state.is_some(),
        !input.lifecycle.is_empty(),
        "resource",
        &mut errors,
    );

    finish(errors)
}

pub fn validate_system(input: &SystemInput) -> Result<()> {
    let mut errors = None;

    validate_version_literal(&input.schema, "invalid system schema version", &mut errors);
    validate_system_dependency_fields(input, &mut errors);

    validate_api(
        &input.api,
        &ApiValidation {
            async_message: "async system API methods are not supported",
            method_kind: "system API methods",
            generic_message: "generic system API methods are not supported",
        },
        &mut errors,
    );

    for dependency in &input.relations {
        if let Some(compatibility) = &dependency.compatibility {
            validate_requirement_literal(
                compatibility,
                "invalid system dependency version requirement",
                &mut errors,
            );
        }
    }

    if let Some(realization) = &input.realization {
        validate_realization(
            realization,
            &mut errors,
            "system",
            "invalid system realization version requirement",
        );
    }

    for method in input.runtime_methods.iter().flatten() {
        if method.signature.asyncness.is_some() {
            push_error(
                &mut errors,
                Error::new(
                    method.signature.ident.span(),
                    "async system API methods are not supported",
                ),
            );
        }
        validate_supported_signature(
            &method.signature,
            "system runtime methods",
            "generic system runtime methods are not supported",
            &mut errors,
        );
    }

    if let Some(runtime_methods) = &input.runtime_methods {
        let required_methods = input.differential_realization.as_ref().map(|differential| {
            input
                .api
                .methods
                .iter()
                .filter(|method| {
                    differential
                        .mediated
                        .iter()
                        .any(|name| name == &method.signature.ident)
                })
                .cloned()
                .collect::<Vec<_>>()
        });
        validate_runtime_method_integrity(
            required_methods.as_ref().or(Some(&input.api.methods)),
            runtime_methods,
            &mut errors,
        );
    }
    validate_self_realization_sections(
        input.runtime_methods.is_some(),
        input.runtime_state.is_some(),
        !input.lifecycle.is_empty(),
        "system",
        &mut errors,
    );

    finish(errors)
}

pub fn validate_adapter(input: &AdapterInput) -> Result<()> {
    let mut errors = None;

    if let Some(schema) = &input.schema {
        validate_requirement_literal(
            schema,
            "invalid adapter support version requirement",
            &mut errors,
        );
    }
    validate_version_literal(&input.realization, "invalid adapter version", &mut errors);
    validate_relation_names(&input.relations, &mut errors, "adapter");

    for method in &input.runtime_methods {
        if method.signature.asyncness.is_some() {
            push_error(
                &mut errors,
                Error::new(
                    method.signature.ident.span(),
                    "async adapter runtime methods are not supported",
                ),
            );
        }
        validate_supported_signature(
            &method.signature,
            "adapter runtime methods",
            "generic adapter runtime methods are not supported",
            &mut errors,
        );
    }

    validate_duplicate_methods(
        input
            .runtime_methods
            .iter()
            .map(|method| &method.signature)
            .collect::<Vec<_>>(),
        "runtime",
        &mut errors,
    );

    finish(errors)
}

pub fn validate_component(input: &ComponentInput) -> Result<()> {
    let mut errors = None;
    let mut names = BTreeMap::new();

    validate_component_dependency_fields(input, &mut errors);
    let dependencies_required = !input.requires.is_empty() || !input.systems.is_empty();

    for operation in &input.operations {
        if let Some(first_span) = names.insert(operation.name.to_string(), operation.name.span()) {
            let mut error = Error::new(
                operation.name.span(),
                format!("duplicate component operation `{}`", operation.name),
            );
            error.combine(Error::new(first_span, "first operation declared here"));
            push_error(&mut errors, error);
        }

        validate_component_handler(
            &operation.handler,
            dependencies_required,
            operation.context == ComponentOperationContext::Invocation,
            &mut errors,
        );
    }

    finish(errors)
}

fn finish(errors: Option<Error>) -> Result<()> {
    if let Some(error) = errors {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_self_realization_sections(
    owns_runtime: bool,
    has_state: bool,
    has_lifecycle: bool,
    subject: &str,
    errors: &mut Option<Error>,
) {
    if !owns_runtime && has_state {
        push_error(
            errors,
            Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "{subject}! `state {{ ... }}` requires `runtime {{ ... }}` because state belongs to a self-realizing live layer"
                ),
            ),
        );
    }
    if !owns_runtime && has_lifecycle {
        push_error(
            errors,
            Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "{subject}! `lifecycle {{ ... }}` requires `runtime {{ ... }}` because lifecycle hooks belong to a self-realizing live layer"
                ),
            ),
        );
    }
}

fn validate_api(api: &ApiDefinition, rules: &ApiValidation<'_>, errors: &mut Option<Error>) {
    validate_version_literal(&api.version, "invalid API version", errors);
    for method in &api.methods {
        if method.signature.asyncness.is_some() {
            push_error(
                errors,
                Error::new(method.signature.ident.span(), rules.async_message),
            );
        }
        validate_supported_signature(
            &method.signature,
            rules.method_kind,
            rules.generic_message,
            errors,
        );
    }
}

fn validate_version_literal(
    literal: &VersionLiteral,
    message: &'static str,
    errors: &mut Option<Error>,
) {
    if let VersionLiteral::Versioned(version) = literal
        && let Err(source) = Version::parse(&version.value())
    {
        push_error(
            errors,
            Error::new(version.span(), format!("{message}: {source}")),
        );
    }
}

fn validate_requirement_literal(
    literal: &RequirementLiteral,
    message: &'static str,
    errors: &mut Option<Error>,
) {
    if let RequirementLiteral::Versioned(version) = literal
        && let Err(source) = VersionReq::parse(&version.value())
    {
        push_error(
            errors,
            Error::new(version.span(), format!("{message}: {source}")),
        );
    }
}

fn validate_component_handler(
    handler: &Expr,
    dependencies_required: bool,
    context_required: bool,
    errors: &mut Option<Error>,
) {
    let closure = match handler {
        Expr::Closure(closure) => closure,
        _ => {
            push_error(
                errors,
                Error::new(
                    handler.span(),
                    "component operation handlers must use a closure expression like `|input: Type| async move { ... }`",
                ),
            );
            return;
        }
    };

    validate_component_handler_closure(closure, dependencies_required, context_required, errors);
}

fn validate_component_handler_closure(
    closure: &ExprClosure,
    dependencies_required: bool,
    context_required: bool,
    errors: &mut Option<Error>,
) {
    let expected_inputs = 1 + usize::from(dependencies_required) + usize::from(context_required);
    if closure.inputs.len() != expected_inputs {
        push_error(
            errors,
            Error::new(
                closure.inputs.span(),
                match (context_required, dependencies_required) {
                    (false, false) => {
                        "component operation handlers must declare exactly one typed input parameter"
                    }
                    (false, true) => {
                        "component operation handlers with dependencies must declare `|dependencies, input: Type|`"
                    }
                    (true, false) => {
                        "context-aware component operation handlers must declare `|context, input: Type|`"
                    }
                    (true, true) => {
                        "context-aware component operation handlers with dependencies must declare `|context, dependencies, input: Type|`"
                    }
                },
            ),
        );
        return;
    }

    let mut input_index = 0;
    if context_required {
        let context_input = &closure.inputs[0];
        if !matches!(
            context_input,
            Pat::Ident(PatIdent {
                by_ref: None,
                mutability: None,
                subpat: None,
                ..
            })
        ) {
            push_error(
                errors,
                Error::new(
                    context_input.span(),
                    "context-aware component handlers must use a simple `context` binding as their first parameter",
                ),
            );
        }
        input_index += 1;
    }
    if dependencies_required {
        let dependency_input = &closure.inputs[input_index];
        if !matches!(
            dependency_input,
            Pat::Ident(PatIdent {
                by_ref: None,
                mutability: None,
                subpat: None,
                ..
            })
        ) {
            push_error(
                errors,
                Error::new(
                    dependency_input.span(),
                    "component dependency handlers must use a simple `dependencies` binding as their first parameter",
                ),
            );
        }
        input_index += 1;
    }

    let Some(first_input) = closure.inputs.iter().nth(input_index) else {
        return;
    };
    let Pat::Type(pat_type) = first_input else {
        push_error(
            errors,
            Error::new(
                first_input.span(),
                "component operation handlers must declare their input as `name: Type`",
            ),
        );
        return;
    };
    let Pat::Ident(PatIdent {
        by_ref: None,
        mutability: None,
        subpat: None,
        ..
    }) = pat_type.pat.as_ref()
    else {
        push_error(
            errors,
            Error::new(
                pat_type.pat.span(),
                "component operation handlers must use a simple `name: Type` input binding",
            ),
        );
        return;
    };
}

fn validate_component_dependency_fields(input: &ComponentInput, errors: &mut Option<Error>) {
    let mut fields = BTreeMap::new();
    let mut systems = BTreeMap::new();

    for requirement in &input.requires {
        let name = requirement.field.to_string();
        if let Some(first_span) = fields.insert(name.clone(), requirement.field.span()) {
            let mut error = Error::new(
                requirement.field.span(),
                format!("duplicate component dependency field `{name}`"),
            );
            error.combine(Error::new(first_span, "first dependency declared here"));
            push_error(errors, error);
        }
        validate_requirement_literal(
            &requirement.compatibility,
            "invalid component Resource dependency version requirement",
            errors,
        );
    }

    for dependency in &input.systems {
        let name = dependency.field.to_string();
        if let Some(first_span) = fields.insert(name.clone(), dependency.field.span()) {
            let mut error = Error::new(
                dependency.field.span(),
                format!("duplicate component dependency field `{name}`"),
            );
            error.combine(Error::new(first_span, "first dependency declared here"));
            push_error(errors, error);
        }
        let target = dependency.system.to_token_stream().to_string();
        if let Some(first_span) = systems.insert(target.clone(), dependency.system.span()) {
            let mut error = Error::new(
                dependency.system.span(),
                format!(
                    "component! does not support duplicate System requirement target `{target}`; ComponentSpec has no requirement occurrence identity"
                ),
            );
            error.combine(Error::new(
                first_span,
                "first System requirement declared here",
            ));
            push_error(errors, error);
        }
        validate_requirement_literal(
            &dependency.compatibility,
            "invalid component System dependency version requirement",
            errors,
        );
    }
}

fn validate_resource_dependency_fields(input: &ResourceInput, errors: &mut Option<Error>) {
    let mut seen = BTreeMap::new();
    let reserved = ["config", "module_id"];
    let realization_field = input
        .realization
        .as_ref()
        .map(|realization| to_snake_case(&realization.name));

    for requirement in &input.relations {
        let field = requirement.field.to_string();
        if let Some(first_span) = seen.insert(field.clone(), requirement.field.span()) {
            let mut error = Error::new(
                requirement.field.span(),
                format!("duplicate resource dependency field `{field}` is not supported"),
            );
            error.combine(Error::new(
                first_span,
                format!("first dependency field `{field}` declared here"),
            ));
            push_error(errors, error);
        }

        if reserved.contains(&field.as_str()) {
            push_error(
                errors,
                Error::new(
                    requirement.field.span(),
                    format!(
                        "resource dependency field `{field}` is reserved by generated runtime state"
                    ),
                ),
            );
        }

        if realization_field.as_deref() == Some(field.as_str()) {
            push_error(
                errors,
                Error::new(
                    requirement.field.span(),
                    format!(
                        "resource dependency field `{field}` collides with the generated realization field"
                    ),
                ),
            );
        }
    }
}

fn validate_system_dependency_fields(input: &SystemInput, errors: &mut Option<Error>) {
    validate_relation_names(&input.relations, errors, "system");
}

fn validate_relation_names(
    dependencies: &[crate::ast::RelationDefinition],
    errors: &mut Option<Error>,
    subject: &str,
) {
    let mut seen = BTreeMap::new();
    let reserved = ["config", "module_id", "adapter"];

    for dependency in dependencies {
        let field = dependency.field.to_string();
        if let Some(first_span) = seen.insert(field.clone(), dependency.field.span()) {
            let mut error = Error::new(
                dependency.field.span(),
                format!("duplicate system dependency field `{field}` is not supported"),
            );
            error.combine(Error::new(
                first_span,
                format!("first dependency field `{field}` declared here"),
            ));
            push_error(errors, error);
        }

        if reserved.contains(&field.as_str()) {
            push_error(
                errors,
                Error::new(
                    dependency.field.span(),
                    format!(
                        "{subject} system dependency field `{field}` is reserved by generated runtime state"
                    ),
                ),
            );
        }
    }
}

fn validate_realization(
    realization: &RealizationDefinition,
    errors: &mut Option<Error>,
    subject: &'static str,
    invalid_requirement_message: &'static str,
) {
    validate_requirement_literal(
        &realization.compatibility,
        invalid_requirement_message,
        errors,
    );

    for method in &realization.methods {
        if method.signature.asyncness.is_some() {
            push_error(
                errors,
                Error::new(
                    method.signature.ident.span(),
                    format!("async {subject} realization methods are not supported"),
                ),
            );
        }
        validate_supported_signature(
            &method.signature,
            &format!("{subject} realization methods"),
            &format!("generic {subject} realization methods are not supported"),
            errors,
        );
    }

    validate_duplicate_methods(
        realization
            .methods
            .iter()
            .map(|method| &method.signature)
            .collect::<Vec<_>>(),
        "realization",
        errors,
    );
}

fn validate_runtime_method_integrity(
    primary_methods: Option<&Vec<crate::ast::ContractMethod>>,
    runtime_methods: &[crate::ast::RuntimeMethod],
    errors: &mut Option<Error>,
) {
    let Some(primary_methods) = primary_methods else {
        return;
    };

    validate_duplicate_methods(
        primary_methods
            .iter()
            .map(|method| &method.signature)
            .collect::<Vec<_>>(),
        "contract",
        errors,
    );
    validate_duplicate_methods(
        runtime_methods
            .iter()
            .map(|method| &method.signature)
            .collect::<Vec<_>>(),
        "runtime",
        errors,
    );

    let declared = primary_methods
        .iter()
        .map(|method| {
            (
                method.signature.ident.to_string(),
                signature_key(&method.signature),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let implemented = runtime_methods
        .iter()
        .map(|method| {
            (
                method.signature.ident.to_string(),
                signature_key(&method.signature),
            )
        })
        .collect::<BTreeMap<_, _>>();

    for method in primary_methods {
        let name = method.signature.ident.to_string();
        match implemented.get(&name) {
            None => push_error(
                errors,
                Error::new(
                    method.signature.ident.span(),
                    format!("missing runtime implementation for API method `{name}`"),
                ),
            ),
            Some(runtime_signature) if runtime_signature != &signature_key(&method.signature) => {
                push_error(
                    errors,
                    Error::new(
                        method.signature.ident.span(),
                        format!(
                            "runtime method `{name}` does not match the declared contract signature"
                        ),
                    ),
                )
            }
            Some(_) => {}
        }
    }

    for method in runtime_methods {
        let name = method.signature.ident.to_string();
        if !declared.contains_key(&name) {
            push_error(
                errors,
                Error::new(
                    method.signature.ident.span(),
                    format!("runtime method `{name}` has no matching contract declaration"),
                ),
            );
        }
    }
}

fn signature_key(signature: &syn::Signature) -> String {
    signature.to_token_stream().to_string()
}

fn validate_supported_signature(
    signature: &Signature,
    kind: &str,
    generic_message: &str,
    errors: &mut Option<Error>,
) {
    if signature.constness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
    {
        push_error(
            errors,
            Error::new(
                signature.ident.span(),
                format!("{kind} must use plain `fn` signatures"),
            ),
        );
    }

    if !signature.generics.params.is_empty() || signature.generics.where_clause.is_some() {
        push_error(errors, Error::new(signature.ident.span(), generic_message));
    }

    validate_receiver(signature, kind, errors);
    validate_argument_patterns(signature, kind, errors);
}

fn validate_receiver(signature: &Signature, kind: &str, errors: &mut Option<Error>) {
    let Some(first) = signature.inputs.first() else {
        push_error(
            errors,
            Error::new(
                signature.ident.span(),
                format!("{kind} must declare an `&self` receiver"),
            ),
        );
        return;
    };

    match first {
        FnArg::Receiver(receiver)
            if receiver.reference.is_some()
                && receiver.mutability.is_none()
                && receiver.colon_token.is_none() => {}
        FnArg::Receiver(_) => push_error(
            errors,
            Error::new(
                signature.ident.span(),
                format!("{kind} must use an `&self` receiver"),
            ),
        ),
        FnArg::Typed(_) => push_error(
            errors,
            Error::new(
                signature.ident.span(),
                format!("{kind} must start with an `&self` receiver"),
            ),
        ),
    }
}

fn validate_argument_patterns(signature: &Signature, kind: &str, errors: &mut Option<Error>) {
    for arg in signature.inputs.iter().skip(1) {
        let FnArg::Typed(arg) = arg else {
            push_error(
                errors,
                Error::new(
                    signature.ident.span(),
                    format!("{kind} may contain only one receiver argument"),
                ),
            );
            continue;
        };

        match arg.pat.as_ref() {
            Pat::Ident(PatIdent {
                by_ref: None,
                mutability: None,
                subpat: None,
                ..
            }) => {}
            _ => push_error(
                errors,
                Error::new(
                    arg.pat.span(),
                    format!("{kind} arguments must use simple `name: Type` bindings"),
                ),
            ),
        }
    }
}

fn validate_duplicate_methods(
    signatures: Vec<&Signature>,
    kind: &'static str,
    errors: &mut Option<Error>,
) {
    let mut seen = BTreeMap::new();

    for signature in signatures {
        let name = signature.ident.to_string();
        if let Some(first_span) = seen.insert(name.clone(), signature.ident.span()) {
            let mut error = Error::new(
                signature.ident.span(),
                format!("duplicate {kind} method `{name}` is not supported"),
            );
            error.combine(Error::new(
                first_span,
                format!("first `{name}` {kind} method declared here"),
            ));
            push_error(errors, error);
        }
    }
}

fn to_snake_case(name: &syn::Ident) -> String {
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

fn push_error(target: &mut Option<Error>, error: Error) {
    if let Some(existing) = target {
        existing.combine(error);
    } else {
        *target = Some(error);
    }
}
