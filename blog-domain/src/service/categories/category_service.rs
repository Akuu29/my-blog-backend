use super::CategoryServiceError;
use crate::model::categories::i_category_repository::ICategoryRepository;
use uuid::Uuid;

pub struct CategoryService<T>
where
    T: ICategoryRepository,
{
    repository: T,
}

impl<T> CategoryService<T>
where
    T: ICategoryRepository,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    /// Business Rule: A user can only modify or delete their own categories
    pub async fn verify_ownership(
        &self,
        category_id: Uuid,
        user_public_id: Uuid,
    ) -> Result<(), CategoryServiceError> {
        let category = self.repository.find(category_id).await.map_err(|_| {
            // TODO Propagation of repository errors.
            CategoryServiceError::NotFound
        })?;

        if category.user_public_id != user_public_id {
            return Err(CategoryServiceError::Unauthorized);
        }

        Ok(())
    }
}
