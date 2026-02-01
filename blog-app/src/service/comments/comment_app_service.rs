use super::CommentUsecaseError;
use blog_domain::{
    model::{
        comments::{
            comment::{Comment, NewComment, UpdateComment},
            i_comment_repository::{CommentFilter, ICommentRepository},
        },
        common::{item_count::ItemCount, pagination::Pagination},
    },
    service::comments::CommentService,
};
use uuid::Uuid;

pub struct CommentAppService<T: ICommentRepository> {
    repository: T,
    comment_service: CommentService<T>,
}

impl<T: ICommentRepository> CommentAppService<T>
where
    T: Clone,
{
    /// Create a new CommentAppService
    ///
    /// # Design Decision: Why instantiate CommentService inside new()?
    ///
    /// Instead of passing CommentService as a parameter, we instantiate it here for:
    ///
    /// 1. **API Simplicity**: Caller only needs to provide repository
    /// 2. **Encapsulation**: AppService manages its own domain service dependencies
    /// 3. **Consistency**: CommentService always uses the same repository as AppService
    /// 4. **Appropriate for this architecture**:
    ///    - Domain services are simple (thin wrapper around repository)
    ///    - Repository is Arc-wrapped, so clone() is cheap (O(1) reference count increment)
    ///    - Testing is done at repository level, not domain service level
    ///
    /// Trade-off: Less flexible than dependency injection, but simpler for current needs.
    /// If domain services become complex or need multiple implementations, consider
    /// adding a `with_service()` constructor for testing/customization.
    pub fn new(repository: T) -> Self {
        let comment_service = CommentService::new(repository.clone());
        Self {
            repository,
            comment_service,
        }
    }

    pub async fn create(
        &self,
        user_public_id: Option<Uuid>,
        payload: NewComment,
    ) -> Result<Comment, CommentUsecaseError> {
        self.repository
            .create(user_public_id, payload)
            .await
            .map_err(|e| CommentUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn find(&self, comment_id: Uuid) -> Result<Comment, CommentUsecaseError> {
        self.repository
            .find(comment_id, CommentFilter::default())
            .await
            .map_err(|e| CommentUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn all(
        &self,
        filter: CommentFilter,
        pagination: Pagination,
    ) -> Result<(Vec<Comment>, ItemCount), CommentUsecaseError> {
        self.repository
            .all(filter, pagination)
            .await
            .map_err(|e| CommentUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn update_with_auth(
        &self,
        comment_id: Uuid,
        user_public_id: Uuid,
        payload: UpdateComment,
    ) -> Result<Comment, CommentUsecaseError> {
        // Verify ownership using domain service
        self.comment_service
            .verify_ownership(comment_id, user_public_id)
            .await?;

        self.repository
            .update(comment_id, payload)
            .await
            .map_err(|e| CommentUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn delete_with_auth(
        &self,
        comment_id: Uuid,
        user_public_id: Uuid,
    ) -> Result<(), CommentUsecaseError> {
        // Verify ownership using domain service
        self.comment_service
            .verify_ownership(comment_id, user_public_id)
            .await?;

        self.repository
            .delete(comment_id)
            .await
            .map_err(|e| CommentUsecaseError::RepositoryError(e.to_string()))?;

        Ok(())
    }
}
