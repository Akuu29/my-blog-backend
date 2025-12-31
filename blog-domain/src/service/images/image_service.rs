use super::ImageServiceError;
use crate::model::images::i_image_repository::IImageRepository;
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
    ) -> Result<(), ImageServiceError> {
        let image_with_owner = self
            .repository
            .find_with_owner(image_id)
            .await
            .map_err(|e| ImageServiceError::InternalError(e.to_string()))?
            .ok_or(ImageServiceError::NotFound)?;

        if image_with_owner.article_owner_id != user_id {
            return Err(ImageServiceError::Unauthorized);
        }

        Ok(())
    }
}
