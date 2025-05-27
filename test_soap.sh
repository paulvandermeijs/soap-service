#!/bin/bash

# Test the SOAP calculator service
# First, let's check the WSDL

echo "=== Testing SOAP Calculator Service ==="
echo ""

echo "1. Checking WSDL (GET /soap/calculator/wsdl):"
curl -s "http://localhost:3000/soap/calculator/wsdl" | head -20
echo ""

echo "2. Testing SOAP Add operation:"
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