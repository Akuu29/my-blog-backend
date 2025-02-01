use crate::db::utils::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::tags::{
    i_tag_repository::ITagRepository,
    tag::{NewTag, Tag},
    tag_filter::TagFilter,
};
use sqlx::{types::Uuid, QueryBuilder};

#[derive(Clone)]
pub struct TagRepository {
    pool: sqlx::PgPool,
}

impl TagRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ITagRepository for TagRepository {
    async fn create(&self, user_id: Uuid, payload: NewTag) -> anyhow::Result<Tag> {
        let tag = sqlx::query_as::<_, Tag>(
            r#"
            INSERT INTO tags (
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

        Ok(tag)
    }
    async fn all(&self, tag_filter: TagFilter) -> anyhow::Result<Vec<Tag>> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT
                id,
                name,
                created_at,
                updated_at
            FROM tags
            "#,
        );

        let mut conditions = vec![];

        if tag_filter.user_id.is_some() {
            conditions.push("user_id = $1");
        }

        if !conditions.is_empty() {
            query.push(" WHERE ").push(conditions.join(" AND "));
        }

        query.push(" ORDER BY id DESC; ");
        let tags = query
            .build_query_as::<Tag>()
            .bind(tag_filter.user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(tags)
    }
    async fn delete(&self, tag_id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM tags
            WHERE id = $1;
            "#,
        )
        .bind(tag_id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}
