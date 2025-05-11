use crate::model::articles::article::{Article, ArticleStatus, NewArticle, UpdateArticle};
use crate::model::common::pagination::Pagination;
use async_trait::async_trait;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use sqlx::types::Uuid;
use validator::Validate;

#[serde_as]
#[derive(Debug, Default, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ArticleFilter {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub user_id: Option<Uuid>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub status: Option<ArticleStatus>,
}

impl ArticleFilter {
    pub fn new(user_id: Option<Uuid>, status: Option<ArticleStatus>) -> Self {
        Self { user_id, status }
    }
}

#[async_trait]
pub trait IArticleRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: Uuid, payload: NewArticle) -> anyhow::Result<Article>;
    async fn find(&self, article_id: i32, article_filter: ArticleFilter)
        -> anyhow::Result<Article>;
    async fn all(
        &self,
        article_filter: ArticleFilter,
        pagination: Pagination,
    ) -> anyhow::Result<Vec<Article>>;
    async fn update(&self, article_id: i32, payload: UpdateArticle) -> anyhow::Result<Article>;
    async fn delete(&self, article_id: i32) -> anyhow::Result<()>;
}
