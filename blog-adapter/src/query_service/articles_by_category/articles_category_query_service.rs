use async_trait::async_trait;
use blog_app::query_service::articles_by_category::{
    articles_by_category_dto::ArticleByCategoryDto,
    i_articles_by_category_query_service::IArticlesByCategoryQueryService,
};

#[derive(Debug, Clone)]
pub struct ArticlesByCategoryQueryService {
    pool: sqlx::PgPool,
}

impl ArticlesByCategoryQueryService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IArticlesByCategoryQueryService for ArticlesByCategoryQueryService {
    async fn find_article_title_by_category(
        &self,
        category_name: String,
    ) -> anyhow::Result<Vec<ArticleByCategoryDto>> {
        let article_title_and_category = sqlx::query_as::<_, ArticleByCategoryDto>(
            r#"
            SELECT
                articles.id as article_id,
                articles.title as article_title,
                categories.name as category_name
            FROM articles
            INNER JOIN categories
            ON articles.category_id = categories.id
            WHERE categories.name = $1
            "#,
        )
        .bind(category_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(article_title_and_category)
    }
}
