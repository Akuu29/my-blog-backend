use async_trait::async_trait;
use blog_app::query_service::article_image::i_article_image_query_service::IArticleImageQueryService;
use blog_domain::model::images::image::ImageDataProps;
use sqlx::types::Uuid;

#[derive(Debug, Clone)]
pub struct ArticleImageQueryService {
    pool: sqlx::PgPool,
}

impl ArticleImageQueryService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IArticleImageQueryService for ArticleImageQueryService {
    async fn is_image_owned_by_user(&self, image_id: i32, user_id: Uuid) -> anyhow::Result<bool> {
        let images = sqlx::query_as::<_, ImageDataProps>(
            r#"
            SELECT
                *
            FROM images as i
            INNER JOIN (
                SELECT
                    id as article_id
                FROM articles
                WHERE user_id = $2
            ) as a
            ON i.article_id = a.article_id
            WHERE i.id = $1
            "#,
        )
        .bind(image_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(images.len() > 0)
    }
}
