use crate::model::tags::tag::{NewTag, Tag};
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::types::Uuid;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagFilter {
    pub user_id: Option<Uuid>,
    pub tag_ids: Option<Vec<i32>>,
}

#[async_trait]
pub trait ITagRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: Uuid, payload: NewTag) -> anyhow::Result<Tag>;
    async fn all(&self, tag_filter: TagFilter) -> anyhow::Result<Vec<Tag>>;
    async fn delete(&self, tag_id: i32) -> anyhow::Result<()>;
}
