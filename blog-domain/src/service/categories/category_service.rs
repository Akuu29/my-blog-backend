use crate::model::categories::i_category_repository::ICategoryRepository;
use crate::service::error::DomainServiceError;
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
        user_id: Uuid,
    ) -> Result<(), DomainServiceError> {
        let category = self.repository.find(category_id).await?;

        if category.user_id != user_id {
            return Err(DomainServiceError::Unauthorized);
        }

        Ok(())
    }
}
