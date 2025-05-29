//! Generate WSDL document and endpoint

use crate::parser::{ServiceConfig, SoapOperation, TypeInfo};
use std::collections::HashMap;

/// Generates a complete WSDL document for the SOAP service.
/// 
/// Creates all WSDL sections including types, messages, port types, bindings,
/// and service definitions based on the service configuration and operations.
pub fn generate_wsdl(
    config: &ServiceConfig,
    operations: &[SoapOperation],
    types: &HashMap<String, TypeInfo>,
) -> String {
    let schema_types = generate_schema_types(types);
    let messages = generate_messages(operations);
    let port_type = generate_port_type(config, operations);
    let binding = generate_binding(config, operations);
    let service = generate_service(config);
    
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://schemas.xmlsoap.org/wsdl/"
             xmlns:soap="http://schemas.xmlsoap.org/wsdl/soap/"
             xmlns:tns="{namespace}"
             xmlns:xsd="http://www.w3.org/2001/XMLSchema"
             targetNamespace="{namespace}"
             elementFormDefault="qualified">

    <types>
        <xsd:schema targetNamespace="{namespace}" elementFormDefault="qualified">
{schema_types}
        </xsd:schema>
    </types>

{messages}

{port_type}

{binding}

{service}

</definitions>"#,
        namespace = config.namespace,
        schema_types = schema_types,
        messages = messages,
        port_type = port_type,
        binding = binding,
        service = service,
    )
}

/// Generates XSD schema type definitions for all request/response types.
fn generate_schema_types(types: &HashMap<String, TypeInfo>) -> String {
    let mut schema = String::new();
    
    for (type_name, type_info) in types {
        schema.push_str(&format!(
            r#"            <xsd:element name="{}" type="tns:{}Type"/>
            <xsd:complexType name="{}Type">
                <xsd:sequence>
"#,
            type_name, type_name, type_name
        ));
        
        for field in &type_info.fields {
            let xsd_type = &field.field_type;
            let min_occurs = if field.optional { " minOccurs=\"0\"" } else { "" };
            
            schema.push_str(&format!(
                r#"                    <xsd:element name="{}" type="{}"{}/>"#,
                field.xml_name, xsd_type, min_occurs
            ));
            schema.push('\n');
        }
        
        schema.push_str(
            r#"                </xsd:sequence>
            </xsd:complexType>
"#,
        );
    }
    
    schema
}

/// Generates WSDL message definitions for all SOAP operations.
/// 
/// Creates request and response message elements for each operation.
fn generate_messages(operations: &[SoapOperation]) -> String {
    let mut messages = String::new();
    
    for operation in operations {
        let request_type = extract_type_name(&operation.request_type);
        let response_type = extract_type_name(&operation.response_type);
        
        messages.push_str(&format!(
            r#"    <message name="{}Request">
        <part name="parameters" element="tns:{}"/>
    </message>
    
    <message name="{}Response">
        <part name="parameters" element="tns:{}"/>
    </message>
    
"#,
            operation.name, request_type, operation.name, response_type
        ));
    }
    
    messages
}

/// Generates the WSDL port type defining the service interface.
/// 
/// Lists all operations with their input and output message types.
fn generate_port_type(config: &ServiceConfig, operations: &[SoapOperation]) -> String {
    let mut port_type = format!(
        r#"    <portType name="{}">
"#,
        config.port_name
    );
    
    for operation in operations {
        port_type.push_str(&format!(
            r#"        <operation name="{}">
            <input message="tns:{}Request"/>
            <output message="tns:{}Response"/>
        </operation>
"#,
            operation.name, operation.name, operation.name
        ));
    }
    
    port_type.push_str("    </portType>\n");
    port_type
}

/// Generates SOAP binding configuration for the service.
/// 
/// Defines the SOAP transport and message format for each operation.
fn generate_binding(config: &ServiceConfig, operations: &[SoapOperation]) -> String {
    let binding_name = format!("{}Binding", config.service_name);
    let mut binding = format!(
        r#"    <binding name="{}" type="tns:{}">
        <soap:binding style="document" transport="http://schemas.xmlsoap.org/soap/http"/>
"#,
        binding_name, config.port_name
    );
    
    for operation in operations {
        let soap_action = format!("{}/{}", config.namespace, operation.name);
        binding.push_str(&format!(
            r#"        <operation name="{}">
            <soap:operation soapAction="{}"/>
            <input>
                <soap:body use="literal"/>
            </input>
            <output>
                <soap:body use="literal"/>
            </output>
        </operation>
"#,
            operation.name, soap_action
        ));
    }
    
    binding.push_str("    </binding>\n");
    binding
}

/// Generates the WSDL service definition with endpoint location.
fn generate_service(config: &ServiceConfig) -> String {
    let binding_name = format!("{}Binding", config.service_name);
    
    format!(
        r#"    <service name="{}">
        <port name="{}" binding="tns:{}">
            <soap:address location="http://localhost:8080{}"/>
        </port>
    </service>"#,
        config.service_name, config.port_name, binding_name, config.bind_path
    )
}

/// Extracts the type name from a syn::Type for WSDL generation.
fn extract_type_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
        _ => "Unknown".to_string(),
    }
}


