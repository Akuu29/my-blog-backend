use crate::model::images::image::{ImageData, ImageDataProps, ImageWithOwner, NewImage};
use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageFilter {
    #[serde(rename = "articleId")]
    pub article_public_id: Option<Uuid>,
}

#[async_trait]
pub trait IImageRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewImage) -> anyhow::Result<ImageDataProps>;
    async fn all(&self, filter: ImageFilter) -> anyhow::Result<Vec<ImageDataProps>>;
    async fn find_data(&self, image_id: Uuid) -> anyhow::Result<ImageData>;
    async fn find_with_owner(&self, image_id: Uuid) -> anyhow::Result<Option<ImageWithOwner>>;
    async fn delete(&self, image_id: Uuid) -> anyhow::Result<()>;
}
