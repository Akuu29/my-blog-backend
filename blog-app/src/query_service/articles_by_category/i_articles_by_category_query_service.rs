use crate::query_service::articles_by_category::articles_by_category_dto::ArticleByCategoryDto;
use async_trait::async_trait;

#[async_trait]
pub trait IArticlesByCategoryQueryService:
    Clone + std::marker::Send + std::marker::Sync + 'static
{
    async fn find_article_title_by_category(
        &self,
        category_name: String,
    ) -> anyhow::Result<Vec<ArticleByCategoryDto>>;
}
