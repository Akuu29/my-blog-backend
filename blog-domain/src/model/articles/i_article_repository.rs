use crate::model::articles::article::{Article, ArticleStatus, NewArticle, UpdateArticle};
use crate::model::common::pagination::Pagination;
use async_trait::async_trait;
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
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
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "categoryId")]
    pub category_public_id: Option<Uuid>,
}

impl ArticleFilter {
    pub fn new(
        user_id: Option<Uuid>,
        status: Option<ArticleStatus>,
        category_public_id: Option<Uuid>,
    ) -> Self {
        Self {
            user_id,
            status,
            category_public_id,
        }
    }
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
    ) -> anyhow::Result<Vec<Article>>;
    async fn update(
        &self,
        article_id: Uuid,
        update_article: UpdateArticle,
    ) -> anyhow::Result<Article>;
    async fn delete(&self, article_id: Uuid) -> anyhow::Result<()>;
    async fn attach_tags(&self, article_id: Uuid, tag_ids: Vec<Uuid>) -> anyhow::Result<()>;
}
