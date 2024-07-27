use crate::{
    model::token::{AccessTokenClaims, ApiCredentials},
    repository::token::TokenRepository,
    service::token::TokenService,
};
use jsonwebtoken::TokenData;

pub struct TokenUseCase<T: TokenRepository> {
    service: TokenService<T>,
}

impl<T: TokenRepository> TokenUseCase<T> {
    pub fn new(repository: T) -> Self {
        Self {
            service: TokenService::new(repository),
        }
    }

    pub async fn verify_id_token(&self, id_token: &str) -> anyhow::Result<ApiCredentials> {
        let token_data = self.service.verify_id_token(id_token).await?;

        let access_token = self.service.generate_access_token(&token_data).unwrap();
        let refresh_token = self
            .service
            .generate_refresh_token(&token_data.claims.sub())
            .unwrap();

        Ok(ApiCredentials::new(access_token, refresh_token))
    }

    pub async fn verify_access_token(
        &self,
        access_token: &str,
    ) -> anyhow::Result<TokenData<AccessTokenClaims>> {
        let token_data = self.service.verify_access_token(access_token).await?;

        Ok(token_data)
    }
}
