use async_trait::async_trait;
use blog_domain::model::tags::tag::Tag;

#[async_trait]
pub trait ITagsAttachedArticleQueryService:
    Clone + std::marker::Send + std::marker::Sync + 'static
{
    async fn find_tags_by_article_id(&self, article_id: i32) -> anyhow::Result<Vec<Tag>>;
}
