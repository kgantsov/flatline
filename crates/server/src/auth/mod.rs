pub mod handlers;
pub mod middleware;
pub mod oidc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SessionClaims {
    pub sub: String, // user UUID
    pub exp: usize,
}
