//! Analyze request/response struct types

use std::collections::HashMap;
use syn::{Error, Result, Type, TypePath};

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub xml_name: String,
    pub field_type: String,
    pub optional: bool,
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
            })
        }
        _ => Err(Error::new_spanned(
            ty,
            "Only named types are supported for SOAP operations",
        )),
    }
}

fn extract_type_name(type_path: &TypePath) -> String {
    type_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .unwrap_or_else(|| "Unknown".to_string())
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