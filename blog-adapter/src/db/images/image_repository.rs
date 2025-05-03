use crate::db::utils::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::images::{
    i_image_repository::IImageRepository,
    image::{Image, NewImage},
};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct ImageRepository {
    pool: PgPool,
}

impl ImageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IImageRepository for ImageRepository {
    async fn create(&self, new_image: NewImage) -> anyhow::Result<Image> {
        let image = sqlx::query_as::<_, Image>(
            r#"
            INSERT INTO images (
                name,
                mime_type,
                data,
                url,
                storage_type,
                article_id
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6
            )
            RETURNING *;
            "#,
        )
        .bind(new_image.name)
        .bind(new_image.mime_type)
        .bind(new_image.data)
        .bind(new_image.url)
        .bind(new_image.storage_type.to_string())
        .bind(new_image.article_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(image)
    }

    async fn find(&self, image_id: i32) -> anyhow::Result<Image> {
        let image = sqlx::query_as::<_, Image>(
            r#"
            SELECT * FROM images
            WHERE id = $1;
            "#,
        )
        .bind(image_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(image)
    }

    async fn delete(&self, image_id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM images
            WHERE id = $1;
            "#,
        )
        .bind(image_id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}
