#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub oauth_issuer_url: String,
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_redirect_url: String,
    pub jwt_secret: String,
}

// parse env variables and init Config
pub fn init_config() -> Config {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:flatline.db".to_string());
    let oauth_issuer_url =
        std::env::var("OAUTH_ISSUER_URL").expect("OAUTH_ISSUER_URL must be set");
    let oauth_client_id = std::env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID must be set");
    let oauth_client_secret =
        std::env::var("OAUTH_CLIENT_SECRET").expect("OAUTH_CLIENT_SECRET must be set");
    let oauth_redirect_url =
        std::env::var("OAUTH_REDIRECT_URL").expect("OAUTH_REDIRECT_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    Config {
        database_url,
        oauth_issuer_url,
        oauth_client_id,
        oauth_client_secret,
        oauth_redirect_url,
        jwt_secret,
    }
}
