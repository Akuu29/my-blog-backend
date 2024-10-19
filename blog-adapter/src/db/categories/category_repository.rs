use crate::db::utils::RepositoryError;
use anyhow::Ok;
use async_trait::async_trait;
use blog_domain::model::categories::{
    category::{Category, NewCategory, UpdateCategory},
    i_category_repository::ICategoryRepository,
};
use sqlx::types::Uuid;

#[derive(Clone)]
pub struct CategoryRepository {
    pool: sqlx::PgPool,
}

impl CategoryRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ICategoryRepository for CategoryRepository {
    async fn create(&self, user_id: Uuid, payload: NewCategory) -> anyhow::Result<Category> {
        let category = sqlx::query_as::<_, Category>(
            r#"
            INSERT INTO categories (
                name,
                user_id
            )
            VALUES (
                $1,
                $2
            )
            RETURNING *;
            "#,
        )
        .bind(payload.name)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(category)
    }

    async fn all(&self) -> anyhow::Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>(
            r#"
            SELECT
                id,
                name,
                created_at,
                updated_at
            FROM categories
            ORDER BY id;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(categories)
    }

    async fn update(&self, category_id: i32, payload: UpdateCategory) -> anyhow::Result<Category> {
        let category = sqlx::query_as::<_, Category>(
            r#"
            UPDATE categories
            SET
                name = $1
            WHERE id = $2
            RETURNING *;
            "#,
        )
        .bind(payload.name)
        .bind(category_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(category)
    }

    async fn delete(&self, category_id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM categories
            WHERE id = $1;
            "#,
        )
        .bind(category_id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}
