//! Parse and validate async functions

use syn::{
    Error, FnArg, GenericArgument, Ident, Item, ItemFn, ItemMod, PathArguments, Result, ReturnType,
    Type, TypePath, Visibility,
};

#[derive(Debug, Clone)]
pub struct SoapOperation {
    pub name: String,
    pub function_name: Ident,
    pub request_type: Type,
    pub response_type: Type,
}

pub fn extract_soap_operations(module: &ItemMod) -> Result<Vec<SoapOperation>> {
    let mut operations = Vec::new();
    
    if let Some((_, items)) = &module.content {
        for item in items {
            if let Item::Fn(func) = item {
                if is_valid_soap_function(func)? {
                    let operation = parse_soap_function(func)?;
                    operations.push(operation);
                }
            }
        }
    }
    
    Ok(operations)
}

fn is_valid_soap_function(func: &ItemFn) -> Result<bool> {
    // Check if function is public
    if !matches!(func.vis, Visibility::Public(_)) {
        return Ok(false);
    }
    
    // Check if function is async
    if func.sig.asyncness.is_none() {
        return Ok(false);
    }
    
    Ok(true)
}

fn parse_soap_function(func: &ItemFn) -> Result<SoapOperation> {
    let function_name = func.sig.ident.clone();
    let name = generate_operation_name(&function_name);
    
    // Validate function signature
    let request_type = extract_request_type(func)?;
    let (response_type, _error_type) = extract_return_types(func)?;
    
    Ok(SoapOperation {
        name,
        function_name,
        request_type,
        response_type,
    })
}

fn generate_operation_name(function_name: &Ident) -> String {
    // Convert snake_case function name to PascalCase operation name
    let func_str = function_name.to_string();
    func_str
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn extract_request_type(func: &ItemFn) -> Result<Type> {
    let inputs = &func.sig.inputs;
    
    // Function should have exactly one parameter (the request)
    if inputs.len() != 1 {
        return Err(Error::new_spanned(
            &func.sig,
            "SOAP operation functions must have exactly one parameter (the request type)",
        ));
    }
    
    match inputs.first().unwrap() {
        FnArg::Typed(pat_type) => Ok((*pat_type.ty).clone()),
        FnArg::Receiver(_) => Err(Error::new_spanned(
            &func.sig,
            "SOAP operation functions cannot have self parameters",
        )),
    }
}

fn extract_return_types(func: &ItemFn) -> Result<(Type, Type)> {
    let return_type = match &func.sig.output {
        ReturnType::Default => {
            return Err(Error::new_spanned(
                &func.sig,
                "SOAP operation functions must return Result<ResponseType, ErrorType>",
            ));
        }
        ReturnType::Type(_, ty) => ty,
    };
    
    // Parse Result<T, E> type
    if let Type::Path(TypePath { path, .. }) = return_type.as_ref() {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Result" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() == 2 {
                        let response_type = match &args.args[0] {
                            GenericArgument::Type(ty) => ty.clone(),
                            _ => {
                                return Err(Error::new_spanned(
                                    return_type,
                                    "Invalid Result type: first argument must be a type",
                                ));
                            }
                        };
                        
                        let error_type = match &args.args[1] {
                            GenericArgument::Type(ty) => ty.clone(),
                            _ => {
                                return Err(Error::new_spanned(
                                    return_type,
                                    "Invalid Result type: second argument must be a type",
                                ));
                            }
                        };
                        
                        return Ok((response_type, error_type));
                    }
                }
            }
        }
    }
    
    Err(Error::new_spanned(
        return_type,
        "Function must return Result<ResponseType, ErrorType>",
    ))
}