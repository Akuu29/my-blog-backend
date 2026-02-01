use async_trait::async_trait;
use blog_domain::model::{
    articles::article::{Article, ArticleStatus},
    common::{item_count::ItemCount, pagination::Pagination},
};
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
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "userId")]
    pub user_public_id: Option<Uuid>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub article_status: Option<ArticleStatus>,
}

#[async_trait]
pub trait IArticlesByTagQueryService:
    Clone + std::marker::Send + std::marker::Sync + 'static
{
    async fn find_article_title_by_tag(
        &self,
        filter: ArticlesByTagFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Article>, ItemCount)>;
}
