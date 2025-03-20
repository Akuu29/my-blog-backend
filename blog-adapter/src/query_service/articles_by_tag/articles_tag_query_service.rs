use async_trait::async_trait;
use blog_app::query_service::articles_by_tag::i_articles_by_tag_query_service::IArticlesByTagQueryService;
use blog_domain::model::articles::article::Article;
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
        tag_ids: Vec<i32>,
        cursor: Option<i32>,
        per_page: i32,
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

        if cursor.is_some() {
            query_builder.push("AND a.id < $2");
        }

        query_builder.push("ORDER BY a.id DESC LIMIT $3;");

        let articles = query_builder
            .build_query_as::<Article>()
            .bind(tag_ids)
            .bind(cursor)
            .bind(per_page)
            .fetch_all(&self.pool)
            .await?;

        Ok(articles)
    }
}
