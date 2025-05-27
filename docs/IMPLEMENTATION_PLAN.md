# SOAP Service Macro Implementation Plan

## Project Overview

Create a Rust procedural macro crate `soap-service` that transforms modules
containing async functions into SOAP web services with automatic Axum router
generation and WSDL endpoint creation.

## Project Structure

```
soap-service/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Main macro entry point and public API
│   ├── parser/
│   │   ├── mod.rs          # Module parsing orchestration
│   │   ├── attributes.rs   # Parse #[service(...)] attributes
│   │   ├── functions.rs    # Parse and validate async functions
│   │   └── types.rs        # Analyze request/response struct types
│   ├── codegen/
│   │   ├── mod.rs          # Code generation orchestration
│   │   ├── router.rs       # Generate Axum router code
│   │   ├── handlers.rs     # Generate SOAP request handlers
│   │   └── wsdl.rs         # Generate WSDL document and endpoint
│   ├── soap/
│   │   ├── mod.rs          # SOAP processing utilities
│   │   ├── envelope.rs     # SOAP envelope parsing/generation
│   │   ├── serialization.rs # XML serialization helpers
│   │   └── faults.rs       # SOAP fault generation
│   └── schema/
│       ├── mod.rs          # XML Schema generation
│       ├── types.rs        # Rust type to XSD mapping
│       └── templates.rs    # WSDL template generation
├── examples/
│   ├── calculator/
│   │   ├── Cargo.toml
│   │   └── src/main.rs     # Complete working example
│   └── multi-service/
│       ├── Cargo.toml
│       └── src/main.rs     # Multiple services example
└── tests/
    ├── integration/
    │   ├── basic_service.rs
    │   ├── multiple_services.rs
    │   └── error_handling.rs
    └── unit/
        ├── parser_tests.rs
        ├── codegen_tests.rs
        └── soap_tests.rs
```

## Phase 1: Project Setup and Dependencies

### Cargo.toml Configuration

```toml
[package]
name = "soap-service"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
# Proc macro essentials
proc-macro2 = "1.0"
syn = { version = "2.0", features = ["full", "extra-traits", "visit"] }
quote = "1.0"

# XML processing
serde = { version = "1.0", features = ["derive"] }
serde-xml-rs = "0.6"
quick-xml = { version = "0.31", features = ["serialize"] }

# Optional runtime dependencies (for generated code)
axum = { version = "0.7", optional = true }
tokio = { version = "1.0", features = ["full"], optional = true }

[features]
default = ["runtime"]
runtime = ["axum", "tokio"]

[dev-dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"
hyper = "1.0"
```

### Initial File Structure Creation

Create all directories and stub files with basic module declarations and TODO
comments.

## Phase 2: Attribute Parsing (src/parser/attributes.rs)

### Parse Service Attributes

```rust
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub namespace: String,
    pub service_name: String,
    pub port_name: String,
    pub bind_path: String,
}

pub fn parse_service_attributes(args: TokenStream) -> Result<ServiceConfig, syn::Error> {
    // Parse: namespace = "...", service_name = "...", etc.
    // Validate required fields are present
    // Return structured config
}
```

### Validation Rules

- `namespace` must be a valid URI
- `service_name` and `port_name` must be valid identifiers
- `bind_path` must start with `/`

## Phase 3: Function Analysis (src/parser/functions.rs)

### Function Signature Validation

```rust
#[derive(Debug, Clone)]
pub struct SoapOperation {
    pub name: String,
    pub function_name: syn::Ident,
    pub request_type: syn::Type,
    pub response_type: syn::Type,
    pub error_type: syn::Type,
}

pub fn extract_soap_operations(module: &syn::ItemMod) -> Result<Vec<SoapOperation>, syn::Error> {
    // Find all `pub async fn` functions
    // Validate signature: fn_name(RequestType) -> Result<ResponseType, ErrorType>
    // Extract type information
    // Generate operation names from function names
}
```

### Function Requirements Check

- Must be `pub async fn`
- Exactly one parameter (request struct)
- Return type must be `Result<T, E>`
- Request/Response types must be in scope

## Phase 4: Type System Analysis (src/parser/types.rs)

### Type Information Extraction

```rust
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub xml_name: String, // From serde rename attribute
    pub field_type: FieldType,
    pub optional: bool,
}

pub fn analyze_type(ty: &syn::Type) -> Result<TypeInfo, syn::Error> {
    // Parse struct definitions
    // Extract field information
    // Handle serde attributes (#[serde(rename = "...")])
    // Detect optional fields (Option<T>)
}
```

## Phase 5: SOAP Envelope Processing (src/soap/envelope.rs)

### Request Processing

```rust
pub struct SoapRequest {
    pub operation: String,
    pub body: String,
    pub headers: std::collections::HashMap<String, String>,
}

pub fn parse_soap_request(xml: &str) -> Result<SoapRequest, SoapError> {
    // Parse SOAP envelope
    // Extract operation name from SOAPAction header or body
    // Extract body content for deserialization
}

pub fn create_soap_response<T: serde::Serialize>(
    result: &T,
    operation: &str,
    namespace: &str
) -> Result<String, SoapError> {
    // Generate SOAP response envelope
    // Serialize response data to XML
    // Include proper namespaces
}
```

### Error Handling

```rust
pub fn create_soap_fault(error: &dyn std::fmt::Display) -> String {
    // Generate SOAP fault response
    // Include error message in faultstring
    // Use standard SOAP fault structure
}
```

## Phase 6: WSDL Generation (src/codegen/wsdl.rs)

### Schema Generation

```rust
pub fn generate_wsdl(
    config: &ServiceConfig,
    operations: &[SoapOperation],
    types: &[TypeInfo]
) -> String {
    // Generate complete WSDL document
    // Include type definitions (XSD schema)
    // Define messages, port types, bindings, services
    // Use proper SOAP 1.1/1.2 bindings
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
```

## Phase 7: Code Generation (src/codegen/router.rs)

### Router Generation

```rust
pub fn generate_router_function(
    config: &ServiceConfig,
    operations: &[SoapOperation]
) -> proc_macro2::TokenStream {
    quote! {
        pub fn router() -> axum::Router {
            axum::Router::new()
                .route(#bind_path, axum::routing::post(soap_handler))
                .route(&format!("{}/wsdl", #bind_path), axum::routing::get(wsdl_handler))
        }

        async fn soap_handler(body: String) -> axum::response::Response {
            // Generated SOAP request dispatcher
        }

        async fn wsdl_handler() -> axum::response::Response {
            // Return generated WSDL
        }
    }
}
```

### Operation Dispatcher

```rust
pub fn generate_dispatcher(operations: &[SoapOperation]) -> proc_macro2::TokenStream {
    // Generate match statement routing operations to functions
    // Handle deserialization and serialization
    // Include error handling with SOAP faults
}
```

## Phase 8: Main Macro Implementation (src/lib.rs)

```rust
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let config = match parser::attributes::parse_service_attributes(args.into()) {
        Ok(config) => config,
        Err(e) => return e.to_compile_error().into(),
    };

    let module = match syn::parse::<syn::ItemMod>(input.clone()) {
        Ok(module) => module,
        Err(e) => return e.to_compile_error().into(),
    };

    let operations = match parser::functions::extract_soap_operations(&module) {
        Ok(ops) => ops,
        Err(e) => return e.to_compile_error().into(),
    };

    let enhanced_module = codegen::generate_enhanced_module(
        module,
        config,
        operations
    );

    enhanced_module.into()
}
```

## Phase 9: Example Implementation

### Calculator Service Example

```rust
// examples/calculator/src/main.rs
use soap_service::service;

#[derive(Debug)]
pub struct ServiceError(pub String);

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[service(
    namespace = "http://example.com/calculator",
    service_name = "CalculatorService",
    port_name = "CalculatorPort",
    bind_path = "/soap/calculator"
)]
mod calculator {
    use super::ServiceError;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct AddRequest {
        #[serde(rename = "Operand1")]
        pub a: i32,
        #[serde(rename = "Operand2")]
        pub b: i32,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct AddResponse {
        #[serde(rename = "Result")]
        pub sum: i32,
    }

    pub async fn add(req: AddRequest) -> Result<AddResponse, ServiceError> {
        if req.a == 0 && req.b == 0 {
            return Err(ServiceError("Cannot add two zeros".to_string()));
        }
        Ok(AddResponse { sum: req.a + req.b })
    }
}

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .merge(calculator::router())
        .route("/health", axum::routing::get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Calculator SOAP Service running on http://localhost:3000");
    println!("WSDL available at: http://localhost:3000/soap/calculator/wsdl");

    axum::serve(listener, app).await.unwrap();
}
```

## Phase 10: Testing Strategy

### Unit Tests

- Attribute parsing with various configurations
- Function signature validation
- Type analysis accuracy
- WSDL generation correctness
- SOAP envelope parsing/generation

### Integration Tests

- Complete service generation and compilation
- SOAP request/response roundtrip
- Multiple services in one application
- Error handling and SOAP fault generation
- WSDL validation against standard tools

### Example-based Testing

- Ensure examples compile and run
- Test with real SOAP clients
- Performance benchmarks

## Implementation Order

1. **Phase 1-2**: Setup project structure and attribute parsing
2. **Phase 3**: Function analysis and validation
3. **Phase 4**: Type system analysis with serde integration
4. **Phase 5**: Basic SOAP envelope processing
5. **Phase 6**: WSDL generation for simple types
6. **Phase 7**: Code generation for router and handlers
7. **Phase 8**: Main macro integration and testing
8. **Phase 9**: Complete working example
9. **Phase 10**: Comprehensive testing and documentation

## Success Criteria

- [ ] Macro compiles without errors
- [ ] Generated code compiles without warnings
- [ ] Calculator example runs and handles SOAP requests
- [ ] WSDL validates against standard WSDL validators
- [ ] SOAP clients can successfully call generated services
- [ ] Error handling produces valid SOAP faults
- [ ] Multiple services can coexist in one application
- [ ] Documentation is complete with examples

## Deliverables

1. Complete `soap-service` crate ready for publication
2. Working calculator example demonstrating all features
3. Comprehensive test suite
4. Documentation with usage examples
5. README with quick start guide
