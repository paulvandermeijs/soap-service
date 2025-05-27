//! SOAP fault generation

pub fn create_soap_fault(_error: &dyn std::fmt::Display) -> String {
    // TODO: Generate SOAP fault response
    // TODO: Include error message in faultstring
    // TODO: Use standard SOAP fault structure
    r#"<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <soap:Fault>
            <faultcode>Server</faultcode>
            <faultstring>Not implemented</faultstring>
        </soap:Fault>
    </soap:Body>
</soap:Envelope>"#
        .to_string()
}