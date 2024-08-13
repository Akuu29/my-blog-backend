use crate::model::articles::article::{Article, NewArticle, UpdateArticle};
use async_trait::async_trait;

#[async_trait]
pub trait IArticleRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: i32, payload: NewArticle) -> anyhow::Result<Article>;
    async fn find(&self, id: i32) -> anyhow::Result<Article>;
    async fn all(&self) -> anyhow::Result<Vec<Article>>;
    async fn update(&self, id: i32, payload: UpdateArticle) -> anyhow::Result<Article>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}
