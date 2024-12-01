use blog_domain::model::categories::{
    category::{Category, CategoryFilter, NewCategory, UpdateCategory},
    i_category_repository::ICategoryRepository,
};
use sqlx::types::Uuid;

pub struct CategoryAppService<T: ICategoryRepository> {
    repository: T,
}

impl<T: ICategoryRepository> CategoryAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, user_id: Uuid, payload: NewCategory) -> anyhow::Result<Category> {
        self.repository.create(user_id, payload).await
    }

    pub async fn all(&self, category_filter: CategoryFilter) -> anyhow::Result<Vec<Category>> {
        self.repository.all(category_filter).await
    }

    pub async fn update(
        &self,
        category_id: i32,
        payload: UpdateCategory,
    ) -> anyhow::Result<Category> {
        self.repository.update(category_id, payload).await
    }

    pub async fn delete(&self, category_id: i32) -> anyhow::Result<()> {
        self.repository.delete(category_id).await
    }
}
