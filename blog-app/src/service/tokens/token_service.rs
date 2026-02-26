use super::error::TokenServiceError;
use crate::config::TokenConfig;
use blog_domain::model::{
    tokens::{
        token::{AccessTokenClaims, RefreshTokenClaims},
        token_string::{AccessTokenString, RefreshTokenString, TokenString},
    },
    users::user::User,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, TokenData, Validation};

pub struct TokenService {
    config: TokenConfig,
}

impl TokenService {
    pub fn new(config: TokenConfig) -> Self {
        Self { config }
    }

    pub fn generate_access_token(&self, user: &User) -> Result<String, TokenServiceError> {
        let claims = AccessTokenClaims::new(user, &self.config.audience, &self.config.issuer);
        let encoding_key = EncodingKey::from_secret(self.config.access_token_secret_key.as_bytes());
        let token_string = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &claims,
            &encoding_key,
        )?;

        Ok(token_string)
    }

    pub fn generate_refresh_token(&self, user: &User) -> Result<String, TokenServiceError> {
        let claims = RefreshTokenClaims::new(user, &self.config.audience, &self.config.issuer);
        let encoding_key =
            EncodingKey::from_secret(self.config.refresh_token_secret_key.as_bytes());

        let token_string = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &claims,
            &encoding_key,
        )?;

        Ok(token_string)
    }

    pub fn verify_access_token(
        &self,
        token: AccessTokenString,
    ) -> Result<TokenData<AccessTokenClaims>, TokenServiceError> {
        let decoding_key = DecodingKey::from_secret(self.config.access_token_secret_key.as_bytes());
        let validation = {
            let mut validation = Validation::new(Algorithm::HS256);
            validation.set_audience(&[self.config.audience.clone()]);
            validation.set_issuer(&[self.config.issuer.clone()]);
            validation
        };

        let token_data =
            jsonwebtoken::decode::<AccessTokenClaims>(token.str(), &decoding_key, &validation)?;

        Ok(token_data)
    }

    pub fn verify_refresh_token(
        &self,
        token: RefreshTokenString,
    ) -> Result<TokenData<RefreshTokenClaims>, TokenServiceError> {
        let decoding_key =
            DecodingKey::from_secret(self.config.refresh_token_secret_key.as_bytes());
        let validation = {
            let mut validation = Validation::new(Algorithm::HS256);
            validation.set_audience(&[self.config.audience.clone()]);
            validation.set_issuer(&[self.config.issuer.clone()]);
            validation
        };

        let token_data =
            jsonwebtoken::decode::<RefreshTokenClaims>(token.str(), &decoding_key, &validation)?;

        Ok(token_data)
    }
}
