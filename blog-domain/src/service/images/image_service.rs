use crate::model::images::i_image_repository::IImageRepository;
use crate::service::error::DomainServiceError;
use uuid::Uuid;

pub struct ImageService<T>
where
    T: IImageRepository,
{
    repository: T,
}

impl<T> ImageService<T>
where
    T: IImageRepository,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    /// Business Rule: A user can only access images that belong to their own articles
    pub async fn verify_ownership(
        &self,
        image_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), DomainServiceError> {
        let image_with_owner = self
            .repository
            .find_with_owner(image_id)
            .await?
            .ok_or(DomainServiceError::NotFound)?;

        if image_with_owner.article_owner_id != user_id {
            return Err(DomainServiceError::Unauthorized);
        }

        Ok(())
    }
}
