//! Generate Axum router code

use crate::parser::{ServiceConfig, SoapOperation};
use proc_macro2::TokenStream;

pub fn generate_router_function(
    _config: &ServiceConfig,
    _operations: &[SoapOperation],
) -> TokenStream {
    // TODO: Generate router with SOAP and WSDL endpoints
    quote::quote! {
        // TODO: Implementation
    }
}

pub fn generate_dispatcher(_operations: &[SoapOperation]) -> TokenStream {
    // TODO: Generate match statement routing operations to functions
    // TODO: Handle deserialization and serialization
    // TODO: Include error handling with SOAP faults
    quote::quote! {
        // TODO: Implementation
    }
}