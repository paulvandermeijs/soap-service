//! Analyze request/response struct types

use std::collections::HashMap;
use syn::{
    Attribute, Data, DeriveInput, Error, Fields, GenericArgument, Lit, Meta, PathArguments, Result,
    Type, TypePath,
};

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
    I32,
    I64,
    F32,
    F64,
    Bool,
    Optional(Box<FieldType>),
    Vec(Box<FieldType>),
    Custom(String),
}

pub fn analyze_type(ty: &Type) -> Result<TypeInfo> {
    match ty {
        Type::Path(type_path) => {
            let type_name = extract_type_name(type_path);
            
            // For now, we'll create a placeholder TypeInfo
            // In a real implementation, we'd need access to the actual struct definition
            Ok(TypeInfo {
                name: type_name,
                fields: vec![], // Would be populated from actual struct definition
                namespace: None,
            })
        }
        _ => Err(Error::new_spanned(
            ty,
            "Only named types are supported for SOAP operations",
        )),
    }
}

pub fn analyze_struct_definition(item: &DeriveInput) -> Result<TypeInfo> {
    let struct_name = item.ident.to_string();
    
    let data = match &item.data {
        Data::Struct(data) => data,
        _ => {
            return Err(Error::new_spanned(
                item,
                "Only structs are supported for SOAP types",
            ));
        }
    };
    
    let fields = match &data.fields {
        Fields::Named(fields) => &fields.named,
        _ => {
            return Err(Error::new_spanned(
                item,
                "Only structs with named fields are supported",
            ));
        }
    };
    
    let mut field_infos = Vec::new();
    
    for field in fields {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let xml_name = extract_xml_name_from_attributes(&field.attrs, &field_name);
        let field_type = analyze_field_type(&field.ty)?;
        let optional = is_optional_type(&field.ty);
        
        field_infos.push(FieldInfo {
            name: field_name,
            xml_name,
            field_type,
            optional,
        });
    }
    
    Ok(TypeInfo {
        name: struct_name,
        fields: field_infos,
        namespace: None, // Could be extracted from attributes if needed
    })
}

fn extract_type_name(type_path: &TypePath) -> String {
    type_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn extract_xml_name_from_attributes(attrs: &[Attribute], default_name: &str) -> String {
    for attr in attrs {
        if attr.path().is_ident("serde") {
            if let Ok(meta_list) = attr.meta.require_list() {
                let tokens_str = meta_list.tokens.to_string();
                if tokens_str.contains("rename") {
                    // Simple parsing of rename = "value"
                    if let Some(start) = tokens_str.find('"') {
                        if let Some(end) = tokens_str.rfind('"') {
                            if start < end {
                                return tokens_str[start + 1..end].to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    default_name.to_string()
}

fn analyze_field_type(ty: &Type) -> Result<FieldType> {
    match ty {
        Type::Path(type_path) => {
            let type_name = extract_type_name(type_path);
            
            // Handle Option<T>
            if type_name == "Option" {
                if let Some(inner_type) = extract_generic_argument(type_path) {
                    let inner_field_type = analyze_field_type(inner_type)?;
                    return Ok(FieldType::Optional(Box::new(inner_field_type)));
                }
            }
            
            // Handle Vec<T>
            if type_name == "Vec" {
                if let Some(inner_type) = extract_generic_argument(type_path) {
                    let inner_field_type = analyze_field_type(inner_type)?;
                    return Ok(FieldType::Vec(Box::new(inner_field_type)));
                }
            }
            
            // Handle primitive types
            match type_name.as_str() {
                "String" | "str" => Ok(FieldType::String),
                "i32" => Ok(FieldType::I32),
                "i64" => Ok(FieldType::I64),
                "f32" => Ok(FieldType::F32),
                "f64" => Ok(FieldType::F64),
                "bool" => Ok(FieldType::Bool),
                _ => Ok(FieldType::Custom(type_name)),
            }
        }
        _ => Err(Error::new_spanned(
            ty,
            "Unsupported field type for SOAP operations",
        )),
    }
}

fn extract_generic_argument(type_path: &TypePath) -> Option<&Type> {
    if let Some(segment) = type_path.path.segments.last() {
        if let PathArguments::AngleBracketed(args) = &segment.arguments {
            if let Some(GenericArgument::Type(ty)) = args.args.first() {
                return Some(ty);
            }
        }
    }
    None
}

fn is_optional_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        let type_name = extract_type_name(type_path);
        type_name == "Option"
    } else {
        false
    }
}

pub fn collect_types_from_operations(
    operations: &[crate::parser::SoapOperation],
) -> Result<HashMap<String, TypeInfo>> {
    let mut types = HashMap::new();
    
    for operation in operations {
        // Analyze request type
        let request_type_info = analyze_type(&operation.request_type)?;
        types.insert(request_type_info.name.clone(), request_type_info);
        
        // Analyze response type  
        let response_type_info = analyze_type(&operation.response_type)?;
        types.insert(response_type_info.name.clone(), response_type_info);
        
        // Note: We skip error types for now as they're typically not part of WSDL
    }
    
    Ok(types)
}