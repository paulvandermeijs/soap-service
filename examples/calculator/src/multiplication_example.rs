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
    namespace = "http://example.com/math",
    service_name = "MathService", 
    port_name = "MathPort",
    bind_path = "/soap/math"
)]
mod math_service {
    use super::ServiceError;
    use serde::{Deserialize, Serialize};

    // Different struct names and field names to show genericity
    #[derive(Deserialize, Serialize, Debug)]
    pub struct MultiplyRequest {
        #[serde(rename = "FirstNumber")]
        pub x: f64,
        #[serde(rename = "SecondNumber")]
        pub y: f64,
        #[serde(rename = "Precision")]
        pub precision: Option<u8>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    pub struct MultiplyResponse {
        #[serde(rename = "Product")]
        pub result: f64,
        #[serde(rename = "Calculation")]
        pub formula: String,
    }

    pub async fn multiply(req: MultiplyRequest) -> Result<MultiplyResponse, ServiceError> {
        let precision = req.precision.unwrap_or(2);
        let result = req.x * req.y;
        
        Ok(MultiplyResponse {
            result: (result * 10_f64.powi(precision as i32)).round() / 10_f64.powi(precision as i32),
            formula: format!("{} × {} = {:.precision$}", req.x, req.y, result, precision = precision as usize),
        })
    }
}

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .merge(math_service::router())
        .route("/health", axum::routing::get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();

    println!("Math SOAP Service running on http://localhost:3001");
    println!("WSDL available at: http://localhost:3001/soap/math/wsdl");

    axum::serve(listener, app).await.unwrap();
}