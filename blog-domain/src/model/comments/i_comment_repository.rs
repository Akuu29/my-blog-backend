use crate::model::comments::comment::{Comment, NewComment, UpdateComment};
use async_trait::async_trait;

#[async_trait]
pub trait ICommentRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewComment) -> anyhow::Result<Comment>;
    async fn find(&self, id: i32) -> anyhow::Result<Comment>;
    // TODO Bad approach because it's not scalable
    async fn find_by_article_id(&self, article_id: i32) -> anyhow::Result<Vec<Comment>>;
    async fn update(&self, id: i32, payload: UpdateComment) -> anyhow::Result<Comment>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}
