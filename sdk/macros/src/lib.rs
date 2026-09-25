//! Fabric's procedural-macro authoring frontend.
//!
//! This crate implements `resource!`, `system!`, `adapter!`, and `component!`.
//! Normal users should depend on the `fabric` umbrella crate, which re-exports
//! them. Generated raw modules and helper types are implementation machinery,
//! not normal authoring vocabulary.

#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod ast;
mod codegen;
mod parse;
mod validate;

#[cfg(test)]
mod tests;

#[proc_macro]
pub fn resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ast::ResourceInput);
    match validate::validate_resource(&input) {
        Ok(()) => codegen::expand_resource(&input).into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn system(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ast::SystemInput);
    match validate::validate_system(&input) {
        Ok(()) => codegen::expand_system(&input).into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn adapter(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ast::AdapterInput);
    match validate::validate_adapter(&input) {
        Ok(()) => codegen::expand_adapter(&input).into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ast::ComponentInput);
    match validate::validate_component(&input) {
        Ok(()) => codegen::expand_component(&input).into(),
        Err(error) => error.into_compile_error().into(),
    }
}
