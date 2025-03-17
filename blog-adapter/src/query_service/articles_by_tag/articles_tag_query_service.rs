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
        tag_ids: Option<Vec<i32>>,
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
            "#,
        );

        if tag_ids.is_some() {
            query_builder.push("WHERE at.tag_id = ANY($1)");
        }

        query_builder.push(" ORDER BY id DESC;");

        let query = query_builder.build_query_as::<Article>();

        let articles = if let Some(tag_ids) = tag_ids {
            query.bind(tag_ids).fetch_all(&self.pool).await?
        } else {
            query.fetch_all(&self.pool).await?
        };

        Ok(articles)
    }
}
