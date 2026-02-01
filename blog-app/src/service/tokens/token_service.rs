use super::error::TokenServiceError;
use blog_domain::model::{
    tokens::{
        token::{AccessTokenClaims, RefreshTokenClaims},
        token_string::{AccessTokenString, RefreshTokenString, TokenString},
    },
    users::user::User,
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, TokenData, Validation};

#[derive(Default)]
pub struct TokenService {}

impl TokenService {
    pub fn generate_access_token(&self, user: &User) -> Result<String, TokenServiceError> {
        let claims = AccessTokenClaims::new(user);

        let secret_key = std::env::var("ACCESS_TOKEN_SECRET_KEY")
            .map_err(|e| TokenServiceError::InternalError(format!("Missing ACCESS_TOKEN_SECRET_KEY: {}", e)))?;
        let encoding_key = EncodingKey::from_secret(secret_key.as_bytes());
        let token_string = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::HS256),
            &claims,
            &encoding_key,
        )?;

        Ok(token_string)
    }

    pub fn generate_refresh_token(&self, user: &User) -> Result<String, TokenServiceError> {
        let claims = RefreshTokenClaims::new(user);

        let secret_key = std::env::var("REFRESH_TOKEN_SECRET_KEY")
            .map_err(|e| TokenServiceError::InternalError(format!("Missing REFRESH_TOKEN_SECRET_KEY: {}", e)))?;
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
        token: AccessTokenString,
    ) -> Result<TokenData<AccessTokenClaims>, TokenServiceError> {
        let secret_key = std::env::var("ACCESS_TOKEN_SECRET_KEY")
            .map_err(|e| TokenServiceError::InternalError(format!("Missing ACCESS_TOKEN_SECRET_KEY: {}", e)))?;
        let decoding_key = DecodingKey::from_secret(secret_key.as_bytes());
        let validation = {
            let mut validation = Validation::new(Algorithm::HS256);
            let audience = std::env::var("AUDIENCE")
                .map_err(|e| TokenServiceError::InternalError(format!("Missing AUDIENCE: {}", e)))?;
            validation.set_audience(&[audience]);
            let issuer = std::env::var("ISSUER")
                .map_err(|e| TokenServiceError::InternalError(format!("Missing ISSUER: {}", e)))?;
            validation.set_issuer(&[issuer]);
            validation
        };

        // JWT errors are automatically converted via From<jsonwebtoken::errors::Error>
        // ExpiredSignature -> TokenExpired -> ErrorCategory::Authentication -> 401
        let token_data =
            jsonwebtoken::decode::<AccessTokenClaims>(token.str(), &decoding_key, &validation)?;

        Ok(token_data)
    }

    pub fn verify_refresh_token(
        &self,
        token: RefreshTokenString,
    ) -> Result<TokenData<RefreshTokenClaims>, TokenServiceError> {
        let secret_key = std::env::var("REFRESH_TOKEN_SECRET_KEY")
            .map_err(|e| TokenServiceError::InternalError(format!("Missing REFRESH_TOKEN_SECRET_KEY: {}", e)))?;
        let decoding_key = DecodingKey::from_secret(secret_key.as_bytes());
        let validation = {
            let mut validation = Validation::new(Algorithm::HS256);
            let audience = std::env::var("AUDIENCE")
                .map_err(|e| TokenServiceError::InternalError(format!("Missing AUDIENCE: {}", e)))?;
            validation.set_audience(&[audience]);
            let issuer = std::env::var("ISSUER")
                .map_err(|e| TokenServiceError::InternalError(format!("Missing ISSUER: {}", e)))?;
            validation.set_issuer(&[issuer]);
            validation
        };

        // JWT errors are automatically converted via From<jsonwebtoken::errors::Error>
        let token_data =
            jsonwebtoken::decode::<RefreshTokenClaims>(token.str(), &decoding_key, &validation)?;

        Ok(token_data)
    }
}
