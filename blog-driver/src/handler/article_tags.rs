use crate::{
    model::{api_response::ApiResponse, auth_token::AuthToken},
    utils::{app_error::AppError, error_handler::ErrorHandler},
};
use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use blog_app::service::{
    article_tags::article_tags_app_service::ArticleTagsAppService,
    articles::article_app_service::ArticleAppService, tags::tag_app_service::TagAppService,
    tokens::token_app_service::TokenAppService,
};
use blog_domain::model::{
    article_tags::{
        article_tags::ArticleAttachedTags, i_article_tags_repository::IArticleTagsRepository,
    },
    articles::i_article_repository::{ArticleFilter, IArticleRepository},
    tags::i_tag_repository::{ITagRepository, TagFilter},
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use std::sync::Arc;

#[tracing::instrument(
    name = "attach_tags_to_article",
    skip(
        article_tags_app_service,
        token_app_service,
        article_app_service,
        tag_app_service,
        token,
    )
)]
pub async fn attach_tags_to_article<T, U, V, W>(
    Extension(article_tags_app_service): Extension<Arc<ArticleTagsAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    Extension(article_app_service): Extension<Arc<ArticleAppService<V>>>,
    Extension(tag_app_service): Extension<Arc<TagAppService<W>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Json(payload): Json<ArticleAttachedTags>,
) -> Result<impl IntoResponse, ApiResponse<String>>
where
    T: IArticleTagsRepository,
    U: ITokenRepository,
    V: IArticleRepository,
    W: ITagRepository,
{
    let token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to verify access token")
        })?;
    let user_id = token_data.claims.sub();

    // Remove all attached tags from the article in article_tags_app_service.attach_tags_to_article,
    // we need to check if the article and tags exists.
    let article_filter = ArticleFilter {
        user_id: Some(user_id),
        ..Default::default()
    };
    let article = article_app_service
        .find(payload.article_id, article_filter)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to find article")
        })?;

    let tags = tag_app_service
        .all(TagFilter {
            tag_ids: Some(payload.tag_ids),
            user_id: Some(user_id),
        })
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to find tags")
        })?;

    let payload = ArticleAttachedTags {
        article_id: article.id,
        tag_ids: tags.iter().map(|tag| tag.id).collect(),
    };

    let article_tags = article_tags_app_service
        .attach_tags_to_article(payload)
        .await
        .map_err(|e| {
            let app_err = AppError::from(e);
            app_err.handle_error("Failed to find tags")
        })?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&article_tags).unwrap()),
        None,
    ))
}
