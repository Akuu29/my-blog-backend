use blog_domain::model::{
    tokens::token::{AccessTokenClaims, RefreshTokenClaims},
    users::user::User,
};
use jsonwebtoken::{Algorithm, EncodingKey};

#[derive(Default)]
pub struct TokenService {}

impl TokenService {
    pub fn generate_access_token(&self, user: &User) -> anyhow::Result<String> {
        let claims = AccessTokenClaims::new(user);

        let secret_key =
            std::env::var("ACCESS_TOKEN_SECRET_KEY").expect("undefined ACCESS_TOKEN_SECRET_KEY");
        let encoding_key = EncodingKey::from_secret(secret_key.as_bytes());
        let token_string = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &claims,
            &encoding_key,
        )?;

        Ok(token_string)
    }

    pub fn generate_refresh_token(&self, user: &User) -> anyhow::Result<String> {
        let claims = RefreshTokenClaims::new(user);

        let secret_key =
            std::env::var("ACCESS_TOKEN_SECRET_KEY").expect("undefined ACCESS_TOKEN_SECRET_KEY");
        let encoding_key = EncodingKey::from_secret(secret_key.as_bytes());

        let token_string = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &claims,
            &encoding_key,
        )?;

        Ok(token_string)
    }
}
