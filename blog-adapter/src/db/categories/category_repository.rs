use crate::utils::repository_error::RepositoryError;
use anyhow::Ok;
use async_trait::async_trait;
use blog_domain::model::categories::{
    category::{Category, CategoryFilter, NewCategory, UpdateCategory},
    i_category_repository::ICategoryRepository,
};
use sqlx::query_builder::QueryBuilder;
use uuid::Uuid;

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
                (SELECT id FROM users WHERE public_id = $2)
            )
            RETURNING
                public_id,
                name,
                created_at,
                updated_at
            ;
            "#,
        )
        .bind(payload.name)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(category)
    }

    async fn all(&self, category_filter: CategoryFilter) -> anyhow::Result<Vec<Category>> {
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                public_id,
                name,
                created_at,
                updated_at
            FROM categories
            "#,
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

        if let Some(public_id) = category_filter.public_id {
            push_condition(&mut qb);
            qb.push("public_id = ").push_bind(public_id);
        }

        if let Some(name) = category_filter.name {
            push_condition(&mut qb);
            qb.push("name = ").push_bind(name);
        }

        qb.push(" ORDER BY id;");

        let categories = qb
            .build_query_as::<Category>()
            .fetch_all(&self.pool)
            .await?;

        Ok(categories)
    }

    async fn update(&self, category_id: Uuid, payload: UpdateCategory) -> anyhow::Result<Category> {
        let category = sqlx::query_as::<_, Category>(
            r#"
            UPDATE categories
            SET
                name = $1
            WHERE public_id = $2
            RETURNING
                public_id,
                name,
                created_at,
                updated_at
            ;
            "#,
        )
        .bind(payload.name)
        .bind(category_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(category)
    }

    async fn delete(&self, category_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM categories
            WHERE public_id = $1
            ;
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
