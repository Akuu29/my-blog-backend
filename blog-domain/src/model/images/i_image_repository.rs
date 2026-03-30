use crate::model::{
    error::RepositoryError,
    images::image::{ImageData, ImageDataProps, ImageWithOwner, NewImage},
};
use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageFilter {
    pub article_id: Option<Uuid>,
}

#[async_trait]
pub trait IImageRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewImage) -> Result<ImageDataProps, RepositoryError>;
    async fn all(&self, filter: ImageFilter) -> Result<Vec<ImageDataProps>, RepositoryError>;
    async fn find_data(&self, image_id: Uuid) -> Result<ImageData, RepositoryError>;
    async fn find_with_owner(
        &self,
        image_id: Uuid,
    ) -> Result<Option<ImageWithOwner>, RepositoryError>;
    async fn delete(&self, image_id: Uuid) -> Result<(), RepositoryError>;
}
