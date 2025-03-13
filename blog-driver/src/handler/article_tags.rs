use crate::model::{api_response::ApiResponse, auth_token::AuthToken};
use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use blog_app::service::{
    article_tags::article_tags_app_service::ArticleTagsAppService,
    tokens::token_app_service::TokenAppService,
};
use blog_domain::model::{
    article_tags::{
        article_tags::ArticleAttachedTags, i_article_tags_repository::IArticleTagsRepository,
    },
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use std::sync::Arc;

pub async fn attach_tags_to_article<T: IArticleTagsRepository, U: ITokenRepository>(
    Extension(article_tags_app_service): Extension<Arc<ArticleTagsAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Json(payload): Json<ArticleAttachedTags>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    token_app_service
        .verify_access_token(token)
        .await
        .map_err(|_| ApiResponse::new(StatusCode::UNAUTHORIZED, None, None))?;

    let article_tags = article_tags_app_service
        .attach_tags_to_article(payload)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(article_tags),
        None,
    ))
}
