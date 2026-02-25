use crate::model::{
    articles::article::{Article, ArticleStatus, NewArticle, UpdateArticle},
    common::{item_count::ItemCount, pagination::Pagination},
};
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::types::Uuid;
use validator::Validate;

#[derive(Debug, Default, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ArticleFilter {
    pub user_id: Option<Uuid>,
    pub status: Option<ArticleStatus>,
    pub category_id: Option<Uuid>,
    #[validate(length(max = 100, message = "title_contains length must be 100 or less"))]
    pub title_contains: Option<String>,
}

#[async_trait]
pub trait IArticleRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, user_id: Uuid, new_article: NewArticle) -> anyhow::Result<Article>;
    async fn find(
        &self,
        article_id: Uuid,
        article_filter: ArticleFilter,
    ) -> anyhow::Result<Article>;
    async fn all(
        &self,
        article_filter: ArticleFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Article>, ItemCount)>;
    async fn update(
        &self,
        article_id: Uuid,
        update_article: UpdateArticle,
    ) -> anyhow::Result<Article>;
    async fn delete(&self, article_id: Uuid) -> anyhow::Result<()>;
    async fn attach_tags(&self, article_id: Uuid, tag_ids: Vec<Uuid>) -> anyhow::Result<()>;
}
