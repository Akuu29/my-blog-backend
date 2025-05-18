use crate::model::article_tags::article_tags::{ArticleAttachedTags, ArticleTag};
use async_trait::async_trait;

#[async_trait]
pub trait IArticleTagsRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn tx_begin(&self) -> anyhow::Result<sqlx::Transaction<'static, sqlx::Postgres>>;
    async fn delete(&self, article_id: i32) -> anyhow::Result<()>;
    async fn bulk_insert(&self, payload: ArticleAttachedTags) -> anyhow::Result<Vec<ArticleTag>>;
}
