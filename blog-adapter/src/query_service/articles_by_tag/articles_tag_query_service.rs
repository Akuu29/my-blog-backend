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
        tag_ids: Option<Vec<String>>,
    ) -> anyhow::Result<Vec<Article>> {
        let mut query = QueryBuilder::new(
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
            "#,
        );

        let mut conditions = vec![];

        if tag_ids.is_some() {
            query.push("INNER JOIN article_tags as at ON a.id = at.article_id");

            let tag_ids = tag_ids.unwrap();
            let tag_ids_str = tag_ids.join(",");
            conditions.push(format!("at.tag_id IN ({})", tag_ids_str));
        }

        if !conditions.is_empty() {
            query.push(" WHERE ").push(conditions.join(" AND "));
        }

        query.push(" ORDER BY id DESC;");

        let articles = query
            .build_query_as::<Article>()
            .fetch_all(&self.pool)
            .await?;

        Ok(articles)
    }
}
