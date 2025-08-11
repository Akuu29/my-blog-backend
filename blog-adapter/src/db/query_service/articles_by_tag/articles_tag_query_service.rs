use async_trait::async_trait;
use blog_app::query_service::articles_by_tag::i_articles_by_tag_query_service::{
    ArticlesByTagFilter, IArticlesByTagQueryService,
};
use blog_domain::model::{articles::article::Article, common::pagination::Pagination};
use sqlx::query_builder::QueryBuilder;

#[derive(Debug, Clone)]
pub struct ArticlesByTagQueryService {
    pool: sqlx::PgPool,
}

impl ArticlesByTagQueryService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IArticlesByTagQueryService for ArticlesByTagQueryService {
    async fn find_article_title_by_tag(
        &self,
        filter: ArticlesByTagFilter,
        pagination: Pagination,
    ) -> anyhow::Result<Vec<Article>> {
        let mut query_builder = QueryBuilder::new(
            r#"
            WITH tag_ids AS (
                SELECT id
                FROM tags
                WHERE public_id = ANY ($1)
            )
            SELECT
                public_id,
                title,
                body,
                status,
                (SELECT public_id FROM categories WHERE id = category_id) as category_public_id,
                created_at,
                updated_at
            FROM articles a
            WHERE EXISTS (
                SELECT 1
                FROM article_tags AS at
                WHERE at.article_id = a.id
                AND at.tag_id = ANY (SELECT id FROM tag_ids)
            )
            "#,
        );

        if pagination.cursor.is_some() {
            query_builder.push("AND a.id < $2");
        }

        query_builder.push("ORDER BY a.id DESC LIMIT $3;");

        let articles = query_builder
            .build_query_as::<Article>()
            .bind(filter.tag_ids)
            .bind(pagination.cursor)
            .bind(pagination.per_page)
            .fetch_all(&self.pool)
            .await?;

        Ok(articles)
    }
}
