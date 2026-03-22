use crate::model::tags::i_tag_repository::ITagRepository;
use crate::service::error::DomainServiceError;
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
        user_id: Uuid,
    ) -> Result<(), DomainServiceError> {
        let tag = self.repository.find(tag_id).await?;

        if tag.user_id != user_id {
            return Err(DomainServiceError::Unauthorized);
        }

        Ok(())
    }
}
