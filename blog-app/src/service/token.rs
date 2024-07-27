use crate::{
    model::token::{AccessTokenClaims, IdTokenClaims, RefreshTokenClaims},
    repository::token::TokenRepository,
};
use jsonwebtoken::{errors, Algorithm, DecodingKey, EncodingKey, TokenData, Validation};

#[derive(Clone)]
pub struct TokenService<T: TokenRepository> {
    repository: T,
}

impl<T: TokenRepository> TokenService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn verify_id_token(&self, token: &str) -> anyhow::Result<TokenData<IdTokenClaims>> {
        let jwks = self.repository.fetch_jwks().await?;

        let token_header = jsonwebtoken::decode_header(token)?;

        let kid = token_header.kid.unwrap();
        let pem = jwks.get(&kid).unwrap();
        let decoding_key = DecodingKey::from_rsa_pem(pem.as_bytes())?;

        let validation = {
            let mut validation = Validation::new(Algorithm::RS256);
            let audience =
                std::env::var("FIREBASE_PROJECT_ID").expect("undefined FIREBASE_PROJECT_ID");
            validation.set_audience(&[audience.clone()]);
            let issuer = format!("https://securetoken.google.com/{}", audience);
            validation.set_issuer(&[issuer]);
            validation
        };

        let token_data = jsonwebtoken::decode::<IdTokenClaims>(token, &decoding_key, &validation)
            .map_err(|e| match e.into_kind() {
                errors::ErrorKind::ExpiredSignature => anyhow::anyhow!("expired signature"),
                _ => anyhow::anyhow!("Unknown error"),
            })
            .unwrap();

        Ok(token_data)
    }

    pub async fn verify_access_token(
        &self,
        id_token: &str,
    ) -> anyhow::Result<TokenData<AccessTokenClaims>> {
        let secret_key = std::env::var("SECRET_KEY").expect("undefined SECRET_KEY");
        let decoding_key = DecodingKey::from_secret(secret_key.as_bytes());
        let validation = {
            let mut validation = Validation::new(Algorithm::HS256);
            let audience = std::env::var("SERVICE_NAME").expect("undefined SERVICE_NAME");
            validation.set_audience(&[audience]);
            let issuer = std::env::var("API_NAME").expect("undefined API_NAME");
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

    pub fn generate_access_token(
        &self,
        token_data: &TokenData<IdTokenClaims>,
    ) -> anyhow::Result<String> {
        let claims = AccessTokenClaims::new(token_data.claims.sub());

        let secret_key = std::env::var("SECRET_KEY").expect("undefined SECRET_KEY");
        let encoding_key = EncodingKey::from_secret(secret_key.as_bytes());
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &claims,
            &encoding_key,
        )?;

        Ok(token)
    }

    pub fn generate_refresh_token(&self, sub: &str) -> anyhow::Result<String> {
        let claims = RefreshTokenClaims::new(sub.to_string());

        let secret_key = std::env::var("SECRET_KEY").expect("undefined SECRET_KEY");
        let encoding_key = EncodingKey::from_secret(secret_key.as_bytes());

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &claims,
            &encoding_key,
        )?;

        Ok(token)
    }

    fn verify_refresh_token(&self, refresh_token: &str) -> anyhow::Result<()> {
        todo!()
    }

    fn refresh_access_token(&self, token: &str) -> anyhow::Result<()> {
        todo!()
    }
}
