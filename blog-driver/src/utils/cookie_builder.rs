use crate::config::CookieConfig;
use cookie::{time::Duration, Cookie};

pub struct CookieBuilder {
    config: CookieConfig,
}

impl CookieBuilder {
    pub fn new(config: CookieConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> Self {
        Self::new(CookieConfig::from_env())
    }

    pub fn build_custom_cookie(&self, name: &str, value: &str) -> cookie::Cookie<'static> {
        Cookie::build((name.to_owned(), value.to_owned()))
            .http_only(self.config.http_only)
            .max_age(Duration::days(self.config.max_age_days))
            .path(self.config.path.clone())
            .same_site(self.config.same_site.to_same_site())
            .secure(self.config.secure)
            .build()
    }
}
