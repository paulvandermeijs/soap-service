# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust procedural macro crate that transforms modules into SOAP web services. The `#[service]` macro generates complete SOAP endpoints with Axum routers, XML parsing/serialization, and WSDL generation.

## Development Commands

### Core Library
```bash
# Build the procedural macro
cargo build

# Run all tests (when implemented)
cargo test

# Check for issues
cargo check

# Fix auto-fixable warnings
cargo fix --allow-dirty
```

### Examples
```bash
# Build all examples
cd examples/calculator && cargo build
cd examples/concatenation-service && cargo build

# Run specific examples
cd examples/calculator && cargo run --bin calculator-example      # port 3000
cd examples/calculator && cargo run --bin multiplication-example  # port 3001  
cd examples/concatenation-service && cargo run                    # port 3002

# Test SOAP endpoints
curl -X POST http://localhost:3000/soap/calculator \
  -H "Content-Type: text/xml; charset=utf-8" \
  -H "SOAPAction: \"Add\"" \
  -d '<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
        <soap:Body><Add><Operand1>15</Operand1><Operand2>25</Operand2></Add></soap:Body>
      </soap:Envelope>'

# Access WSDL
curl http://localhost:3000/soap/calculator/wsdl
```

## Architecture

### Procedural Macro Pipeline
1. **Attribute Parsing** (`parser/attributes.rs`): Extracts `#[service(...)]` configuration (namespace, service_name, etc.)
2. **Function Analysis** (`parser/functions.rs`): Validates async functions and extracts SOAP operations  
3. **Type Analysis** (`parser/types.rs`): Analyzes request/response struct types for WSDL generation
4. **Code Generation** (`codegen/wsdl.rs`): Generates WSDL documents and Axum router code
5. **SOAP Processing** (inline in `lib.rs`): Handles XML parsing, SOAP envelope processing, and response generation

### Generated Code Structure
The macro transforms a module like:
```rust
#[service(namespace = "...", service_name = "...", bind_path = "/soap/...")]
mod my_service {
    pub async fn my_operation(req: MyRequest) -> Result<MyResponse, MyError> { ... }
}
```

Into a module with:
- `router()` function returning `axum::Router`
- SOAP request handlers with XML parsing
- WSDL endpoint at `{bind_path}/wsdl`
- Automatic serde-based XML serialization/deserialization

### XML Processing
- Uses `serde_xml_rs` for generic request/response serialization
- Custom SOAP envelope parsing for operation extraction
- Supports mixed field types: integers, floats, strings, booleans, optional fields

### Key Components
- **`ServiceConfig`**: Parsed macro attributes (namespace, service_name, port_name, bind_path)
- **`SoapOperation`**: Function metadata (name, request_type, response_type)  
- **`TypeInfo`**: Type analysis for WSDL generation (name, fields)

## Working Examples

### Calculator (integers)
- `Add` operation: `i32` + `i32` = `i32`
- Port 3000, endpoint `/soap/calculator`

### Multiplication (floats)  
- `Multiply` operation: `f64` × `f64` with optional precision
- Port 3001, endpoint `/soap/math`

### Concatenation (strings)
- `Concatenate` operation: String manipulation with options
- Port 3002, endpoint `/soap/strings`

## Codebase State

This codebase has been extensively cleaned and optimized:
- All unused functions, imports, and code removed
- Generic XML parsing working with multiple data types
- Zero compilation warnings in core library
- All examples tested and functional
- Ready for feature development (Issue #2: Enhanced WSDL generation)