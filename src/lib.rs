mod codegen;
mod parser;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemMod};

/// Transforms a module into a SOAP web service with automatic router and WSDL generation.
/// 
/// Takes service configuration attributes and generates Axum handlers, XML parsing,
/// and WSDL endpoints for all async functions in the annotated module.
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

/// Generates the enhanced module with SOAP service functionality.
/// 
/// Creates router functions, SOAP handlers, WSDL endpoints, and operation dispatchers
/// for the given service configuration and extracted operations.
fn generate_enhanced_module(
    mut module: ItemMod,
    config: parser::ServiceConfig,
    operations: Vec<parser::SoapOperation>,
) -> TokenStream2 {
    let bind_path = &config.bind_path;
    let wsdl_path = format!("{}/wsdl", bind_path);
    let namespace = &config.namespace;

    // Collect type information
    let types = match parser::collect_types_from_operations(&operations) {
        Ok(types) => types,
        Err(_) => std::collections::HashMap::new(),
    };

    // Generate WSDL content
    let wsdl_content = codegen::generate_wsdl(&config, &operations, &types);

    // Generate operation dispatcher
    let operation_handlers = generate_operation_handlers(&operations, namespace);

    let router_code = quote! {
        use std::collections::HashMap;

        pub fn router() -> axum::Router {
            axum::Router::new()
                .route(#bind_path, axum::routing::post(soap_handler))
                .route(#wsdl_path, axum::routing::get(wsdl_handler))
        }

        async fn soap_handler(body: String) -> axum::response::Response {
            match handle_soap_request(&body).await {
                Ok(response) => {
                    axum::response::Response::builder()
                        .status(200)
                        .header("Content-Type", "text/xml; charset=utf-8")
                        .header("SOAPAction", "")
                        .body(response.into())
                        .unwrap()
                }
                Err(error) => {
                    let fault = create_soap_fault(&error);
                    axum::response::Response::builder()
                        .status(500)
                        .header("Content-Type", "text/xml; charset=utf-8")
                        .body(fault.into())
                        .unwrap()
                }
            }
        }

        async fn handle_soap_request(xml: &str) -> Result<String, String> {
            // Parse SOAP envelope using proper XML parsing
            let parsed_request = parse_soap_envelope(xml)?;
            let operation = &parsed_request.operation;
            let body_content = &parsed_request.body_xml;

            #operation_handlers

            Err(format!("Unknown operation: {}", operation))
        }

        #[derive(Debug)]
        struct ParsedSoapRequest {
            operation: String,
            body_xml: String,
            namespace: Option<String>,
        }

        fn parse_soap_envelope(xml: &str) -> Result<ParsedSoapRequest, String> {
            // Handle different SOAP Body variations
            let body_start_patterns = ["<soap:Body>", "<SOAP-ENV:Body>", "<Body>"];
            let body_end_patterns = ["</soap:Body>", "</SOAP-ENV:Body>", "</Body>"];

            let mut body_start_pos = None;
            let mut body_end_pos = None;
            let mut body_tag_len = 0;

            // Find body start
            for pattern in &body_start_patterns {
                if let Some(pos) = xml.find(pattern) {
                    body_start_pos = Some(pos);
                    body_tag_len = pattern.len();
                    break;
                }
            }

            // Find body end
            for pattern in &body_end_patterns {
                if let Some(pos) = xml.find(pattern) {
                    body_end_pos = Some(pos);
                    break;
                }
            }

            let body_start = body_start_pos.ok_or("SOAP Body start tag not found")?;
            let body_end = body_end_pos.ok_or("SOAP Body end tag not found")?;

            if body_start + body_tag_len >= body_end {
                return Err("Invalid SOAP Body structure".to_string());
            }

            let body_content = &xml[body_start + body_tag_len..body_end];
            let trimmed_body = body_content.trim();

            // Extract operation name from first element in body
            let operation = extract_first_element_name(trimmed_body)?;

            Ok(ParsedSoapRequest {
                operation,
                body_xml: trimmed_body.to_string(),
                namespace: extract_target_namespace(xml),
            })
        }

        fn extract_first_element_name(xml: &str) -> Result<String, String> {
            let xml = xml.trim();
            if !xml.starts_with('<') {
                return Err("No XML element found".to_string());
            }

            let after_bracket = &xml[1..];
            let tag_end = after_bracket.find('>')
                .ok_or("Invalid XML: no closing bracket found")?;

            let tag_content = &after_bracket[..tag_end];

            // Handle self-closing tags
            let tag_name = if tag_content.ends_with('/') {
                &tag_content[..tag_content.len() - 1]
            } else {
                tag_content
            };

            // Remove namespace prefix and attributes
            let clean_name = tag_name.split_whitespace().next().unwrap_or(tag_name);
            let operation = if clean_name.contains(':') {
                clean_name.split(':').last().unwrap_or(clean_name)
            } else {
                clean_name
            };

            Ok(operation.to_string())
        }

        fn extract_target_namespace(xml: &str) -> Option<String> {
            // Look for targetNamespace or xmlns attributes
            if let Some(start) = xml.find("targetNamespace=\"") {
                let after_start = &xml[start + 17..];
                if let Some(end) = after_start.find('"') {
                    return Some(after_start[..end].to_string());
                }
            }

            // Fallback to default xmlns
            if let Some(start) = xml.find("xmlns=\"") {
                let after_start = &xml[start + 7..];
                if let Some(end) = after_start.find('"') {
                    return Some(after_start[..end].to_string());
                }
            }

            None
        }

        fn create_simple_soap_response(content: &str, operation: &str, namespace: &str) -> String {
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
               xmlns:tns="{}">
    <soap:Body>
        <tns:{}Response>
            {}
        </tns:{}Response>
    </soap:Body>
</soap:Envelope>"#,
                namespace, operation, content, operation
            )
        }

        fn extract_xml_value(xml: &str, tag_name: &str) -> Option<String> {
            // Try multiple patterns to handle namespaces and variations
            let patterns = [
                format!("<{}>", tag_name),
                format!("<{}:", tag_name),  // Handle namespace prefixes
                format!("<tns:{}>", tag_name),
                format!("<ns1:{}>", tag_name),
            ];

            for start_pattern in &patterns {
                if let Some(start_pos) = xml.find(start_pattern) {
                    // Find the actual end of the opening tag
                    let tag_start = start_pos + start_pattern.len();
                    let remaining = &xml[start_pos..];

                    if let Some(close_bracket) = remaining.find('>') {
                        let content_start = start_pos + close_bracket + 1;

                        // Look for the closing tag
                        let end_patterns = [
                            format!("</{}>", tag_name),
                            format!("</{}:", tag_name),
                            format!("</tns:{}>", tag_name),
                            format!("</ns1:{}>", tag_name),
                        ];

                        for end_pattern in &end_patterns {
                            if let Some(end_pos) = xml[content_start..].find(end_pattern) {
                                let actual_end = content_start + end_pos;
                                if content_start <= actual_end {
                                    let content = &xml[content_start..actual_end];
                                    return Some(decode_xml_content(content.trim()));
                                }
                            }
                        }

                        // Handle self-closing tags like <tag/>
                        if remaining[..close_bracket].ends_with('/') {
                            return Some(String::new());
                        }
                    }
                }
            }
            None
        }

        fn decode_xml_content(content: &str) -> String {
            content
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&")
                .replace("&quot;", "\"")
                .replace("&apos;", "'")
        }

        // Generic request parsing using serde_xml_rs directly on operation XML
        fn parse_request_from_xml<T>(xml: &str) -> Result<T, String>
        where
            T: for<'de> ::serde::Deserialize<'de>,
        {
            // The xml parameter is already the operation content (e.g., "<Add><Operand1>123</Operand1><Operand2>456</Operand2></Add>")
            // Use serde_xml_rs to deserialize it directly
            ::serde_xml_rs::from_str(xml)
                .map_err(|e| format!("XML deserialization error: {} for XML: {}", e, xml))
        }


        // Generic response serialization using serde_xml_rs
        fn serialize_response_to_xml<T>(response: &T) -> Result<String, String>
        where
            T: ::serde::Serialize,
        {
            // Use serde_xml_rs for serialization
            ::serde_xml_rs::to_string(response)
                .map_err(|e| format!("XML serialization error: {}", e))
        }


        fn create_soap_fault(error: &str) -> String {
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <soap:Fault>
            <faultcode>Server</faultcode>
            <faultstring>{}</faultstring>
        </soap:Fault>
    </soap:Body>
</soap:Envelope>"#,
                error
            )
        }

        async fn wsdl_handler() -> axum::response::Response {
            let wsdl = #wsdl_content;

            axum::response::Response::builder()
                .status(200)
                .header("Content-Type", "text/xml; charset=utf-8")
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

/// Generates SOAP operation handlers for dispatching requests to service functions.
/// 
/// Creates conditional branches that parse XML requests, call the appropriate async function,
/// and serialize responses back to SOAP XML format.
fn generate_operation_handlers(
    operations: &[parser::SoapOperation],
    namespace: &str,
) -> TokenStream2 {
    let mut handlers = Vec::new();

    for operation in operations {
        let op_name = &operation.name;
        let func_name = &operation.function_name;
        let request_type = &operation.request_type;
        let response_type = &operation.response_type;

        handlers.push(quote! {
            if operation == #op_name {
                // Generic XML parsing using serde
                let request_data: #request_type = match parse_request_from_xml(&body_content) {
                    Ok(data) => data,
                    Err(e) => return Err(format!("Failed to parse request: {}", e)),
                };

                let result: #response_type = #func_name(request_data).await
                    .map_err(|e| format!("Operation failed: {}", e))?;

                // Generic response serialization using serde
                let response_xml = match serialize_response_to_xml(&result) {
                    Ok(xml) => xml,
                    Err(e) => return Err(format!("Failed to serialize response: {}", e)),
                };

                return Ok(create_simple_soap_response(&response_xml, #op_name, #namespace));
            }
        });
    }

    quote! {
        #(#handlers)*
    }
}
