use axum::async_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Unknown,
    Up,
    Down,
}

pub struct CheckOutcome {
    pub status: Status, // Up / Down
    pub status_code: Option<u16>,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

#[async_trait]
pub trait Checker: Send + Sync {
    async fn check(&self) -> CheckOutcome;
}
