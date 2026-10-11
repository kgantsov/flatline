use axum::async_trait;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::time::{Duration, timeout};
use tracing::info;

use crate::monitor::checker::{CheckOutcome, Checker, Status};

pub struct TcpChecker {
    hostname: String,
    port: u16,
    timeout: Duration,
}

impl TcpChecker {
    pub fn new(hostname: String, port: u16, timeout: Duration) -> Self {
        Self {
            hostname,
            port,
            timeout,
        }
    }
}

#[async_trait]
impl Checker for TcpChecker {
    async fn check(&self) -> CheckOutcome {
        let start_time = std::time::Instant::now();

        let addr_str = format!("{}:{}", self.hostname, self.port);
        let addr: SocketAddr = match addr_str.parse() {
            Ok(a) => a,
            Err(e) => {
                return CheckOutcome {
                    status: Status::Down,
                    status_code: None,
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    error: Some(format!("Invalid socket address: {}", e)),
                };
            }
        };

        match timeout(self.timeout, TcpStream::connect(addr)).await {
            // Timeout elapsed
            Err(_) => CheckOutcome {
                status: Status::Down,
                status_code: None,
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(String::from("Timeout error")),
            },
            // Connection attempt completed
            Ok(result) => match result {
                Ok(_) => CheckOutcome {
                    status: Status::Up,
                    status_code: None,
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                },
                Err(err) => {
                    info!("TCP connection error: {}", err);
                    CheckOutcome {
                        status: Status::Down,
                        status_code: None,
                        response_time_ms: start_time.elapsed().as_millis() as u64,
                        error: Some(String::from("Connection refused or unreachable")),
                    }
                } // Connection refused or unreachable
            },
        }
    }
}
