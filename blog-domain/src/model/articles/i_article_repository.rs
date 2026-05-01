use crate::model::{
    articles::article::{Article, ArticleStatus, NewArticle, UpdateArticle},
    common::{item_count::ItemCount, pagination::Pagination},
    error::RepositoryError,
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
    async fn create(
        &self,
        user_id: Uuid,
        new_article: NewArticle,
    ) -> Result<Article, RepositoryError>;
    async fn find(
        &self,
        article_id: Uuid,
        article_filter: ArticleFilter,
    ) -> Result<Article, RepositoryError>;
    async fn all(
        &self,
        article_filter: ArticleFilter,
        pagination: Pagination,
    ) -> Result<(Vec<Article>, ItemCount), RepositoryError>;
    async fn update(
        &self,
        article_id: Uuid,
        update_article: UpdateArticle,
    ) -> Result<Article, RepositoryError>;
    async fn delete(&self, article_id: Uuid) -> Result<(), RepositoryError>;
    async fn attach_tags(
        &self,
        article_id: Uuid,
        tag_ids: Vec<Uuid>,
    ) -> Result<(), RepositoryError>;
}
