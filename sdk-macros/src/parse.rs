use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{
    Error, Expr, Ident, LitStr, Path, Result, Token, Type, Visibility, braced, parenthesized,
};

use crate::ast::{
    AdapterInput, ApiDefinition, ComponentInput, ComponentParticipationRealization,
    ConfigDefinition, ConfigField, ContractMethod, DifferentialRealizationDefinition,
    RelationDefinition, RequirementLiteral, ResourceInput, RuntimeLifecycleDefinition,
    RuntimeMethod, RuntimeStateDefinition, SystemInput, VersionLiteral,
};

mod kw {
    // Retained only to issue precise 0.5 migration diagnostics for removed
    // high-level grammar; they are not part of the canonical AST.
    syn::custom_keyword!(adapter);
    syn::custom_keyword!(api);
    syn::custom_keyword!(component);
    syn::custom_keyword!(compatibility);
    syn::custom_keyword!(config);
    syn::custom_keyword!(contracts);
    syn::custom_keyword!(host);
    syn::custom_keyword!(id);
    syn::custom_keyword!(implements);
    syn::custom_keyword!(initialize);
    syn::custom_keyword!(lifecycle);
    syn::custom_keyword!(mediate);
    // Recognized only to report the Component frontend removal; it is never
    // lowered into the canonical AST.
    syn::custom_keyword!(operations);
    syn::custom_keyword!(primary);
    syn::custom_keyword!(prepare);
    syn::custom_keyword!(provisional);
    syn::custom_keyword!(realization);
    syn::custom_keyword!(relations);
    syn::custom_keyword!(requires);
    syn::custom_keyword!(runtime);
    syn::custom_keyword!(state);
    syn::custom_keyword!(start);
    syn::custom_keyword!(stop);
    syn::custom_keyword!(supports);
    syn::custom_keyword!(health);
    syn::custom_keyword!(schema);
    syn::custom_keyword!(system);
    syn::custom_keyword!(teardown);
    syn::custom_keyword!(version);
    syn::custom_keyword!(resource);
}

struct PendingApiDefinition {
    methods: Vec<ContractMethod>,
}

impl Parse for ResourceInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let visibility = input.parse::<Visibility>()?;
        let name = input.parse::<Ident>()?;
        let content;
        braced!(content in input);

        let mut resource_id = None;
        let mut schema = None;
        let mut config = None;
        let mut relations = None;
        let mut api = None;
        let mut differential_realization = None;
        let mut runtime_methods = None;
        let mut runtime_state = None;
        let mut lifecycle = None;

        while !content.is_empty() {
            if content.peek(kw::id) {
                content.parse::<kw::id>()?;
                content.parse::<Token![:]>()?;
                if resource_id.is_some() {
                    return Err(content.error("resource! supports only one `id: ...;` declaration"));
                }
                resource_id = Some(content.parse::<LitStr>()?);
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::schema) {
                content.parse::<kw::schema>()?;
                return Err(content.error(
                    "`schema: ...;` was removed from resource! in Fabric 0.5; omit it for provisional definitions or use `version: \"...\";`",
                ));
            } else if content.peek(kw::version) {
                content.parse::<kw::version>()?;
                content.parse::<Token![:]>()?;
                if schema.is_some() {
                    return Err(
                        content.error("resource! supports only one `version: ...;` declaration")
                    );
                }
                schema = Some(parse_version_literal(&content)?);
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::config) {
                content.parse::<kw::config>()?;
                if config.is_some() {
                    return Err(
                        content.error("resource! supports only one `config { ... }` section")
                    );
                }
                config = Some(parse_config_definition(&content)?);
            } else if content.peek(kw::relations) {
                content.parse::<kw::relations>()?;
                if relations.is_some() {
                    return Err(
                        content.error("resource! supports only one `relations { ... }` section")
                    );
                }
                relations = Some(parse_relations(&content)?);
            } else if content.peek(kw::requires) {
                content.parse::<kw::requires>()?;
                return Err(content.error(
                    "`requires { ... }` was removed from resource! in Fabric 0.5; use `relations { requires { dependency: Target; } }`",
                ));
            } else if content.peek(kw::api) {
                content.parse::<kw::api>()?;
                if api.is_some() {
                    return Err(content.error("resource! supports only one `api { ... }` section"));
                }
                api = Some(parse_api(&content)?);
            } else if content.peek(kw::contracts) {
                content.parse::<kw::contracts>()?;
                return Err(content.error(
                    "`contracts { primary ... }` was removed from resource! in Fabric 0.5; use `api { ... }`",
                ));
            } else if content.peek(kw::adapter) {
                content.parse::<kw::adapter>()?;
                return Err(content.error(
                    "the legacy `adapter Name { ... }` resource realization was removed in Fabric 0.5; use a separate `adapter! { Name for Resource { ... } }` declaration, or `realization { ... }` for a differential boundary",
                ));
            } else if content.peek(kw::realization) {
                content.parse::<kw::realization>()?;
                if differential_realization.is_some() {
                    return Err(
                        content.error("resource! supports only one `realization { ... }` section")
                    );
                }
                differential_realization = Some(parse_differential_realization(&content)?);
            } else if content.peek(kw::runtime) {
                content.parse::<kw::runtime>()?;
                if runtime_methods.is_some() {
                    return Err(
                        content.error("resource! supports only one `runtime { ... }` section")
                    );
                }
                runtime_methods = Some(parse_runtime_methods(&content)?);
            } else if content.peek(kw::state) {
                content.parse::<kw::state>()?;
                if runtime_state.is_some() {
                    return Err(
                        content.error("resource! supports only one `state { ... }` section")
                    );
                }
                runtime_state = Some(parse_runtime_state(&content)?);
            } else if content.peek(kw::lifecycle) {
                content.parse::<kw::lifecycle>()?;
                if lifecycle.is_some() {
                    return Err(
                        content.error("resource! supports only one `lifecycle { ... }` section")
                    );
                }
                lifecycle = Some(parse_runtime_lifecycle(&content)?);
            } else {
                return Err(content.error("unsupported resource! section"));
            }
        }

        let name_for_errors = name.clone();
        let schema = schema.unwrap_or(VersionLiteral::Provisional);
        let api = resolve_api(api, &schema, &name_for_errors, "resource")?;
        validate_differential_realization(
            &api,
            differential_realization.as_ref(),
            runtime_methods.as_deref(),
            "resource",
        )?;

        Ok(Self {
            visibility,
            name,
            resource_id: resource_id.ok_or_else(|| {
                Error::new(
                    name_for_errors.span(),
                    "resource! requires an `id: ...;` declaration",
                )
            })?,
            schema,
            config: config.unwrap_or(ConfigDefinition::None),
            relations: relations.unwrap_or_default(),
            api,
            differential_realization,
            runtime_methods,
            runtime_state,
            lifecycle: lifecycle.unwrap_or_default(),
        })
    }
}

impl Parse for SystemInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let visibility = input.parse::<Visibility>()?;
        let name = input.parse::<Ident>()?;
        let content;
        braced!(content in input);

        let mut system_id = None;
        let mut schema = None;
        let mut config = None;
        let mut relations = None;
        let mut api = None;
        let mut differential_realization = None;
        let mut runtime_methods = None;
        let mut runtime_state = None;
        let mut lifecycle = None;

        while !content.is_empty() {
            if content.peek(kw::id) {
                content.parse::<kw::id>()?;
                content.parse::<Token![:]>()?;
                if system_id.is_some() {
                    return Err(content.error("system! supports only one `id: ...;` declaration"));
                }
                system_id = Some(content.parse::<LitStr>()?);
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::schema) {
                content.parse::<kw::schema>()?;
                return Err(content.error(
                    "`schema: ...;` was removed from system! in Fabric 0.5; omit it for provisional definitions or use `version: \"...\";`",
                ));
            } else if content.peek(kw::version) {
                content.parse::<kw::version>()?;
                content.parse::<Token![:]>()?;
                if schema.is_some() {
                    return Err(
                        content.error("system! supports only one `version: ...;` declaration")
                    );
                }
                schema = Some(parse_version_literal(&content)?);
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::config) {
                content.parse::<kw::config>()?;
                if config.is_some() {
                    return Err(content.error("system! supports only one `config { ... }` section"));
                }
                config = Some(parse_config_definition(&content)?);
            } else if content.peek(kw::relations) {
                content.parse::<kw::relations>()?;
                if relations.is_some() {
                    return Err(
                        content.error("system! supports only one `relations { ... }` section")
                    );
                }
                relations = Some(parse_relations(&content)?);
            } else if content.peek(kw::system) {
                content.parse::<kw::system>()?;
                return Err(content.error(
                    "`system { ... }` dependencies were removed from system! in Fabric 0.5; use `relations { requires { dependency: Target; } }`",
                ));
            } else if content.peek(kw::api) {
                content.parse::<kw::api>()?;
                if api.is_some() {
                    return Err(content.error("system! supports only one `api { ... }` section"));
                }
                api = Some(parse_api(&content)?);
            } else if content.peek(kw::contracts) {
                content.parse::<kw::contracts>()?;
                return Err(content.error(
                    "`contracts { primary ... }` was removed from system! in Fabric 0.5; use `api { ... }`",
                ));
            } else if content.peek(kw::adapter) {
                content.parse::<kw::adapter>()?;
                return Err(content.error(
                    "the legacy `adapter Name { ... }` system realization was removed in Fabric 0.5; use a separate `adapter! { Name for System { ... } }` declaration, or `realization { ... }` for a differential boundary",
                ));
            } else if content.peek(kw::realization) {
                content.parse::<kw::realization>()?;
                if differential_realization.is_some() {
                    return Err(
                        content.error("system! supports only one `realization { ... }` section")
                    );
                }
                differential_realization = Some(parse_differential_realization(&content)?);
            } else if content.peek(kw::runtime) {
                content.parse::<kw::runtime>()?;
                if runtime_methods.is_some() {
                    return Err(
                        content.error("system! supports only one `runtime { ... }` section")
                    );
                }
                runtime_methods = Some(parse_runtime_methods(&content)?);
            } else if content.peek(kw::state) {
                content.parse::<kw::state>()?;
                if runtime_state.is_some() {
                    return Err(content.error("system! supports only one `state { ... }` section"));
                }
                runtime_state = Some(parse_runtime_state(&content)?);
            } else if content.peek(kw::lifecycle) {
                content.parse::<kw::lifecycle>()?;
                if lifecycle.is_some() {
                    return Err(
                        content.error("system! supports only one `lifecycle { ... }` section")
                    );
                }
                lifecycle = Some(parse_runtime_lifecycle(&content)?);
            } else {
                return Err(content.error("unsupported system! section"));
            }
        }

        let name_for_errors = name.clone();
        let schema = schema.unwrap_or(VersionLiteral::Provisional);
        let api = resolve_api(api, &schema, &name_for_errors, "system")?;
        validate_differential_realization(
            &api,
            differential_realization.as_ref(),
            runtime_methods.as_deref(),
            "system",
        )?;

        Ok(Self {
            visibility,
            name,
            system_id: system_id.ok_or_else(|| {
                Error::new(
                    name_for_errors.span(),
                    "system! requires an `id: ...;` declaration",
                )
            })?,
            schema,
            config: config.unwrap_or(ConfigDefinition::None),
            relations: relations.unwrap_or_default(),
            api,
            differential_realization,
            runtime_methods,
            runtime_state,
            lifecycle: lifecycle.unwrap_or_default(),
        })
    }
}

impl Parse for AdapterInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let visibility = input.parse::<Visibility>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![for]>()?;
        if input.peek(kw::resource) {
            input.parse::<kw::resource>()?;
            return Err(input.error(
                "`adapter! { Name for resource Target implements Interface { ... } }` was removed in Fabric 0.5; write `adapter! { Name for Target { ... } }`",
            ));
        } else if input.peek(kw::system) {
            input.parse::<kw::system>()?;
            return Err(input.error(
                "`adapter! { Name for system Target implements Interface { ... } }` was removed in Fabric 0.5; write `adapter! { Name for Target { ... } }`",
            ));
        }
        let target = input.parse::<Path>()?;
        if input.peek(kw::implements) {
            input.parse::<kw::implements>()?;
            return Err(input.error(
                "`implements ...` was removed from canonical adapter! authoring in Fabric 0.5; the target API is the realization contract, so write `adapter! { Name for Target { ... } }`",
            ));
        }
        let content;
        braced!(content in input);

        let mut schema = None;
        let mut config = None;
        let mut relations = None;
        let mut host_requirement = None;
        let mut runtime = None;
        let mut runtime_state = None;
        let mut lifecycle = None;

        while !content.is_empty() {
            if content.peek(kw::schema) {
                content.parse::<kw::schema>()?;
                return Err(content.error(
                    "`schema: ...;` was removed from adapter! in Fabric 0.5; use `supports: ...;` for an explicit compatibility override",
                ));
            } else if content.peek(kw::supports) {
                content.parse::<kw::supports>()?;
                content.parse::<Token![:]>()?;
                if schema.is_some() {
                    return Err(
                        content.error("adapter! supports only one `supports: ...;` declaration")
                    );
                }
                schema = Some(parse_requirement_literal(&content)?);
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::realization) {
                content.parse::<kw::realization>()?;
                return Err(content.error(
                    "`realization: ...;` was removed from adapter! in Fabric 0.5; target compatibility is inferred, or use `supports: ...;` for an explicit compatibility override",
                ));
            } else if content.peek(kw::version) {
                content.parse::<kw::version>()?;
                return Err(content.error(
                    "`version: ...;` is not an Adapter compatibility declaration in Fabric 0.5; compatibility is inferred from the target or declared with `supports: ...;`",
                ));
            } else if content.peek(kw::config) {
                content.parse::<kw::config>()?;
                if config.is_some() {
                    return Err(
                        content.error("adapter! supports only one `config { ... }` section")
                    );
                }
                config = Some(parse_config_definition(&content)?);
            } else if content.peek(kw::relations) {
                content.parse::<kw::relations>()?;
                if relations.is_some() {
                    return Err(
                        content.error("adapter! supports only one `relations { ... }` section")
                    );
                }
                relations = Some(parse_relations(&content)?);
            } else if content.peek(kw::system) {
                content.parse::<kw::system>()?;
                return Err(content.error(
                    "`system { ... }` dependencies were removed from adapter! in Fabric 0.5; use `relations { requires { dependency: Target; } }`",
                ));
            } else if content.peek(kw::host) {
                content.parse::<kw::host>()?;
                content.parse::<Token![:]>()?;
                if host_requirement.is_some() {
                    return Err(content.error("adapter! supports only one `host: ...;` section"));
                }
                host_requirement = Some(content.parse::<Expr>()?);
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::runtime) {
                content.parse::<kw::runtime>()?;
                if runtime.is_some() {
                    return Err(
                        content.error("adapter! supports only one `runtime { ... }` section")
                    );
                }
                runtime = Some(parse_component_runtime(&content)?);
            } else if content.peek(kw::state) {
                content.parse::<kw::state>()?;
                if runtime_state.is_some() {
                    return Err(content.error("adapter! supports only one `state { ... }` section"));
                }
                runtime_state = Some(parse_runtime_state(&content)?);
            } else if content.peek(kw::lifecycle) {
                content.parse::<kw::lifecycle>()?;
                if lifecycle.is_some() {
                    return Err(
                        content.error("adapter! supports only one `lifecycle { ... }` section")
                    );
                }
                lifecycle = Some(parse_runtime_lifecycle(&content)?);
            } else {
                return Err(content.error("unsupported adapter! section"));
            }
        }

        let name_for_errors = name.clone();

        let runtime = runtime.ok_or_else(|| {
            Error::new(
                name_for_errors.span(),
                "adapter! requires a `runtime { ... }` section",
            )
        })?;
        if runtime_state.is_some() && runtime.state.is_some() {
            return Err(Error::new(
                name_for_errors.span(),
                "adapter! cannot combine top-level `state { ... }` with `runtime { state { ... } }`",
            ));
        }

        Ok(Self {
            visibility,
            name,
            target,
            schema,
            config: config.unwrap_or(ConfigDefinition::None),
            relations: relations.unwrap_or_default(),
            host_requirement,
            runtime_methods: runtime.methods,
            runtime_state: runtime.state.or(runtime_state),
            component_prepare: runtime.prepare,
            component_teardown: runtime.teardown,
            lifecycle: lifecycle.unwrap_or_default(),
        })
    }
}

impl Parse for ComponentInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let visibility = input.parse::<Visibility>()?;
        let name = input.parse::<Ident>()?;
        let content;
        braced!(content in input);

        let mut component_id = None;
        let mut config = None;
        let mut relations = None;
        let mut api = None;
        let mut runtime = None;

        while !content.is_empty() {
            if content.peek(kw::id) {
                content.parse::<kw::id>()?;
                content.parse::<Token![:]>()?;
                if component_id.is_some() {
                    return Err(
                        content.error("component! supports only one `id: ...;` declaration")
                    );
                }
                component_id = Some(content.parse::<LitStr>()?);
                content.parse::<Token![;]>()?;
            } else if content.peek(kw::config) {
                content.parse::<kw::config>()?;
                if config.is_some() {
                    return Err(
                        content.error("component! supports only one `config { ... }` section")
                    );
                }
                config = Some(parse_config_definition(&content)?);
            } else if content.peek(kw::relations) {
                content.parse::<kw::relations>()?;
                if relations.is_some() {
                    return Err(
                        content.error("component! supports only one `relations { ... }` section")
                    );
                }
                relations = Some(parse_relations(&content)?);
            } else if content.peek(kw::api) {
                content.parse::<kw::api>()?;
                if api.is_some() {
                    return Err(content.error("component! supports only one `api { ... }` section"));
                }
                api = Some(parse_api(&content)?);
            } else if content.peek(kw::runtime) {
                content.parse::<kw::runtime>()?;
                if runtime.is_some() {
                    return Err(
                        content.error("component! supports only one `runtime { ... }` section")
                    );
                }
                runtime = Some(parse_component_runtime(&content)?);
            } else if content.peek(kw::requires) {
                return Err(content.error(
                    "component! `requires { ... }` was removed; use `relations { requires { role: Target; } }`",
                ));
            } else if content.peek(kw::system) {
                return Err(content.error(
                    "component! `system { ... }` was removed; use `relations { requires { role: Target; } }`",
                ));
            } else if content.peek(kw::operations) {
                return Err(content.error(
                    "component! `operations { ... }` was removed; declare callable behavior with `api { ... }` and implement it in `runtime { ... }`",
                ));
            } else if content.peek(kw::teardown) {
                return Err(content.error(
                    "component! top-level `teardown { ... }` was removed; place it inside `runtime { teardown { ... } }`",
                ));
            } else {
                return Err(content.error("unsupported component! section"));
            }
        }

        let name_for_errors = name.clone();

        Ok(Self {
            visibility,
            name,
            component_id: component_id.ok_or_else(|| {
                Error::new(
                    name_for_errors.span(),
                    "component! requires an `id: ...;` declaration",
                )
            })?,
            config: config.unwrap_or(ConfigDefinition::None),
            relations: relations.unwrap_or_default(),
            api: api.map(|api| ApiDefinition {
                name: Ident::new("Api", name_for_errors.span()),
                version: VersionLiteral::Provisional,
                methods: api.methods,
            }),
            runtime,
        })
    }
}

fn parse_component_runtime(input: ParseStream<'_>) -> Result<ComponentParticipationRealization> {
    let content;
    braced!(content in input);
    let mut methods = Vec::new();
    let mut state = None;
    let mut prepare = None;
    let mut teardown = None;
    while !content.is_empty() {
        if content.peek(kw::state) {
            content.parse::<kw::state>()?;
            if state.is_some() {
                return Err(
                    content.error("component runtime supports only one `state { ... }` section")
                );
            }
            state = Some(parse_runtime_state(&content)?);
        } else if content.peek(kw::prepare) {
            content.parse::<kw::prepare>()?;
            if prepare.is_some() {
                return Err(
                    content.error("component runtime supports only one `prepare { ... }` hook")
                );
            }
            prepare = Some(content.parse::<syn::Block>()?);
        } else if content.peek(kw::teardown) {
            content.parse::<kw::teardown>()?;
            if teardown.is_some() {
                return Err(
                    content.error("component runtime supports only one `teardown { ... }` hook")
                );
            }
            teardown = Some(content.parse::<syn::Block>()?);
        } else {
            let method = content.parse::<syn::ImplItemFn>()?;
            methods.push(RuntimeMethod {
                signature: method.sig,
                body: method.block,
            });
        }
    }
    Ok(ComponentParticipationRealization {
        methods,
        state,
        prepare,
        teardown,
    })
}

fn parse_version_literal(input: ParseStream<'_>) -> Result<VersionLiteral> {
    if input.peek(kw::provisional) {
        input.parse::<kw::provisional>()?;
        Ok(VersionLiteral::Provisional)
    } else if input.peek(LitStr) {
        Ok(VersionLiteral::Versioned(input.parse::<LitStr>()?))
    } else {
        Err(input.error("expected `provisional` or a version string literal"))
    }
}

fn parse_config_fields(input: ParseStream<'_>) -> Result<Vec<ConfigField>> {
    let content;
    braced!(content in input);
    let mut fields = Vec::new();
    while !content.is_empty() {
        let name = content.parse::<Ident>()?;
        content.parse::<Token![:]>()?;
        let ty = content.parse::<Type>()?;
        content.parse::<Token![;]>()?;
        fields.push(ConfigField { name, ty });
    }
    Ok(fields)
}

fn parse_config_definition(input: ParseStream<'_>) -> Result<ConfigDefinition> {
    if input.peek(Token![:]) {
        input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        input.parse::<Token![;]>()?;
        Ok(ConfigDefinition::Type(ty))
    } else if input.peek(syn::token::Brace) {
        parse_config_fields(input).map(ConfigDefinition::Inline)
    } else {
        Err(input.error("expected `config { ... }` or `config: RustConfigType;`"))
    }
}

fn parse_api(input: ParseStream<'_>) -> Result<PendingApiDefinition> {
    let content;
    braced!(content in input);
    let mut methods = Vec::new();
    while !content.is_empty() {
        let method = content.parse::<syn::TraitItemFn>()?;
        if method.default.is_some() {
            return Err(Error::new(
                method.sig.ident.span(),
                "api methods are signatures and must not include bodies",
            ));
        }
        methods.push(ContractMethod {
            signature: method.sig,
        });
    }
    Ok(PendingApiDefinition { methods })
}

fn resolve_api(
    api: Option<PendingApiDefinition>,
    default_version: &VersionLiteral,
    name_for_errors: &Ident,
    subject: &'static str,
) -> Result<ApiDefinition> {
    let api = api.ok_or_else(|| {
        Error::new(
            name_for_errors.span(),
            format!("{subject}! requires an `api {{ ... }}` section"),
        )
    })?;
    Ok(ApiDefinition {
        name: Ident::new("Api", name_for_errors.span()),
        version: default_version.clone(),
        methods: api.methods,
    })
}

fn parse_relations(input: ParseStream<'_>) -> Result<Vec<RelationDefinition>> {
    let content;
    braced!(content in input);
    if !content.peek(kw::requires) {
        return Err(content.error("relations! currently supports only `requires { ... }`"));
    }
    content.parse::<kw::requires>()?;
    let required;
    braced!(required in content);
    if !content.is_empty() {
        return Err(content.error("unexpected tokens after `requires { ... }`"));
    }
    let mut relations = Vec::new();
    while !required.is_empty() {
        let field = required.parse::<Ident>()?;
        required.parse::<Token![:]>()?;
        let target = required.parse::<Path>()?;
        let compatibility = if required.peek(syn::token::Paren) {
            Some(parse_requirement_invocation(&required)?)
        } else {
            None
        };
        required.parse::<Token![;]>()?;
        relations.push(RelationDefinition {
            field,
            target,
            compatibility,
        });
    }
    Ok(relations)
}

fn parse_requirement_invocation(input: ParseStream<'_>) -> Result<RequirementLiteral> {
    let content;
    parenthesized!(content in input);
    if content.peek(kw::provisional) {
        content.parse::<kw::provisional>()?;
        if !content.is_empty() {
            return Err(content.error("unexpected tokens after `provisional`"));
        }
        return Ok(RequirementLiteral::Provisional);
    }

    if content.peek(kw::version) {
        content.parse::<kw::version>()?;
        content.parse::<Token![=]>()?;
        let value = content.parse::<LitStr>()?;
        if !content.is_empty() {
            return Err(content.error("unexpected tokens after version requirement"));
        }
        return Ok(RequirementLiteral::Versioned(value));
    }

    Err(content.error(
        "resource dependency requires explicit `provisional` or `version = \"...\"` compatibility",
    ))
}

fn parse_differential_realization(
    input: ParseStream<'_>,
) -> Result<DifferentialRealizationDefinition> {
    let content;
    braced!(content in input);
    let mut mediated = Vec::new();
    let mut operations = Vec::new();

    while !content.is_empty() {
        if content.peek(kw::mediate) {
            content.parse::<kw::mediate>()?;
            let method = content.parse::<Ident>()?;
            content.parse::<Token![;]>()?;
            if mediated.iter().any(|existing| existing == &method) {
                return Err(Error::new(
                    method.span(),
                    "realization! declares this mediated API method more than once",
                ));
            }
            mediated.push(method);
        } else {
            let method = content.parse::<syn::TraitItemFn>()?;
            if operations
                .iter()
                .any(|existing: &ContractMethod| existing.signature.ident == method.sig.ident)
            {
                return Err(Error::new(
                    method.sig.ident.span(),
                    "realization! declares this realization-only operation more than once",
                ));
            }
            operations.push(ContractMethod {
                signature: method.sig,
            });
        }
    }

    Ok(DifferentialRealizationDefinition {
        mediated,
        operations,
    })
}

fn validate_differential_realization(
    api: &ApiDefinition,
    differential: Option<&DifferentialRealizationDefinition>,
    runtime_methods: Option<&[RuntimeMethod]>,
    subject: &'static str,
) -> Result<()> {
    let Some(differential) = differential else {
        return Ok(());
    };
    let api_method = |name: &Ident| {
        api.methods
            .iter()
            .find(|method| method.signature.ident == *name)
    };
    for mediated in &differential.mediated {
        if api_method(mediated).is_none() {
            return Err(Error::new(
                mediated.span(),
                format!(
                    "{subject}! realization mediates `{mediated}`, but that method is not in the semantic API"
                ),
            ));
        }
        let Some(runtime) = runtime_methods.and_then(|methods| {
            methods
                .iter()
                .find(|method| method.signature.ident == *mediated)
        }) else {
            return Err(Error::new(
                mediated.span(),
                format!(
                    "{subject}! semantic method `{mediated}` is mediated and requires a matching `runtime` implementation"
                ),
            ));
        };
        let expected = api_method(mediated).expect("checked API method");
        if runtime.signature.to_token_stream().to_string()
            != expected.signature.to_token_stream().to_string()
        {
            return Err(Error::new(
                runtime.signature.span(),
                format!(
                    "{subject}! mediation implementation `{mediated}` must match its semantic API signature"
                ),
            ));
        }
    }
    for operation in &differential.operations {
        if api_method(&operation.signature.ident).is_some() {
            return Err(Error::new(
                operation.signature.ident.span(),
                format!(
                    "{subject}! realization-only operation `{}` conflicts with a semantic API method",
                    operation.signature.ident
                ),
            ));
        }
    }
    if let Some(methods) = runtime_methods {
        for method in methods {
            if !differential
                .mediated
                .iter()
                .any(|name| name == &method.signature.ident)
            {
                return Err(Error::new(
                    method.signature.ident.span(),
                    format!(
                        "{subject}! runtime method `{}` is not declared as mediated",
                        method.signature.ident
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn parse_requirement_literal(input: ParseStream<'_>) -> Result<RequirementLiteral> {
    if input.peek(kw::provisional) {
        input.parse::<kw::provisional>()?;
        Ok(RequirementLiteral::Provisional)
    } else if input.peek(LitStr) {
        Ok(RequirementLiteral::Versioned(input.parse::<LitStr>()?))
    } else {
        Err(input.error("expected `provisional` or a version requirement string literal"))
    }
}

fn parse_runtime_methods(input: ParseStream<'_>) -> Result<Vec<RuntimeMethod>> {
    let content;
    braced!(content in input);
    let mut methods = Vec::new();
    while !content.is_empty() {
        let method = content.parse::<syn::ImplItemFn>()?;
        methods.push(RuntimeMethod {
            signature: method.sig,
            body: method.block,
        });
    }
    Ok(methods)
}

fn parse_runtime_state(input: ParseStream<'_>) -> Result<RuntimeStateDefinition> {
    let content;
    braced!(content in input);
    let ty = content.parse::<Type>()?;
    content.parse::<Token![=]>()?;
    let initializer = content.parse::<Expr>()?;
    content.parse::<Token![;]>()?;
    if !content.is_empty() {
        return Err(content.error("state { ... } accepts exactly one `Type = expression;` entry"));
    }
    Ok(RuntimeStateDefinition { ty, initializer })
}

fn parse_runtime_lifecycle(input: ParseStream<'_>) -> Result<RuntimeLifecycleDefinition> {
    let content;
    braced!(content in input);
    let mut lifecycle = RuntimeLifecycleDefinition::default();
    while !content.is_empty() {
        if content.peek(kw::initialize) {
            content.parse::<kw::initialize>()?;
            if lifecycle.initialize.is_some() {
                return Err(content.error("lifecycle supports only one `initialize { ... }` hook"));
            }
            lifecycle.initialize = Some(content.parse::<syn::Block>()?);
        } else if content.peek(kw::start) {
            content.parse::<kw::start>()?;
            if lifecycle.start.is_some() {
                return Err(content.error("lifecycle supports only one `start { ... }` hook"));
            }
            lifecycle.start = Some(content.parse::<syn::Block>()?);
        } else if content.peek(kw::stop) {
            content.parse::<kw::stop>()?;
            if lifecycle.stop.is_some() {
                return Err(content.error("lifecycle supports only one `stop { ... }` hook"));
            }
            lifecycle.stop = Some(content.parse::<syn::Block>()?);
        } else if content.peek(kw::health) {
            content.parse::<kw::health>()?;
            if lifecycle.health.is_some() {
                return Err(content.error("lifecycle supports only one `health: ...;` hook"));
            }
            content.parse::<Token![:]>()?;
            lifecycle.health = Some(content.parse::<Expr>()?);
            content.parse::<Token![;]>()?;
        } else {
            return Err(content.error("expected initialize, start, stop, or health in lifecycle"));
        }
    }
    Ok(lifecycle)
}

syn::custom_keyword!(contracts);
syn::custom_keyword!(implements);
syn::custom_keyword!(primary);
