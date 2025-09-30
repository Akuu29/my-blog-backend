use crate::model::{
    categories::category::{Category, NewCategory, UpdateCategory},
    common::{item_count::ItemCount, pagination::Pagination},
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use uuid::Uuid;
use validator::Validate;

#[serde_as]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CategoryFilter {
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "id")]
    pub public_id: Option<Uuid>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub name: Option<String>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "userId")]
    pub user_public_id: Option<Uuid>,
}

#[async_trait]
pub trait ICategoryRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: Uuid, payload: NewCategory) -> anyhow::Result<Category>;
    async fn all(
        &self,
        category_filter: CategoryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Category>, ItemCount)>;
    async fn update(&self, category_id: Uuid, payload: UpdateCategory) -> anyhow::Result<Category>;
    async fn delete(&self, category_id: Uuid) -> anyhow::Result<()>;
}
