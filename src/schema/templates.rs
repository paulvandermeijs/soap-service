//! WSDL template generation

pub fn wsdl_template() -> &'static str {
    // TODO: Return WSDL template with placeholders
    r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://schemas.xmlsoap.org/wsdl/"
             xmlns:tns="{{namespace}}"
             xmlns:soap="http://schemas.xmlsoap.org/wsdl/soap/"
             xmlns:xsd="http://www.w3.org/2001/XMLSchema"
             targetNamespace="{{namespace}}">
    <!-- TODO: Complete WSDL template -->
</definitions>"#
}