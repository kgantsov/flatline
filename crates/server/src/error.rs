use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
    Forbidden(String),
}

#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, Json(ErrorBody { error: msg })).into_response()
            }
            ApiError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, Json(ErrorBody { error: msg })).into_response()
            }

            ApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(ErrorBody { error: msg })).into_response()
            }

            ApiError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody { error: msg }),
            )
                .into_response(),
        }
    }
}
