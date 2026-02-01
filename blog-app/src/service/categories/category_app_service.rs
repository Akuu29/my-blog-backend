use super::CategoryUsecaseError;
use blog_domain::model::categories::i_category_repository::CategoryFilter;
use blog_domain::{
    model::{
        categories::{
            category::{Category, NewCategory, UpdateCategory},
            i_category_repository::ICategoryRepository,
        },
        common::{item_count::ItemCount, pagination::Pagination},
    },
    service::categories::CategoryService,
};
use uuid::Uuid;

pub struct CategoryAppService<T: ICategoryRepository> {
    repository: T,
    category_service: CategoryService<T>,
}

impl<T: ICategoryRepository> CategoryAppService<T> {
    pub fn new(repository: T) -> Self {
        let category_service = CategoryService::new(repository.clone());
        Self {
            repository,
            category_service,
        }
    }

    pub async fn create(&self, user_id: Uuid, payload: NewCategory) -> anyhow::Result<Category> {
        self.repository.create(user_id, payload).await
    }

    pub async fn all(
        &self,
        category_filter: CategoryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Category>, ItemCount)> {
        self.repository.all(category_filter, pagination).await
    }

    pub async fn update_with_auth(
        &self,
        user_id: Uuid,
        category_id: Uuid,
        payload: UpdateCategory,
    ) -> Result<Category, CategoryUsecaseError> {
        // Verify category ownership
        self.category_service
            .verify_ownership(category_id, user_id)
            .await?;

        self.repository
            .update(category_id, payload)
            .await
            .map_err(|e| CategoryUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn delete_with_auth(
        &self,
        user_id: Uuid,
        category_id: Uuid,
    ) -> Result<(), CategoryUsecaseError> {
        // Verify category ownership
        self.category_service
            .verify_ownership(category_id, user_id)
            .await?;

        self.repository
            .delete(category_id)
            .await
            .map_err(|e| CategoryUsecaseError::RepositoryError(e.to_string()))
    }
}
