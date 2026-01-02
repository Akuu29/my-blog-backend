use super::error::TokenServiceError;
use crate::service::tokens::token_service::TokenService;
use blog_domain::model::{
    tokens::{
        i_token_repository::ITokenRepository,
        token::{AccessTokenClaims, IdTokenClaims, RefreshTokenClaims},
        token_string::{AccessTokenString, IdTokenString, RefreshTokenString, TokenString},
    },
    users::user::User,
};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};

pub struct TokenAppService<T: ITokenRepository> {
    repository: T,
    service: TokenService,
}

impl<T: ITokenRepository> TokenAppService<T> {
    pub fn new(repository: T) -> Self {
        Self {
            repository,
            service: TokenService::default(),
        }
    }

    pub async fn verify_id_token(
        &self,
        token: IdTokenString,
    ) -> Result<TokenData<IdTokenClaims>, TokenServiceError> {
        let jwks = self.repository.fetch_jwks().await
            .map_err(|e| TokenServiceError::RepositoryError(e.to_string()))?;

        let token_header = jsonwebtoken::decode_header(token.str())?;

        let kid = token_header.kid
            .ok_or_else(|| TokenServiceError::InvalidToken)?;
        let pem = jwks.get(&kid)
            .ok_or_else(|| TokenServiceError::InvalidToken)?;
        let decoding_key = DecodingKey::from_rsa_pem(pem.as_bytes())?;

        let validation = {
            let mut validation = Validation::new(Algorithm::RS256);
            let audience = std::env::var("FIREBASE_PROJECT_ID")
                .map_err(|e| TokenServiceError::InternalError(format!("Missing FIREBASE_PROJECT_ID: {}", e)))?;
            validation.set_audience(&[audience.clone()]);
            let issuer = format!("https://securetoken.google.com/{}", audience);
            validation.set_issuer(&[issuer]);
            validation
        };

        // JWT errors are automatically converted via From<jsonwebtoken::errors::Error>
        let token_data =
            jsonwebtoken::decode::<IdTokenClaims>(token.str(), &decoding_key, &validation)?;

        Ok(token_data)
    }

    pub async fn verify_access_token(
        &self,
        token: AccessTokenString,
    ) -> Result<TokenData<AccessTokenClaims>, TokenServiceError> {
        self.service.verify_access_token(token)
    }

    pub fn verify_refresh_token(
        &self,
        token: RefreshTokenString,
    ) -> Result<TokenData<RefreshTokenClaims>, TokenServiceError> {
        self.service.verify_refresh_token(token)
    }

    pub fn generate_access_token(&self, user: &User) -> Result<String, TokenServiceError> {
        self.service.generate_access_token(&user)
    }

    pub fn generate_refresh_token(&self, user: &User) -> Result<String, TokenServiceError> {
        self.service.generate_refresh_token(&user)
    }
}
