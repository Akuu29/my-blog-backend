use crate::service::tokens::token_service::TokenService;
use blog_domain::model::{
    tokens::{
        i_token_repository::ITokenRepository,
        token::{AccessTokenClaims, IdTokenClaims, RefreshTokenClaims},
    },
    users::user::User,
};
use jsonwebtoken::{errors, Algorithm, DecodingKey, TokenData, Validation};

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
        self.service.verify_access_token(id_token)
    }

    pub fn verify_refresh_token(
        &self,
        refresh_token: &str,
    ) -> anyhow::Result<TokenData<RefreshTokenClaims>> {
        self.service.verify_refresh_token(refresh_token)
    }

    pub fn generate_access_token(&self, user: &User) -> anyhow::Result<String> {
        self.service.generate_access_token(&user)
    }

    pub fn generate_refresh_token(&self, user: &User) -> anyhow::Result<String> {
        self.service.generate_refresh_token(&user)
    }
}
