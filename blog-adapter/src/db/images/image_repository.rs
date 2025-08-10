use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::images::{
    i_image_repository::{IImageRepository, ImageFilter},
    image::{ImageData, ImageDataProps, NewImage},
};
use sqlx::QueryBuilder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ImageRepository {
    pool: sqlx::PgPool,
}

impl ImageRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IImageRepository for ImageRepository {
    async fn create(&self, new_image: NewImage) -> anyhow::Result<ImageDataProps> {
        let image = sqlx::query_as::<_, ImageDataProps>(
            r#"
            WITH sel_storage_type_id AS (
                    SELECT id FROM storage_types WHERE name = $5
                ),
                sel_article_id AS (
                    SELECT id FROM articles WHERE public_id = $6
                )
            INSERT INTO images (
                name,
                mime_type,
                data,
                url,
                storage_type_id,
                article_id
            )
            SELECT
                $1,
                $2,
                $3,
                $4,
                sel_storage_type_id.id,
                sel_article_id.id
            FROM sel_storage_type_id, sel_article_id
            RETURNING
                public_id,
                name,
                mime_type,
                url,
                $5 AS storage_type,
                $6 AS article_public_id,
                created_at,
                updated_at
            ;
            "#,
        )
        .bind(new_image.name)
        .bind(new_image.mime_type)
        .bind(new_image.data)
        .bind(new_image.url)
        .bind(new_image.storage_type.to_string())
        .bind(new_image.article_public_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(image)
    }

    async fn all(&self, filter: ImageFilter) -> anyhow::Result<Vec<ImageDataProps>> {
        let mut qb = QueryBuilder::new(
            r"
            SELECT
                i.public_id,
                i.name,
                i.mime_type,
                i.url,
                st.name AS storage_type,
                a.public_id AS article_public_id,
                i.created_at,
                i.updated_at
            FROM images AS i
            LEFT JOIN storage_types AS st
            ON i.storage_type_id = st.id
            LEFT JOIN articles AS a
            ON i.article_id = a.id
            ",
        );

        let mut first = true;
        let mut push_condition = |qb: &mut QueryBuilder<'_, sqlx::Postgres>| {
            if first {
                qb.push(" WHERE ");
                first = false;
            } else {
                qb.push(" AND ");
            }
        };

        if let Some(article_public_id) = filter.article_public_id {
            push_condition(&mut qb);
            qb.push("a.public_id = ").push_bind(article_public_id);
        }

        qb.push(" ORDER BY i.id DESC;");

        let images = qb
            .build_query_as::<ImageDataProps>()
            .fetch_all(&self.pool)
            .await?;

        Ok(images)
    }

    async fn find_data(&self, image_id: Uuid) -> anyhow::Result<ImageData> {
        let image_data = sqlx::query_as::<_, ImageData>(
            r#"
            SELECT
                mime_type,
                data
            FROM images
            WHERE public_id = $1
            ;
            "#,
        )
        .bind(image_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(image_data)
    }

    async fn delete(&self, image_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM images
            WHERE public_id = $1
            ;
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
