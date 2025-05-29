use soap_service::service;

#[derive(Debug)]
pub struct ServiceError(pub String);

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ServiceError {}

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