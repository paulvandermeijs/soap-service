//! Parse #[service(...)] attributes

use proc_macro2::TokenStream;
use syn::{parse::Parse, punctuated::Punctuated, Error, Expr, ExprLit, Ident, Lit, Result, Token};

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub namespace: String,
    pub service_name: String,
    pub port_name: String,
    pub bind_path: String,
}

struct ServiceAttribute {
    name: Ident,
    value: String,
}

impl Parse for ServiceAttribute {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let expr: Expr = input.parse()?;
        
        if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = expr {
            Ok(ServiceAttribute {
                name,
                value: lit_str.value(),
            })
        } else {
            Err(Error::new_spanned(expr, "Expected string literal"))
        }
    }
}

struct ServiceAttributes {
    attributes: Punctuated<ServiceAttribute, Token![,]>,
}

impl Parse for ServiceAttributes {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(ServiceAttributes {
            attributes: input.parse_terminated(ServiceAttribute::parse, Token![,])?,
        })
    }
}

/// Parses the #[service(...)] attribute arguments into a validated ServiceConfig.
/// 
/// Expects namespace, service_name, port_name, and bind_path attributes.
/// Validates that all required fields are present and properly formatted.
pub fn parse_service_attributes(args: TokenStream) -> Result<ServiceConfig> {
    let parsed = syn::parse2::<ServiceAttributes>(args)?;
    
    let mut namespace = None;
    let mut service_name = None;
    let mut port_name = None;
    let mut bind_path = None;
    
    for attr in parsed.attributes {
        match attr.name.to_string().as_str() {
            "namespace" => {
                validate_namespace(&attr.value)?;
                namespace = Some(attr.value);
            }
            "service_name" => {
                validate_identifier(&attr.value, "service_name")?;
                service_name = Some(attr.value);
            }
            "port_name" => {
                validate_identifier(&attr.value, "port_name")?;
                port_name = Some(attr.value);
            }
            "bind_path" => {
                validate_bind_path(&attr.value)?;
                bind_path = Some(attr.value);
            }
            _ => {
                return Err(Error::new_spanned(
                    &attr.name,
                    format!("Unknown attribute: {}", attr.name),
                ));
            }
        }
    }
    
    // Ensure all required fields are present
    let namespace = namespace.ok_or_else(|| {
        Error::new(proc_macro2::Span::call_site(), "Missing required attribute: namespace")
    })?;
    let service_name = service_name.ok_or_else(|| {
        Error::new(proc_macro2::Span::call_site(), "Missing required attribute: service_name")
    })?;
    let port_name = port_name.ok_or_else(|| {
        Error::new(proc_macro2::Span::call_site(), "Missing required attribute: port_name")
    })?;
    let bind_path = bind_path.ok_or_else(|| {
        Error::new(proc_macro2::Span::call_site(), "Missing required attribute: bind_path")
    })?;
    
    Ok(ServiceConfig {
        namespace,
        service_name,
        port_name,
        bind_path,
    })
}

/// Validates that the namespace is a proper URI starting with http:// or https://.
fn validate_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty() {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "Namespace cannot be empty",
        ));
    }
    
    // Basic URI validation - should start with http:// or https://
    if !namespace.starts_with("http://") && !namespace.starts_with("https://") {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "Namespace must be a valid URI starting with http:// or https://",
        ));
    }
    
    Ok(())
}

/// Validates that a value is a proper identifier (alphanumeric + underscores, starts with letter/underscore).
fn validate_identifier(value: &str, field_name: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            format!("{} cannot be empty", field_name),
        ));
    }
    
    // Check if it's a valid identifier (starts with letter/underscore, contains alphanumeric/underscore)
    if !value.chars().next().unwrap_or('\0').is_ascii_alphabetic() && !value.starts_with('_') {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            format!("{} must start with a letter or underscore", field_name),
        ));
    }
    
    if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            format!("{} must contain only alphanumeric characters and underscores", field_name),
        ));
    }
    
    Ok(())
}

/// Validates that the bind path starts with '/' and is not just the root path.
fn validate_bind_path(path: &str) -> Result<()> {
    if !path.starts_with('/') {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "bind_path must start with '/'",
        ));
    }
    
    if path == "/" {
        return Err(Error::new(
            proc_macro2::Span::call_site(),
            "bind_path cannot be just '/'",
        ));
    }
    
    Ok(())
}