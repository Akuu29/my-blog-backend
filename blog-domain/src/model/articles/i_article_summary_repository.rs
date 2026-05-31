use crate::model::{
    articles::article_summary::{ArticleSummary, NewArticleSummary},
    error::RepositoryError,
};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait IArticleSummaryRepository: Clone + Send + Sync + 'static {
    async fn upsert(
        &self,
        article_id: Uuid,
        new_summary: NewArticleSummary,
    ) -> Result<ArticleSummary, RepositoryError>;
}
