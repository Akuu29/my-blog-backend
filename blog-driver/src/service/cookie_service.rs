use crate::utils::cookie_builder::CookieBuilder;
use axum_extra::extract::cookie::PrivateCookieJar;
pub struct CookieService {
    cookie_builder: CookieBuilder,
}

impl CookieService {
    pub fn new() -> Self {
        Self {
            cookie_builder: CookieBuilder::from_env(),
        }
    }

    pub fn set_refresh_token(&self, jar: PrivateCookieJar, token_value: &str) -> PrivateCookieJar {
        let url_encoded_token = urlencoding::encode(token_value).into_owned();
        let cookie = self
            .cookie_builder
            .build_custom_cookie("refresh_token", &url_encoded_token);

        jar.add(cookie)
    }

    pub fn clear_refresh_token(&self, jar: PrivateCookieJar) -> PrivateCookieJar {
        let cookie = self.cookie_builder.build_custom_cookie("refresh_token", "");

        jar.remove(cookie)
    }

    pub fn get_refresh_token(&self, jar: &PrivateCookieJar) -> anyhow::Result<String> {
        match jar.get("refresh_token") {
            Some(cookie) => match urlencoding::decode(cookie.value()) {
                Ok(decoded_token) => Ok(decoded_token.to_string()),
                Err(e) => Err(e.into()),
            },
            None => Err(anyhow::anyhow!("Not found refresh token")),
        }
    }
}
