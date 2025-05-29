use soap_service::service;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct ServiceError(pub String);

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ServiceError {}

#[service(
    namespace = "http://example.com/strings",
    service_name = "StringService",
    port_name = "StringPort",
    bind_path = "/soap/strings"
)]
mod string_service {
    use super::ServiceError;
    use serde::{Deserialize, Serialize};

    // String concatenation service demonstrating various field types
    #[derive(Deserialize, Serialize, Debug)]
    pub struct ConcatenateRequest {
        #[serde(rename = "FirstText")]
        pub first: String,
        #[serde(rename = "SecondText")]
        pub second: String,
        #[serde(rename = "Separator")]
        pub separator: Option<String>,
        #[serde(rename = "UpperCase")]
        pub uppercase: bool,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct ConcatenateResponse {
        #[serde(rename = "Result")]
        pub result: String,
        #[serde(rename = "Length")]
        pub length: i32,
        #[serde(rename = "WordCount")]
        pub word_count: i32,
    }

    pub async fn concatenate(req: ConcatenateRequest) -> Result<ConcatenateResponse, ServiceError> {
        if req.first.is_empty() && req.second.is_empty() {
            return Err(ServiceError("Both texts cannot be empty".to_string()));
        }

        let separator = req.separator.unwrap_or(" ".to_string());
        let mut result = format!("{}{}{}", req.first, separator, req.second);
        
        if req.uppercase {
            result = result.to_uppercase();
        }

        let word_count = result.split_whitespace().count() as i32;

        Ok(ConcatenateResponse {
            length: result.len() as i32,
            result,
            word_count,
        })
    }
}

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .merge(string_service::router())
        .route("/health", axum::routing::get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();

    println!("Concatenation SOAP Service running on http://localhost:3002");
    println!("WSDL available at: http://localhost:3002/soap/strings/wsdl");

    axum::serve(listener, app).await.unwrap();
}