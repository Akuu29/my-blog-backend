use crate::model::{
    categories::category::{Category, NewCategory, UpdateCategory},
    common::{item_count::ItemCount, pagination::Pagination},
};
use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Default)]
#[serde(rename_all = "camelCase")]
pub struct CategoryFilter {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub user_id: Option<Uuid>,
}

#[async_trait]
pub trait ICategoryRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: Uuid, payload: NewCategory) -> anyhow::Result<Category>;
    async fn find(&self, category_id: Uuid) -> anyhow::Result<Category>;
    async fn all(
        &self,
        category_filter: CategoryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Category>, ItemCount)>;
    async fn update(&self, category_id: Uuid, payload: UpdateCategory) -> anyhow::Result<Category>;
    async fn delete(&self, category_id: Uuid) -> anyhow::Result<()>;
}
