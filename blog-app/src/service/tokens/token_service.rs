use blog_domain::model::{
    tokens::token::{AccessTokenClaims, RefreshTokenClaims},
    users::user::User,
};
use jsonwebtoken::{errors, Algorithm, DecodingKey, EncodingKey, TokenData, Validation};

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
            std::env::var("REFRESH_TOKEN_SECRET_KEY").expect("undefined REFRESH_TOKEN_SECRET_KEY");
        let encoding_key = EncodingKey::from_secret(secret_key.as_bytes());

        let token_string = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &claims,
            &encoding_key,
        )?;

        Ok(token_string)
    }

    pub fn verify_access_token(
        &self,
        id_token: &str,
    ) -> anyhow::Result<TokenData<AccessTokenClaims>> {
        let secret_key =
            std::env::var("ACCESS_TOKEN_SECRET_KEY").expect("undefined ACCESS_TOKEN_SECRET_KEY");
        let decoding_key = DecodingKey::from_secret(secret_key.as_bytes());
        let validation = {
            let mut validation = Validation::new(Algorithm::HS256);
            let audience = std::env::var("AUDIENCE").expect("undefined AUDIENCE");
            validation.set_audience(&[audience]);
            let issuer = std::env::var("ISSUER").expect("undefined ISSUER");
            validation.set_issuer(&[issuer]);
            validation
        };

        let token_data =
            jsonwebtoken::decode::<AccessTokenClaims>(id_token, &decoding_key, &validation)
                .map_err(|e| match e.into_kind() {
                    errors::ErrorKind::ExpiredSignature => anyhow::anyhow!("expired signature"),
                    _ => anyhow::anyhow!("Unknown error"),
                });

        token_data
    }

    pub fn verify_refresh_token(
        &self,
        refresh_token: &str,
    ) -> anyhow::Result<TokenData<RefreshTokenClaims>> {
        let secret_key =
            std::env::var("REFRESH_TOKEN_SECRET_KEY").expect("undefined REFFRESH_TOKEN_SECRET_KEY");
        let decoding_key = DecodingKey::from_secret(secret_key.as_bytes());
        let validation = {
            let mut validation = Validation::new(Algorithm::HS256);
            let audience = std::env::var("AUDIENCE").expect("undefined AUDIENCE");
            validation.set_audience(&[audience]);
            let issuer = std::env::var("ISSUER").expect("undefined ISSUER");
            validation.set_issuer(&[issuer]);
            validation
        };

        let token_data =
            jsonwebtoken::decode::<RefreshTokenClaims>(refresh_token, &decoding_key, &validation)
                .map_err(|e| match e.into_kind() {
                    errors::ErrorKind::ExpiredSignature => anyhow::anyhow!("expired signature"),
                    _ => anyhow::anyhow!("Unknown error"),
                });

        token_data
    }
}
