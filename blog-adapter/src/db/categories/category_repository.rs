use crate::db::utils::RepositoryError;
use anyhow::Ok;
use async_trait::async_trait;
use blog_domain::model::categories::{
    category::{Category, CategoryFilter, NewCategory, UpdateCategory},
    i_category_repository::ICategoryRepository,
};
use sqlx::{query_builder::QueryBuilder, types::Uuid};

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

    async fn all(&self, category_filter: CategoryFilter) -> anyhow::Result<Vec<Category>> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT
                id,
                name,
                created_at,
                updated_at
            FROM categories
            "#,
        );

        let mut conditions = vec![];

        if category_filter.id.is_some() {
            conditions.push("id = $1");
        }

        if category_filter.name.is_some() {
            conditions.push("name = $2");
        }

        if !conditions.is_empty() {
            query.push(" WHERE ").push(conditions.join(" AND "));
        }

        query.push(" ORDER BY id;");

        let categories = query
            .build_query_as::<Category>()
            .bind(category_filter.id)
            .bind(category_filter.name)
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
