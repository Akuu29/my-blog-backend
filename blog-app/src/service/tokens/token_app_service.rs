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
    ) -> anyhow::Result<TokenData<IdTokenClaims>> {
        let jwks = self.repository.fetch_jwks().await?;

        let token_header = jsonwebtoken::decode_header(token.str())?;

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

        let token_data =
            jsonwebtoken::decode::<IdTokenClaims>(token.str(), &decoding_key, &validation)?;

        Ok(token_data)
    }

    pub async fn verify_access_token(
        &self,
        token: AccessTokenString,
    ) -> anyhow::Result<TokenData<AccessTokenClaims>> {
        self.service.verify_access_token(token)
    }

    pub fn verify_refresh_token(
        &self,
        token: RefreshTokenString,
    ) -> anyhow::Result<TokenData<RefreshTokenClaims>> {
        self.service.verify_refresh_token(token)
    }

    pub fn generate_access_token(&self, user: &User) -> anyhow::Result<String> {
        self.service.generate_access_token(&user)
    }

    pub fn generate_refresh_token(&self, user: &User) -> anyhow::Result<String> {
        self.service.generate_refresh_token(&user)
    }
}
