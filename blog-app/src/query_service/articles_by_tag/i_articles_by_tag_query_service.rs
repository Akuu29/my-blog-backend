use async_trait::async_trait;
use blog_domain::model::{articles::article::Article, common::pagination::Pagination};
use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use uuid::Uuid;
use validator::Validate;

#[serde_as]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ArticlesByTagFilter {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[serde(default)]
    pub tag_ids: Vec<Uuid>,
}

#[async_trait]
pub trait IArticlesByTagQueryService:
    Clone + std::marker::Send + std::marker::Sync + 'static
{
    async fn find_article_title_by_tag(
        &self,
        filter: ArticlesByTagFilter,
        pagination: Pagination,
    ) -> anyhow::Result<Vec<Article>>;
}
