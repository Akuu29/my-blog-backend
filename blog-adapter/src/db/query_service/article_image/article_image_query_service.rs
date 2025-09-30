use async_trait::async_trait;
use blog_app::query_service::article_image::i_article_image_query_service::IArticleImageQueryService;
use blog_domain::model::images::image::ImageDataProps;
use uuid::Uuid;

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
    async fn is_image_owned_by_user(&self, image_id: Uuid, user_id: Uuid) -> anyhow::Result<bool> {
        let images = sqlx::query_as::<_, ImageDataProps>(
            r#"
            SELECT
                i.public_id AS public_id,
                i.name AS name,
                i.mime_type AS mime_type,
                i.url AS url,
                st.name AS storage_type,
                a.public_id AS article_public_id,
                i.created_at AS created_at,
                i.updated_at AS updated_at
            FROM images AS i
            INNER JOIN (
                SELECT
                    a.id,
                    a.public_id
                FROM articles as a
                RIGHT JOIN users AS u
                ON a.user_id = u.id
                WHERE u.public_id = $2
            ) AS a
            ON i.article_id = a.id
            LEFT JOIN storage_types AS st
            ON i.storage_type_id = st.id
            WHERE i.public_id = $1
            ;
            "#,
        )
        .bind(image_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(images.len() > 0)
    }
}
