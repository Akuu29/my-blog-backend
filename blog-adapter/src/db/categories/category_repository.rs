use crate::utils::repository_error::RepositoryError;
use anyhow::Ok;
use async_trait::async_trait;
use blog_domain::model::{
    categories::{
        category::{Category, NewCategory, UpdateCategory},
        i_category_repository::{CategoryFilter, ICategoryRepository},
    },
    common::{item_count::ItemCount, pagination::Pagination},
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

    fn push_category_condition(
        &self,
        qb: &mut QueryBuilder<'_, sqlx::Postgres>,
        filter: &CategoryFilter,
    ) -> bool {
        let mut has_condition = false;
        let mut push_condition = |qb: &mut QueryBuilder<'_, sqlx::Postgres>| {
            if !has_condition {
                qb.push(" WHERE ");
                has_condition = true;
            } else {
                qb.push(" AND ");
            }
        };

        if let Some(public_id) = filter.public_id {
            push_condition(qb);
            qb.push("c.public_id = ").push_bind(public_id);
        }

        if let Some(name) = filter.name.clone() {
            push_condition(qb);
            qb.push("c.name = ").push_bind(name);
        }

        if let Some(user_public_id) = filter.user_public_id {
            push_condition(qb);
            qb.push("u.public_id = ").push_bind(user_public_id);
        }

        return has_condition;
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

    async fn all(
        &self,
        category_filter: CategoryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Category>, ItemCount)> {
        // find categories
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                c.public_id,
                c.name,
                c.created_at,
                c.updated_at
            FROM categories AS c
            LEFT JOIN users AS u ON c.user_id = u.id
            "#,
        );

        // build conditions
        let has_condition = self.push_category_condition(&mut qb, &category_filter);

        if let Some(cursor) = pagination.cursor {
            let cid_option = sqlx::query_scalar!(
                r#"
                SELECT id FROM categories WHERE public_id = $1
                "#,
                cursor
            )
            .fetch_optional(&self.pool)
            .await?;

            let cid = cid_option.ok_or(RepositoryError::NotFound)?;
            if has_condition {
                qb.push(" AND ");
            } else {
                qb.push(" WHERE ");
            }
            qb.push("c.id < ").push_bind(cid);
        }

        qb.push(" ORDER BY c.id DESC LIMIT ")
            .push_bind(pagination.per_page);

        let categories = qb
            .build_query_as::<Category>()
            .fetch_all(&self.pool)
            .await?;

        // count total categories
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                COUNT(*)
            FROM categories AS c
            LEFT JOIN users AS u ON c.user_id = u.id
            "#,
        );
        // build conditions
        self.push_category_condition(&mut qb, &category_filter);
        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await?;

        Ok((categories, total))
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
