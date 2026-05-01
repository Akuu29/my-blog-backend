use crate::model::comments::i_comment_repository::{CommentFilter, ICommentRepository};
use crate::service::error::DomainServiceError;
use uuid::Uuid;

pub struct CommentService<T>
where
    T: ICommentRepository,
{
    repository: T,
}

impl<T> CommentService<T>
where
    T: ICommentRepository,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    /// Business Rule: A user can only modify or delete their own comments
    pub async fn verify_ownership(
        &self,
        comment_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), DomainServiceError> {
        let comment = self
            .repository
            .find(comment_id, CommentFilter::default())
            .await?;

        if comment.user_id != Some(user_id) {
            return Err(DomainServiceError::Unauthorized);
        }

        Ok(())
    }
}
