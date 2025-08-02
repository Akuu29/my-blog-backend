use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::tags::{
    i_tag_repository::{ITagRepository, TagFilter},
    tag::{NewTag, Tag},
};
use sqlx::QueryBuilder;
use uuid::Uuid;

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

        Ok(tag)
    }
    async fn all(&self, tag_filter: TagFilter) -> anyhow::Result<Vec<Tag>> {
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                t.public_id,
                t.name,
                t.created_at,
                t.updated_at
            FROM tags AS t
            LEFT JOIN users AS u
            ON t.user_id = u.id
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

        if let Some(user_id) = tag_filter.user_id {
            push_condition(&mut qb);
            qb.push("u.public_id = ").push_bind(user_id);
        }

        if let Some(tag_ids) = tag_filter.tag_ids {
            push_condition(&mut qb);
            qb.push("t.public_id = ANY(").push_bind(tag_ids).push(")");
        }

        qb.push(" ORDER BY t.id DESC; ");
        let tags = qb.build_query_as::<Tag>().fetch_all(&self.pool).await?;

        Ok(tags)
    }

    async fn delete(&self, tag_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM tags
            WHERE public_id = $1
            ;
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
