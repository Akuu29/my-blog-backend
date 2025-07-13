use cookie::SameSite;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct CookieConfig {
    pub http_only: bool,
    pub secure: bool,
    pub same_site: SameSiteConfig,
    pub max_age_days: i64,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum SameSiteConfig {
    #[serde(rename = "strict")]
    Strict,
    #[serde(rename = "lax")]
    Lax,
    #[serde(rename = "none")]
    None,
}

impl SameSiteConfig {
    pub fn to_same_site(&self) -> SameSite {
        match self {
            SameSiteConfig::Strict => SameSite::Strict,
            SameSiteConfig::Lax => SameSite::Lax,
            SameSiteConfig::None => SameSite::None,
        }
    }
}

impl CookieConfig {
    pub fn from_env() -> Self {
        let environment = env::var("ENVIRONMENT").expect("Undefined ENVIRONMENT");

        match environment.as_str() {
            "prd" => Self::production(),
            "stg" => Self::staging(),
            "dev" => Self::development(),
            _ => panic!("Invalid environment: {}", environment),
        }
    }

    pub fn development() -> Self {
        Self {
            http_only: true,
            secure: false,
            same_site: SameSiteConfig::Lax,
            max_age_days: 30,
            path: "/".to_string(),
        }
    }

    pub fn staging() -> Self {
        Self {
            http_only: true,
            secure: true,
            same_site: SameSiteConfig::None,
            max_age_days: 30,
            path: "/".to_string(),
        }
    }

    pub fn production() -> Self {
        Self {
            http_only: true,
            secure: true,
            same_site: SameSiteConfig::None,
            max_age_days: 30,
            path: "/".to_string(),
        }
    }

    // pub fn from_config_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
    //     let content = std::fs::read_to_string(path)?;
    //     let config = serde_json::from_str::<Self>(&content)?;
    //     Ok(config)
    // }
}
