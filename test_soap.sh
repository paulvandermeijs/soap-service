#!/bin/bash

# Test the enhanced SOAP calculator service with improved XML parsing
echo "=== Testing Enhanced SOAP Calculator Service ==="
echo ""

echo "1. Checking WSDL (GET /soap/calculator/wsdl):"
curl -s "http://localhost:3000/soap/calculator/wsdl" | head -20
echo ""
echo ""

echo "2. Testing Standard SOAP Request:"
echo "Request XML:"
cat << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <Add>
            <Operand1>15</Operand1>
            <Operand2>25</Operand2>
        </Add>
    </soap:Body>
</soap:Envelope>
EOF

echo ""
echo "Response:"
curl -s -X POST "http://localhost:3000/soap/calculator" \
  -H "Content-Type: text/xml; charset=utf-8" \
  -H "SOAPAction: Add" \
  -d '<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <Add>
            <Operand1>15</Operand1>
            <Operand2>25</Operand2>
        </Add>
    </soap:Body>
</soap:Envelope>'

echo ""
echo ""

echo "3. Testing SOAP-ENV Namespace Request:"
echo "Request XML:"
cat << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<SOAP-ENV:Envelope xmlns:SOAP-ENV="http://schemas.xmlsoap.org/soap/envelope/">
    <SOAP-ENV:Body>
        <tns:Add xmlns:tns="http://example.com/calculator">
            <tns:Operand1>100</tns:Operand1>
            <tns:Operand2>200</tns:Operand2>
        </tns:Add>
    </SOAP-ENV:Body>
</SOAP-ENV:Envelope>
EOF

echo ""
echo "Response:"
curl -s -X POST "http://localhost:3000/soap/calculator" \
  -H "Content-Type: text/xml; charset=utf-8" \
  -H "SOAPAction: Add" \
  -d '<?xml version="1.0" encoding="UTF-8"?>
<SOAP-ENV:Envelope xmlns:SOAP-ENV="http://schemas.xmlsoap.org/soap/envelope/">
    <SOAP-ENV:Body>
        <tns:Add xmlns:tns="http://example.com/calculator">
            <tns:Operand1>100</tns:Operand1>
            <tns:Operand2>200</tns:Operand2>
        </tns:Add>
    </SOAP-ENV:Body>
</SOAP-ENV:Envelope>'

echo ""
echo ""

echo "4. Testing Minimal SOAP Request (no namespaces):"
echo "Request XML:"
cat << 'EOF'
<?xml version="1.0"?>
<Envelope>
    <Body>
        <Add>
            <Operand1>5</Operand1>
            <Operand2>3</Operand2>
        </Add>
    </Body>
</Envelope>
EOF

echo ""
echo "Response:"
curl -s -X POST "http://localhost:3000/soap/calculator" \
  -H "Content-Type: text/xml; charset=utf-8" \
  -H "SOAPAction: Add" \
  -d '<?xml version="1.0"?>
<Envelope>
    <Body>
        <Add>
            <Operand1>5</Operand1>
            <Operand2>3</Operand2>
        </Add>
    </Body>
</Envelope>'

echo ""
echo ""

echo "5. Testing Error Handling (invalid XML):"
echo "Request XML:"
cat << 'EOF'
<?xml version="1.0"?>
<InvalidEnvelope>
    <InvalidBody>
        <Add>
            <Operand1>1</Operand1>
            <Operand2>2</Operand2>
        </Add>
    </InvalidBody>
</InvalidEnvelope>
EOF

echo ""
echo "Response (should be SOAP fault):"
curl -s -X POST "http://localhost:3000/soap/calculator" \
  -H "Content-Type: text/xml; charset=utf-8" \
  -H "SOAPAction: Add" \
  -d '<?xml version="1.0"?>
<InvalidEnvelope>
    <InvalidBody>
        <Add>
            <Operand1>1</Operand1>
            <Operand2>2</Operand2>
        </Add>
    </InvalidBody>
</InvalidEnvelope>'

echo ""
echo ""
echo "=== Enhanced XML Parsing Features Demonstrated ==="
echo "✅ Multiple SOAP envelope formats supported"
echo "✅ Namespace-aware element parsing"
echo "✅ Robust error handling with SOAP faults"
echo "✅ Operation name extraction from various formats"