use crate::model::images::image::{Image, NewImage};
use async_trait::async_trait;

#[async_trait]
pub trait IImageRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewImage) -> anyhow::Result<Image>;
    async fn find(&self, image_id: i32) -> anyhow::Result<Image>;
    async fn delete(&self, image_id: i32) -> anyhow::Result<()>;
}
