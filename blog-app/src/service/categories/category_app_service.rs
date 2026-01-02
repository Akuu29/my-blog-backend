use super::CategoryUsecaseError;
use blog_domain::model::{
    categories::{
        category::{Category, NewCategory, UpdateCategory},
        i_category_repository::{CategoryFilter, ICategoryRepository},
    },
    common::{item_count::ItemCount, pagination::Pagination},
};
use uuid::Uuid;

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

    pub async fn all(
        &self,
        category_filter: CategoryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Category>, ItemCount)> {
        self.repository.all(category_filter, pagination).await
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        category_id: Uuid,
        payload: UpdateCategory,
    ) -> Result<Category, CategoryUsecaseError> {
        // check ownership
        let (categories, _) = self
            .repository
            .all(
                CategoryFilter {
                    public_id: Some(category_id),
                    ..Default::default()
                },
                Pagination::default(),
            )
            .await
            .map_err(|e| CategoryUsecaseError::RepositoryError(e.to_string()))?;

        let category = categories
            .first()
            .ok_or(CategoryUsecaseError::ValidationFailed(
                "Category not found".to_string(),
            ))?;

        if category.user_public_id != user_id {
            return Err(CategoryUsecaseError::PermissionDenied(
                "You are not the owner of this category".to_string()
            ));
        }

        self.repository
            .update(category_id, payload)
            .await
            .map_err(|e| CategoryUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn delete(&self, user_id: Uuid, category_id: Uuid) -> Result<(), CategoryUsecaseError> {
        // check ownership
        let (categories, _) = self
            .repository
            .all(
                CategoryFilter {
                    public_id: Some(category_id),
                    ..Default::default()
                },
                Pagination::default(),
            )
            .await
            .map_err(|e| CategoryUsecaseError::RepositoryError(e.to_string()))?;

        let category = categories
            .first()
            .ok_or(CategoryUsecaseError::ValidationFailed(
                "Category not found".to_string(),
            ))?;

        if category.user_public_id != user_id {
            return Err(CategoryUsecaseError::PermissionDenied(
                "You are not the owner of this category".to_string()
            ));
        }

        self.repository
            .delete(category_id)
            .await
            .map_err(|e| CategoryUsecaseError::RepositoryError(e.to_string()))
    }
}
