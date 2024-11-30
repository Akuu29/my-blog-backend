use crate::handler::ValidatedJson;
use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
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
        category::{NewCategory, UpdateCategory},
        i_category_repository::ICategoryRepository,
    },
    tokens::i_token_repository::ITokenRepository,
};
use std::sync::Arc;

pub async fn create_category<T: ICategoryRepository, U: ITokenRepository>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    ValidatedJson(payload): ValidatedJson<NewCategory>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let access_token_data = token_app_service
        .verify_access_token(&access_token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let category = category_app_service
        .create(access_token_data.claims.sub, payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::CREATED, Json(category)))
}

pub async fn all_categories<T: ICategoryRepository>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
) -> Result<impl IntoResponse, StatusCode> {
    let categories = category_app_service
        .all()
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(categories)))
}

pub async fn update_category<T: ICategoryRepository, U: ITokenRepository>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(category_id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateCategory>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let _access_token_data = token_app_service
        .verify_access_token(&access_token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let category = category_app_service
        .update(category_id, payload)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(category)))
}

pub async fn delete_category<T: ICategoryRepository, U: ITokenRepository>(
    Extension(category_app_service): Extension<Arc<CategoryAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(category_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let access_token = bearer.token().to_string();
    let _access_token_data = token_app_service
        .verify_access_token(&access_token)
        .await
        .map_err(|e| {
            tracing::info!("failed to verify access token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    category_app_service
        .delete(category_id)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn find_articles_by_category<T: IArticlesByCategoryQueryService>(
    Extension(articles_by_category_query_service): Extension<Arc<T>>,
    Path(category_name): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let articles_by_category = articles_by_category_query_service
        .find_article_title_by_category(category_name)
        .await
        .or(Err(StatusCode::BAD_REQUEST))?;

    Ok((StatusCode::OK, Json(articles_by_category)))
}
