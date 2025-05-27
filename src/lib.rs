mod codegen;
mod parser;
mod schema;
mod soap;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemMod};

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let config = match parser::parse_service_attributes(args.into()) {
        Ok(config) => config,
        Err(e) => return e.to_compile_error().into(),
    };

    let module = parse_macro_input!(input as ItemMod);

    let operations = match parser::extract_soap_operations(&module) {
        Ok(ops) => ops,
        Err(e) => return e.to_compile_error().into(),
    };

    let enhanced_module = generate_enhanced_module(module, config, operations);
    enhanced_module.into()
}

fn generate_enhanced_module(
    mut module: ItemMod,
    config: parser::ServiceConfig,
    operations: Vec<parser::SoapOperation>,
) -> TokenStream2 {
    // For now, generate a simple router function
    // TODO: Implement full SOAP handling logic
    
    let bind_path = &config.bind_path;
    let wsdl_path = format!("{}/wsdl", bind_path);
    
    let router_code = quote! {
        pub fn router() -> axum::Router {
            axum::Router::new()
                .route(#bind_path, axum::routing::post(soap_handler))
                .route(#wsdl_path, axum::routing::get(wsdl_handler))
        }

        async fn soap_handler(body: String) -> axum::response::Response {
            // TODO: Implement SOAP request handling
            axum::response::Response::builder()
                .status(500)
                .body("SOAP handling not yet implemented".into())
                .unwrap()
        }

        async fn wsdl_handler() -> axum::response::Response {
            // TODO: Generate actual WSDL
            let wsdl = r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://schemas.xmlsoap.org/wsdl/"
             targetNamespace="http://example.com/soap">
    <!-- WSDL generation not yet implemented -->
</definitions>"#;
            
            axum::response::Response::builder()
                .status(200)
                .header("Content-Type", "text/xml")
                .body(wsdl.into())
                .unwrap()
        }
    };

    // Add the router code to the module
    if let Some((brace, ref mut items)) = module.content {
        // Parse the router code as items and add them
        let router_items: syn::File = syn::parse2(router_code).unwrap();
        items.extend(router_items.items);
        module.content = Some((brace, items.clone()));
    }

    quote! { #module }
}