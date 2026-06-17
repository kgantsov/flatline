use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{Validation, decode};
use uuid::Uuid;

use crate::{AppState, auth::SessionClaims, error::ApiError};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let headers = req.headers();
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| {
            headers
                .get(header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies
                        .split(';')
                        .find_map(|c| c.trim().strip_prefix("session="))
                })
        })
        .ok_or(ApiError::Unauthorized)?;

    let data = decode::<SessionClaims>(token, &state.jwt_decoding_key, &Validation::default())
        .map_err(|_| ApiError::Unauthorized)?;

    let user_id = Uuid::parse_str(&data.claims.sub).map_err(|_| ApiError::Unauthorized)?;
    req.extensions_mut().insert(user_id);
    Ok(next.run(req).await)
}
