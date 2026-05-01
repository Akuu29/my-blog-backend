use crate::service::error::UsecaseError;
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

    pub async fn create(
        &self,
        user_id: Uuid,
        payload: NewCategory,
    ) -> Result<Category, UsecaseError> {
        Ok(self.repository.create(user_id, payload).await?)
    }

    pub async fn all(
        &self,
        category_filter: CategoryFilter,
        pagination: Pagination,
    ) -> Result<(Vec<Category>, ItemCount), UsecaseError> {
        Ok(self.repository.all(category_filter, pagination).await?)
    }

    pub async fn update_with_auth(
        &self,
        user_id: Uuid,
        category_id: Uuid,
        payload: UpdateCategory,
    ) -> Result<Category, UsecaseError> {
        // Verify category ownership
        self.category_service
            .verify_ownership(category_id, user_id)
            .await?;

        Ok(self.repository.update(category_id, payload).await?)
    }

    pub async fn delete_with_auth(
        &self,
        user_id: Uuid,
        category_id: Uuid,
    ) -> Result<(), UsecaseError> {
        // Verify category ownership
        self.category_service
            .verify_ownership(category_id, user_id)
            .await?;

        self.repository.delete(category_id).await?;

        Ok(())
    }
}
