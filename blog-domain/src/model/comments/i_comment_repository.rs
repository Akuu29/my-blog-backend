use crate::model::{
    comments::comment::{Comment, NewComment, UpdateComment},
    common::{item_count::ItemCount, pagination::Pagination},
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use uuid::Uuid;
use validator::Validate;

#[serde_as]
#[derive(Debug, Default, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CommentFilter {
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "articleId")]
    pub article_public_id: Option<Uuid>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "userId")]
    pub user_public_id: Option<Uuid>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub user_name: Option<String>,
}

#[async_trait]
pub trait ICommentRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(
        &self,
        user_public_id: Option<Uuid>,
        payload: NewComment,
    ) -> anyhow::Result<Comment>;
    async fn find(
        &self,
        comment_id: Uuid,
        comment_filter: CommentFilter,
    ) -> anyhow::Result<Comment>;
    async fn all(
        &self,
        comment_filter: CommentFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Comment>, ItemCount)>;
    async fn update(&self, comment_id: Uuid, payload: UpdateComment) -> anyhow::Result<Comment>;
    async fn delete(&self, comment_id: Uuid) -> anyhow::Result<()>;
}
