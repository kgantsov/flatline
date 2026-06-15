#[derive(Clone)]
pub struct Config {
    pub database_url: String,
}

// parse env variables and init Config
pub fn init_config() -> Config {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:flatline.db".to_string());

    Config { database_url }
}
