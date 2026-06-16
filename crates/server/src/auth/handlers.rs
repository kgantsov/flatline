use axum::{
    Extension, Json,
    extract::{Query, State},
    http::header,
    response::{IntoResponse, Redirect},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Header, encode};
use openidconnect::{
    AuthorizationCode, CsrfToken, Nonce, Scope, TokenResponse, core::CoreAuthenticationFlow,
};
use serde::Deserialize;
use shared::models::User;
use std::time::Instant;
use uuid::Uuid;

use crate::{AppState, auth::SessionClaims, error::ApiError};

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

pub async fn login(State(state): State<AppState>) -> impl IntoResponse {
    let (auth_url, csrf_token, nonce) = state
        .oidc_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".into()))
        .add_scope(Scope::new("email".into()))
        .add_scope(Scope::new("profile".into()))
        .url();

    state
        .pending_auth
        .insert(csrf_token.secret().clone(), (nonce, Instant::now()));
    Redirect::to(auth_url.as_str())
}

pub async fn callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Pop + verify CSRF state
    let (nonce, _) = state
        .pending_auth
        .remove(&params.state)
        .ok_or_else(|| ApiError::BadRequest("invalid or expired state".into()))?
        .1;

    // 2. Exchange code for tokens
    let token_response = state
        .oidc_client
        .exchange_code(AuthorizationCode::new(params.code))
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .request_async(&state.http_client)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    // 3. Verify ID token + nonce
    let id_token = token_response
        .id_token()
        .ok_or_else(|| ApiError::BadRequest("no id_token in response".into()))?;
    let claims = id_token
        .claims(&state.oidc_client.id_token_verifier(), &nonce)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let sub = claims.subject().as_str().to_string();
    let email = claims.email().map(|e| e.as_str().to_string());
    let name = claims
        .name()
        .and_then(|n| n.get(None))
        .map(|n| n.as_str().to_string());

    // 4. First-user logic
    let user = match state.users.find_by_sub(&sub).await? {
        Some(u) => u,
        None => {
            let count = state.users.count().await?;
            if count > 0 {
                return Err(ApiError::Forbidden("registration is closed".into()));
            }
            let u = User {
                id: Uuid::now_v7(),
                sub,
                email,
                name,
                created_at: Utc::now(),
            };
            state.users.create(&u).await?;
            u
        }
    };

    // 5. Issue JWT session cookie (30-day)
    let session_claims = SessionClaims {
        sub: user.id.to_string(),
        exp: (Utc::now() + Duration::days(30)).timestamp() as usize,
    };
    let jwt = encode(&Header::default(), &session_claims, &state.jwt_encoding_key)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let cookie = format!("session={jwt}; HttpOnly; Path=/; SameSite=Lax; Max-Age=2592000");
    Ok(([(header::SET_COOKIE, cookie)], Redirect::to("/")))
}

pub async fn logout() -> impl IntoResponse {
    (
        [(header::SET_COOKIE, "session=; HttpOnly; Path=/; Max-Age=0")],
        Redirect::to("/"),
    )
}

pub async fn me(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<User>, ApiError> {
    let user = state
        .users
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("user not found".into()))?;
    Ok(Json(user))
}
