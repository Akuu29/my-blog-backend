use crate::model::{
    api_response::ApiResponse, auth_token::AuthToken, validated_json::ValidatedJson,
};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::{
    query_service::articles_by_category::i_articles_by_category_query_service::IArticlesByCategoryQueryService,
    service::{
        categories::category_app_service::CategoryAppService,
        tokens::token_app_service::TokenAppService,
    },
};
use blog_domain::model::{
    categories::{
        category::{CategoryFilter, NewCategory, UpdateCategory},
        i_category_repository::ICategoryRepository,
    },
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use std::sync::Arc;

pub async fn create_category<T: ICategoryRepository, U: ITokenRepository>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    ValidatedJson(payload): ValidatedJson<NewCategory>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            ApiResponse::new(StatusCode::UNAUTHORIZED, None, None)
        })?;

    let category = category_app_service
        .create(access_token_data.claims.sub(), payload)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&category).unwrap()),
        None,
    ))
}

pub async fn all_categories<T: ICategoryRepository>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Query(category_filter): Query<CategoryFilter>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let categories = category_app_service
        .all(category_filter)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&categories).unwrap()),
        None,
    ))
}

pub async fn update_category<T: ICategoryRepository, U: ITokenRepository>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(category_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateCategory>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            ApiResponse::new(StatusCode::UNAUTHORIZED, None, None)
        })?;

    let category = category_app_service
        .update(category_id, payload)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&category).unwrap()),
        None,
    ))
}

pub async fn delete_category<T: ICategoryRepository, U: ITokenRepository>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(category_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let _access_token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            ApiResponse::new(StatusCode::UNAUTHORIZED, None, None)
        })?;

    category_app_service
        .delete(category_id)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}

pub async fn find_articles_by_category<T: IArticlesByCategoryQueryService>(
    Extension(articles_by_category_query_service): Extension<Arc<T>>,
    Path(category_name): Path<String>,
) -> Result<impl IntoResponse, ApiResponse<()>> {
    let articles_by_category = articles_by_category_query_service
        .find_article_title_by_category(category_name)
        .await
        .or(Err(ApiResponse::new(StatusCode::BAD_REQUEST, None, None)))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&articles_by_category).unwrap()),
        None,
    ))
}
