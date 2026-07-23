use axum::{
    Extension, Json,
    extract::{Query, State},
    http::{HeaderMap, header},
    response::{AppendHeaders, IntoResponse, Redirect},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{Header, Validation, decode, encode};
use openidconnect::{
    AuthorizationCode, CsrfToken, Nonce, Scope, TokenResponse, core::CoreAuthenticationFlow,
};
use serde::Deserialize;
use shared::models::User;
use uuid::Uuid;

use crate::{
    AppState,
    auth::{FlowClaims, SessionClaims},
    error::ApiError,
};

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

pub async fn login(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
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

    // Carry the CSRF/nonce in a short-lived signed cookie instead of process
    // memory, so the callback works across restarts and tolerates a duplicated
    // or replayed callback request.
    let flow_claims = FlowClaims {
        csrf: csrf_token.secret().clone(),
        nonce: nonce.secret().clone(),
        exp: (Utc::now() + Duration::minutes(10)).timestamp() as usize,
    };
    let flow_jwt = encode(&Header::default(), &flow_claims, &state.jwt_encoding_key)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let cookie = format!("oidc_flow={flow_jwt}; HttpOnly; Path=/; SameSite=Lax; Max-Age=600");
    Ok((
        [
            (header::SET_COOKIE, cookie),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        Redirect::to(auth_url.as_str()),
    ))
}

pub async fn callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<CallbackParams>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Read + verify the signed flow cookie, then check the CSRF state
    let flow_jwt = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find_map(|c| c.trim().strip_prefix("oidc_flow="))
        })
        .ok_or_else(|| ApiError::BadRequest("invalid or expired state".into()))?;

    let flow = decode::<FlowClaims>(flow_jwt, &state.jwt_decoding_key, &Validation::default())
        .map_err(|_| ApiError::BadRequest("invalid or expired state".into()))?
        .claims;

    if flow.csrf != params.state {
        return Err(ApiError::BadRequest("invalid or expired state".into()));
    }
    let nonce = Nonce::new(flow.nonce);

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

    let session_cookie = format!("session={jwt}; HttpOnly; Path=/; SameSite=Lax; Max-Age=2592000");
    let clear_flow = "oidc_flow=; HttpOnly; Path=/; Max-Age=0".to_string();
    Ok((
        AppendHeaders([
            (header::SET_COOKIE, session_cookie),
            (header::SET_COOKIE, clear_flow),
        ]),
        Redirect::to("/"),
    ))
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
