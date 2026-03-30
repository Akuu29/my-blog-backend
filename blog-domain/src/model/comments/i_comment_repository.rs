use crate::model::{
    comments::comment::{Comment, NewComment, UpdateComment},
    common::{item_count::ItemCount, pagination::Pagination},
    error::RepositoryError,
};
use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Default, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CommentFilter {
    pub article_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
}

#[async_trait]
pub trait ICommentRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(
        &self,
        user_id: Option<Uuid>,
        user_name: String,
        payload: NewComment,
    ) -> Result<Comment, RepositoryError>;
    async fn find(
        &self,
        comment_id: Uuid,
        comment_filter: CommentFilter,
    ) -> Result<Comment, RepositoryError>;
    async fn all(
        &self,
        comment_filter: CommentFilter,
        pagination: Pagination,
    ) -> Result<(Vec<Comment>, ItemCount), RepositoryError>;
    async fn update(
        &self,
        comment_id: Uuid,
        payload: UpdateComment,
    ) -> Result<Comment, RepositoryError>;
    async fn delete(&self, comment_id: Uuid) -> Result<(), RepositoryError>;
}
