//! Analyze request/response struct types

use syn::{Error, Result, Type};

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub xml_name: String,
    pub field_type: FieldType,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Optional(Box<FieldType>),
    Custom(String),
}

pub fn analyze_type(_ty: &Type) -> Result<TypeInfo> {
    // TODO: Parse struct definitions
    // TODO: Extract field information
    // TODO: Handle serde attributes (#[serde(rename = "...")])
    // TODO: Detect optional fields (Option<T>)
    Err(Error::new(
        proc_macro2::Span::call_site(),
        "Not yet implemented",
    ))
}