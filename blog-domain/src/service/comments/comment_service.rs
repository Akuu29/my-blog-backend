use super::CommentServiceError;
use crate::model::comments::i_comment_repository::{CommentFilter, ICommentRepository};
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
        user_public_id: Uuid,
    ) -> Result<(), CommentServiceError> {
        let comment = self
            .repository
            .find(comment_id, CommentFilter::default())
            .await
            .map_err(|e| CommentServiceError::InternalError(e.to_string()))?;

        if comment.user_public_id != Some(user_public_id) {
            return Err(CommentServiceError::Unauthorized);
        }

        Ok(())
    }
}
