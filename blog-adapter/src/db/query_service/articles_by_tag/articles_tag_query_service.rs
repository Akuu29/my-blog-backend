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
            SELECT
                DISTINCT ON (a.id)
                a.id as id,
                a.title as title,
                a.body as body,
                a.status as status,
                a.user_id as user_id,
                a.category_id as category_id,
                a.created_at as created_at,
                a.updated_at as updated_at
            FROM articles as a
            INNER JOIN article_tags as at ON a.id = at.article_id
            WHERE at.tag_id = ANY($1)
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
