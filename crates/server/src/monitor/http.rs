use axum::async_trait;
use reqwest::{Client, Method};
use shared::models::HttpMethod;

use crate::monitor::checker::{CheckOutcome, Checker, Status};

pub struct HttpChecker {
    client: Client,
    url: String,
    method: Method,
    expected_status: Vec<u16>,
}

impl HttpChecker {
    pub fn new(url: String, method: Option<HttpMethod>, expected_status: Option<Vec<u16>>) -> Self {
        let method = match method.unwrap_or(HttpMethod::Get) {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Patch => Method::PATCH,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Options => Method::OPTIONS,
        };
        Self {
            client: Client::new(),
            url,
            method,
            expected_status: expected_status.unwrap_or_else(|| vec![200]),
        }
    }
}

#[async_trait]
impl Checker for HttpChecker {
    async fn check(&self) -> CheckOutcome {
        let start_time = std::time::Instant::now();
        let req = self.client.request(self.method.clone(), &self.url);

        match req.send().await {
            Ok(response) => {
                let status_code = response.status().as_u16();
                if self.expected_status.contains(&status_code) {
                    CheckOutcome {
                        status: Status::Up,
                        status_code: Some(status_code),
                        response_time_ms: start_time.elapsed().as_millis() as u64,
                        error: None,
                    }
                } else {
                    CheckOutcome {
                        status: Status::Down,
                        status_code: Some(status_code),
                        response_time_ms: start_time.elapsed().as_millis() as u64,
                        error: Some(format!("Unexpected status code: {}", status_code)),
                    }
                }
            }
            Err(e) => CheckOutcome {
                status: Status::Down,
                status_code: None,
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(format!("Request failed: {}", e)),
            },
        }
    }
}
