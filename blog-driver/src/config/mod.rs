pub mod cookie_config;

pub use cookie_config::CookieConfig;

/// Application-wide configuration loaded once at startup and passed as an argument to each process.
pub struct AppConfig {
    /// Database connection URL. Passed to `PgPool` for establishing the connection.
    pub database_url: String,
    /// CA certificate for SSL database connections (PEM format).
    /// Empty string means connecting without SSL.
    pub db_ca_cert: String,
    /// List of allowed client origins for CORS. Passed to `CorsLayer::allow_origin`.
    pub client_addrs: Vec<String>,
    /// Master key used to sign and verify encrypted cookies. Passed to `axum_extra::extract::cookie::Key`.
    pub master_key: String,
    /// Address the server binds to (e.g. `0.0.0.0:8000`). Passed to `TcpListener::bind`.
    pub internal_api_domain: String,
    /// Maximum request body size in bytes. Passed to `DefaultBodyLimit`.
    pub max_request_body_size: usize,
    /// Cookie attribute settings (SameSite, Secure, etc.). Passed to `CookieService`.
    /// Derived from the `ENVIRONMENT` variable.
    pub cookie_config: CookieConfig,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        let app_env = std::env::var("ENVIRONMENT").expect("Undefined ENVIRONMENT");

        let client_addrs_str = std::env::var("CLIENT_ADDRS").expect("Undefined CLIENT_ADDRS");
        let client_addrs = client_addrs_str
            .split(",")
            .collect::<Vec<&str>>()
            .iter()
            .map(|addr| addr.to_string())
            .collect::<Vec<String>>();

        AppConfig {
            database_url: std::env::var("DATABASE_URL").expect("Undefined DATABASE_URL"),
            db_ca_cert: std::env::var("DB_CA_CERT").unwrap_or_default(),
            client_addrs,
            master_key: std::env::var("MASTER_KEY").expect("Undefined MASTER_KEY"),
            internal_api_domain: std::env::var("INTERNAL_API_DOMAIN")
                .expect("Undefined INTERNAL_API_DOMAIN"),
            max_request_body_size: std::env::var("MAX_REQUEST_BODY_SIZE")
                .expect("Undefined MAX_REQUEST_BODY_SIZE")
                .parse::<usize>()
                .unwrap(),
            cookie_config: CookieConfig::for_environment(&app_env),
        }
    }
}
