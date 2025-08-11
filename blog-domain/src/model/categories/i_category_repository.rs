use crate::model::categories::category::{Category, CategoryFilter, NewCategory, UpdateCategory};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait ICategoryRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: Uuid, payload: NewCategory) -> anyhow::Result<Category>;
    async fn all(&self, category_filter: CategoryFilter) -> anyhow::Result<Vec<Category>>;
    async fn update(&self, category_id: Uuid, payload: UpdateCategory) -> anyhow::Result<Category>;
    async fn delete(&self, category_id: Uuid) -> anyhow::Result<()>;
}
