use std::time::Instant;

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::Instrument;
use uuid::Uuid;

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Middleware that attaches a request id to every request and logs access.
///
/// The id is taken from an incoming `x-request-id` header when present,
/// otherwise a fresh UUID (v7) is generated. The request is processed inside a
/// tracing span carrying the `request_id` (plus method and path), so every log
/// line emitted while handling the request is tagged with it. An access log
/// line is emitted when the request starts and when it completes (with response
/// status and latency). The id is also echoed back on the response
/// `x-request-id` header.
pub async fn request_id(req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
        method = %req.method(),
        path = %req.uri().path(),
    );

    async move {
        let start = Instant::now();
        tracing::info!("request started");

        let mut response = next.run(req).await;

        tracing::info!(
            status = response.status().as_u16(),
            latency_ms = start.elapsed().as_millis(),
            "request completed"
        );

        if let Ok(value) = HeaderValue::from_str(&request_id) {
            response.headers_mut().insert(REQUEST_ID_HEADER, value);
        }

        response
    }
    .instrument(span)
    .await
}
