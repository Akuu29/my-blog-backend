use crate::model::articles::{
    article::{Article, NewArticle, UpdateArticle},
    article_filter::ArticleFilter,
};
use async_trait::async_trait;
use sqlx::types::Uuid;

#[async_trait]
pub trait IArticleRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: Uuid, payload: NewArticle) -> anyhow::Result<Article>;
    async fn find(
        &self,
        article_id: i32,
        article_filter: Option<ArticleFilter>,
    ) -> anyhow::Result<Article>;
    async fn all(&self) -> anyhow::Result<Vec<Article>>;
    async fn update(&self, article_id: i32, payload: UpdateArticle) -> anyhow::Result<Article>;
    async fn delete(&self, article_id: i32) -> anyhow::Result<()>;
}
