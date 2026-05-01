use crate::model::{
    common::{item_count::ItemCount, pagination::Pagination},
    error::RepositoryError,
    tags::tag::{NewTag, Tag},
};
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::types::Uuid;
use validator::Validate;

#[derive(Debug, Default, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TagFilter {
    pub user_id: Option<Uuid>,
    pub tag_ids: Option<Vec<Uuid>>,
}

impl TagFilter {
    pub fn new(user_id: Option<Uuid>, tag_ids: Option<Vec<Uuid>>) -> Self {
        Self { user_id, tag_ids }
    }
}

#[async_trait]
pub trait ITagRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: Uuid, payload: NewTag) -> Result<Tag, RepositoryError>;
    async fn find(&self, tag_id: Uuid) -> Result<Tag, RepositoryError>;
    async fn all(
        &self,
        tag_filter: TagFilter,
        pagination: Pagination,
    ) -> Result<(Vec<Tag>, ItemCount), RepositoryError>;
    async fn delete(&self, tag_id: Uuid) -> Result<(), RepositoryError>;
}
