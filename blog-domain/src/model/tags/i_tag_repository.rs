use crate::model::tags::{
    tag::{NewTag, Tag},
    tag_filter::TagFilter,
};
use async_trait::async_trait;
use sqlx::types::Uuid;

#[async_trait]
pub trait ITagRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: Uuid, payload: NewTag) -> anyhow::Result<Tag>;
    async fn all(&self, tag_filter: TagFilter) -> anyhow::Result<Vec<Tag>>;
    async fn delete(&self, tag_id: i32) -> anyhow::Result<()>;
}
