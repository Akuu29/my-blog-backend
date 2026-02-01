use super::TagServiceError;
use crate::model::tags::i_tag_repository::ITagRepository;
use uuid::Uuid;

pub struct TagService<T>
where
    T: ITagRepository,
{
    repository: T,
}

impl<T> TagService<T>
where
    T: ITagRepository,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    /// Business Rule: A user can only modify or delete their own tags
    pub async fn verify_ownership(
        &self,
        tag_id: Uuid,
        user_public_id: Uuid,
    ) -> Result<(), TagServiceError> {
        let tag = self.repository.find(tag_id).await.map_err(|_| {
            // TODO Propagation of repository errors.
            TagServiceError::NotFound
        })?;

        if tag.user_public_id != user_public_id {
            return Err(TagServiceError::Unauthorized);
        }

        Ok(())
    }
}
