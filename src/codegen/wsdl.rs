//! Generate WSDL document and endpoint

use crate::parser::{ServiceConfig, SoapOperation, TypeInfo};

pub fn generate_wsdl(
    _config: &ServiceConfig,
    _operations: &[SoapOperation],
    _types: &[TypeInfo],
) -> String {
    // TODO: Generate complete WSDL document
    // TODO: Include type definitions (XSD schema)
    // TODO: Define messages, port types, bindings, services
    // TODO: Use proper SOAP 1.1/1.2 bindings
    String::from("<!-- TODO: WSDL generation not implemented -->")
}

pub fn rust_type_to_xsd_type(rust_type: &str) -> &str {
    match rust_type {
        "i32" => "xsd:int",
        "i64" => "xsd:long",
        "f32" => "xsd:float",
        "f64" => "xsd:double",
        "String" => "xsd:string",
        "bool" => "xsd:boolean",
        _ => "xsd:string", // fallback
    }
}