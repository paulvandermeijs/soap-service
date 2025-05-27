//! SOAP envelope parsing/generation

use quick_xml::{events::Event, Reader, Writer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

#[derive(Debug)]
pub enum SoapError {
    ParseError(String),
    SerializationError(String),
    XmlError(quick_xml::Error),
    SerdeError(String),
}

impl std::fmt::Display for SoapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SoapError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SoapError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            SoapError::XmlError(e) => write!(f, "XML error: {}", e),
            SoapError::SerdeError(msg) => write!(f, "Serde error: {}", msg),
        }
    }
}

impl std::error::Error for SoapError {}

impl From<quick_xml::Error> for SoapError {
    fn from(err: quick_xml::Error) -> Self {
        SoapError::XmlError(err)
    }
}

#[derive(Debug)]
pub struct SoapRequest {
    pub operation: String,
    pub body: String,
    pub headers: HashMap<String, String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SoapEnvelope<T> {
    #[serde(rename = "Body")]
    body: SoapBody<T>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SoapBody<T> {
    #[serde(flatten)]
    content: T,
}

pub fn parse_soap_request(xml: &str) -> Result<SoapRequest, SoapError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut operation = String::new();
    let mut body_content = String::new();
    let mut namespace = None;
    let mut in_body = false;
    let mut body_depth = 0;
    
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .map_err(|_| SoapError::ParseError("Invalid UTF-8 in element name".to_string()))?
                    .to_string();
                
                if name.ends_with(":Body") || name == "Body" {
                    in_body = true;
                    body_depth = 1;
                } else if in_body && body_depth == 1 {
                    // This is the operation element
                    operation = name.to_string();
                    
                    // Extract namespace from attributes
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            if attr.key.as_ref() == b"xmlns" {
                                namespace = Some(
                                    std::str::from_utf8(&attr.value)
                                        .map_err(|_| SoapError::ParseError("Invalid UTF-8 in namespace".to_string()))?
                                        .to_string()
                                );
                            }
                        }
                    }
                    
                    body_depth += 1;
                    
                    // Start capturing the body content
                    let mut body_writer = Writer::new(Cursor::new(Vec::new()));
                    body_writer.write_event(Event::Start(e.clone()))?;
                    
                    // Read until we close this element
                    let mut inner_depth = 1;
                    loop {
                        match reader.read_event_into(&mut buf)? {
                            Event::Start(ref inner_e) => {
                                inner_depth += 1;
                                body_writer.write_event(Event::Start(inner_e.clone()))?;
                            }
                            Event::End(ref inner_e) => {
                                body_writer.write_event(Event::End(inner_e.clone()))?;
                                inner_depth -= 1;
                                if inner_depth == 0 {
                                    break;
                                }
                            }
                            Event::Text(ref text) => {
                                body_writer.write_event(Event::Text(text.clone()))?;
                            }
                            Event::CData(ref cdata) => {
                                body_writer.write_event(Event::CData(cdata.clone()))?;
                            }
                            Event::Empty(ref empty) => {
                                body_writer.write_event(Event::Empty(empty.clone()))?;
                            }
                            Event::Eof => break,
                            _ => {}
                        }
                    }
                    
                    let result = body_writer.into_inner().into_inner();
                    body_content = String::from_utf8(result)
                        .map_err(|_| SoapError::ParseError("Invalid UTF-8 in body content".to_string()))?;
                    break;
                } else if in_body {
                    body_depth += 1;
                }
            }
            Event::End(ref e) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .map_err(|_| SoapError::ParseError("Invalid UTF-8 in element name".to_string()))?
                    .to_string();
                
                if in_body && (name.ends_with(":Body") || name == "Body") {
                    in_body = false;
                    break;
                } else if in_body {
                    body_depth -= 1;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    
    if operation.is_empty() {
        return Err(SoapError::ParseError("No operation found in SOAP body".to_string()));
    }
    
    Ok(SoapRequest {
        operation,
        body: body_content,
        headers: HashMap::new(), // TODO: Parse SOAP headers if needed
        namespace,
    })
}

pub fn create_soap_response<T: serde::Serialize>(
    result: &T,
    operation: &str,
    namespace: &str,
) -> Result<String, SoapError> {
    // Serialize the result to XML first
    let serialized_data = quick_xml::se::to_string(result)
        .map_err(|e| SoapError::SerdeError(e.to_string()))?;
    
    // Create the response operation name (typically operationResponse)
    let response_operation = format!("{}Response", operation);
    
    // Build the complete SOAP envelope
    let soap_response = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
               xmlns:tns="{}">
    <soap:Body>
        <tns:{}>
            {}
        </tns:{}>
    </soap:Body>
</soap:Envelope>"#,
        namespace,
        response_operation,
        extract_inner_xml(&serialized_data)?,
        response_operation
    );
    
    Ok(soap_response)
}

fn extract_inner_xml(xml: &str) -> Result<String, SoapError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut content = String::new();
    let mut inside_root = false;
    
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(_) if !inside_root => {
                inside_root = true;
            }
            Event::Start(e) if inside_root => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap().to_string();
                content.push_str(&format!("<{}>", name));
            }
            Event::End(e) if inside_root => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap().to_string();
                if content.is_empty() && inside_root {
                    // This is the closing of the root element
                    break;
                }
                content.push_str(&format!("</{}>", name));
            }
            Event::Text(e) if inside_root => {
                content.push_str(&e.unescape().unwrap());
            }
            Event::Empty(e) if inside_root => {
                let name = std::str::from_utf8(e.name().as_ref()).unwrap().to_string();
                content.push_str(&format!("<{}/>", name));
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    
    Ok(content)
}