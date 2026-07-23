pub mod handlers;
pub mod middleware;
pub mod oidc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SessionClaims {
    pub sub: String, // user UUID
    pub exp: usize,
}

/// Short-lived claims carrying the OIDC login flow state.
///
/// Stored in a signed `oidc_flow` cookie set on `/auth/login` and read back on
/// `/auth/callback`, so the CSRF/nonce survive restarts and a duplicated
/// callback instead of living in per-process memory.
#[derive(Serialize, Deserialize)]
pub struct FlowClaims {
    pub csrf: String,  // CSRF state token echoed back by the provider
    pub nonce: String, // OIDC nonce, verified against the id_token
    pub exp: usize,
}
