use crate::model::images::image::{ImageData, ImageDataProps, NewImage};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageFilter {
    pub article_id: Option<i32>,
}

#[async_trait]
pub trait IImageRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewImage) -> anyhow::Result<ImageDataProps>;
    async fn all(&self, filter: ImageFilter) -> anyhow::Result<Vec<ImageDataProps>>;
    async fn find_data(&self, image_id: i32) -> anyhow::Result<ImageData>;
    async fn delete(&self, image_id: i32) -> anyhow::Result<()>;
}
