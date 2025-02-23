use async_trait::async_trait;
use blog_domain::model::articles::article::Article;

#[async_trait]
pub trait IArticlesByTagQueryService:
    Clone + std::marker::Send + std::marker::Sync + 'static
{
    async fn find_article_title_by_tag(
        &self,
        tag_ids: Option<Vec<String>>,
    ) -> anyhow::Result<Vec<Article>>;
}
