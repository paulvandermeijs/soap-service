# SOAP Service Test Examples

## Improved XML Parsing Features

The enhanced SOAP service now supports:

### 1. Multiple SOAP Envelope Formats
- `<soap:Body>` (standard)
- `<SOAP-ENV:Body>` (SOAP-ENV namespace)
- `<Body>` (no namespace)

### 2. Namespace-aware Element Parsing
- Elements with namespace prefixes: `<tns:Operand1>`
- Elements without namespaces: `<Operand1>`
- Multiple namespace variations: `<ns1:Operand1>`

### 3. XML Entity Decoding
- `&lt;` → `<`
- `&gt;` → `>`
- `&amp;` → `&`
- `&quot;` → `"`
- `&apos;` → `'`

## Test Request Examples

### Basic SOAP Request
```xml
POST /soap/calculator
Content-Type: text/xml; charset=utf-8
SOAPAction: "Add"

<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <Add>
            <Operand1>15</Operand1>
            <Operand2>25</Operand2>
        </Add>
    </soap:Body>
</soap:Envelope>
```

### SOAP-ENV Namespace Request
```xml
<?xml version="1.0" encoding="UTF-8"?>
<SOAP-ENV:Envelope xmlns:SOAP-ENV="http://schemas.xmlsoap.org/soap/envelope/">
    <SOAP-ENV:Body>
        <tns:Add xmlns:tns="http://example.com/calculator">
            <tns:Operand1>100</tns:Operand1>
            <tns:Operand2>200</tns:Operand2>
        </tns:Add>
    </SOAP-ENV:Body>
</SOAP-ENV:Envelope>
```

### Minimal SOAP Request
```xml
<?xml version="1.0"?>
<Envelope>
    <Body>
        <Add>
            <Operand1>5</Operand1>
            <Operand2>3</Operand2>
        </Add>
    </Body>
</Envelope>
```

### Request with XML Entities
```xml
<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <Add>
            <Operand1>10</Operand1>
            <Operand2>5</Operand2>
        </Add>
    </soap:Body>
</soap:Envelope>
```

## Expected Response Format

All requests should return:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
               xmlns:tns="http://example.com/calculator">
    <soap:Body>
        <tns:AddResponse>
            <Result>40</Result>
        </tns:AddResponse>
    </soap:Body>
</soap:Envelope>
```

## Error Handling

Invalid requests return SOAP faults:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <soap:Fault>
            <faultcode>Server</faultcode>
            <faultstring>SOAP Body start tag not found</faultstring>
        </soap:Fault>
    </soap:Body>
</soap:Envelope>
```

## WSDL Endpoint

Access the service definition at:
```
GET http://localhost:3000/soap/calculator/wsdl
```

## Key Improvements Made

1. **Robust SOAP envelope parsing** - Handles multiple namespace variations
2. **Enhanced element extraction** - Supports namespaced and non-namespaced elements
3. **Better error handling** - Descriptive error messages for parsing failures
4. **XML entity decoding** - Properly handles encoded XML content
5. **Operation name extraction** - Intelligently parses operation names from various formats

The implementation now provides a solid foundation for SOAP web services with improved XML parsing capabilities that handle real-world SOAP client variations.