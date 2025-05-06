use crate::model::{api_response::ApiResponse, auth_token::AuthToken};
use axum::{
    extract::{Extension, Multipart, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use blog_app::{
    query_service::article_image::i_article_image_query_service::IArticleImageQueryService,
    service::{
        images::image_app_service::ImageAppService, tokens::token_app_service::TokenAppService,
    },
};
use blog_domain::model::{
    images::{
        i_image_repository::IImageRepository,
        image::{NewImage, StorageType},
        image_filter::ImageFilter,
    },
    tokens::{i_token_repository::ITokenRepository, token_string::AccessTokenString},
};
use std::sync::Arc;
use validator::Validate;

pub async fn create<T, U>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IImageRepository,
    U: ITokenRepository,
{
    let _token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|_| ApiResponse::new(StatusCode::UNAUTHORIZED, None, None))?;

    // TODO: Move the following to FromRequest
    let mut file_data = Vec::default();
    let mut filename = None;
    let mut article_id = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        match name.as_str() {
            "file" => {
                let data = field.bytes().await.unwrap_or_default();
                file_data = data.to_vec();
            }
            "filename" => {
                filename = Some(field.text().await.unwrap_or_default());
            }
            "article_id" => {
                article_id = Some(field.text().await.unwrap_or_default());
            }
            _ => {}
        }
    }

    let kind =
        infer::get(&file_data).ok_or(ApiResponse::new(StatusCode::BAD_REQUEST, None, None))?;

    let new_image = NewImage {
        name: filename.unwrap(),
        mime_type: kind.mime_type().to_string(),
        data: file_data,
        url: None,
        storage_type: StorageType::Database,
        article_id: article_id.unwrap().parse::<i32>().unwrap(),
    };

    new_image.validate().map_err(|e| {
        let field_errors = e.field_errors();
        for field in field_errors.iter() {
            for error in field.1.iter() {
                match error.code.as_ref() {
                    "INVALID_MIME_TYPE" => {
                        return ApiResponse::new(StatusCode::UNSUPPORTED_MEDIA_TYPE, None, None)
                    }
                    "INVALID_DATA_LENGTH" => {
                        return ApiResponse::new(StatusCode::PAYLOAD_TOO_LARGE, None, None)
                    }
                    "INVALID_IMAGE_DIMENSION" => {
                        return ApiResponse::new(StatusCode::UNPROCESSABLE_ENTITY, None, None)
                    }
                    _ => return ApiResponse::new(StatusCode::BAD_REQUEST, None, None),
                }
            }
        }
        ApiResponse::new(StatusCode::BAD_REQUEST, None, None)
    })?;

    let image = image_app_service
        .create(new_image)
        .await
        .map_err(|_| ApiResponse::new(StatusCode::BAD_REQUEST, None, None))?;

    Ok(ApiResponse::new(
        StatusCode::CREATED,
        Some(serde_json::to_string(&image).unwrap()),
        None,
    ))
}

pub async fn all<T>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Query(filter): Query<ImageFilter>,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IImageRepository,
{
    let images = image_app_service
        .all(filter)
        .await
        .map_err(|_| ApiResponse::new(StatusCode::BAD_REQUEST, None, None))?;

    Ok(ApiResponse::new(
        StatusCode::OK,
        Some(serde_json::to_string(&images).unwrap()),
        None,
    ))
}

pub async fn find_data<T>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Path(image_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IImageRepository,
{
    let image_data = image_app_service
        .find_data(image_id)
        .await
        .map_err(|_| ApiResponse::new(StatusCode::BAD_REQUEST, None, None))?;

    let response = ApiResponse::new(StatusCode::OK, Some(image_data.data.clone()), None)
        .with_header("Content-Type", &image_data.mime_type)
        .with_header("Content-Disposition", "attachment")
        .with_header("Content-Length", &image_data.data.len().to_string());

    Ok(response)
}

pub async fn delete<T, U, E>(
    Extension(image_app_service): Extension<Arc<ImageAppService<T>>>,
    Extension(token_app_service): Extension<Arc<TokenAppService<U>>>,
    Extension(article_image_query_service): Extension<Arc<E>>,
    AuthToken(token): AuthToken<AccessTokenString>,
    Path(image_id): Path<i32>,
) -> Result<impl IntoResponse, ApiResponse<()>>
where
    T: IImageRepository,
    U: ITokenRepository,
    E: IArticleImageQueryService,
{
    let token_data = token_app_service
        .verify_access_token(token)
        .await
        .map_err(|_| ApiResponse::new(StatusCode::UNAUTHORIZED, None, None))?;

    let is_image_owned_by_user = article_image_query_service
        .is_image_owned_by_user(image_id, token_data.claims.sub())
        .await
        .map_err(|_| ApiResponse::new(StatusCode::BAD_REQUEST, None, None))?;

    if !is_image_owned_by_user {
        return Err(ApiResponse::new(StatusCode::FORBIDDEN, None, None));
    }

    image_app_service
        .delete(image_id)
        .await
        .map_err(|_| ApiResponse::new(StatusCode::BAD_REQUEST, None, None))?;

    Ok(ApiResponse::<()>::new(StatusCode::NO_CONTENT, None, None))
}
