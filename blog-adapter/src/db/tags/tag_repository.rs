use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::{
    common::{item_count::ItemCount, pagination::Pagination},
    tags::{
        i_tag_repository::{ITagRepository, TagFilter},
        tag::{NewTag, Tag},
    },
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

    /// push tag where conditions to the query builder
    fn push_tag_condition(
        &self,
        qb: &mut QueryBuilder<'_, sqlx::Postgres>,
        filter: &TagFilter,
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

        if let Some(user_id) = filter.user_id {
            push_condition(qb);
            qb.push("u.public_id = ").push_bind(user_id);
        }

        if let Some(tag_ids) = filter.tag_ids.clone() {
            push_condition(qb);
            qb.push("t.public_id = ANY(").push_bind(tag_ids).push(")");
        }

        return has_condition;
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

    async fn all(
        &self,
        tag_filter: TagFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Tag>, ItemCount)> {
        // find tags
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

        // build conditions
        let has_condition = self.push_tag_condition(&mut qb, &tag_filter);

        if let Some(cursor) = pagination.cursor {
            let cid_option = sqlx::query_scalar!(
                r#"
                SELECT id FROM tags WHERE public_id = $1
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
            qb.push("t.id < ").push_bind(cid);
        }

        qb.push(" ORDER BY t.id DESC LIMIT ")
            .push_bind(pagination.per_page);

        let tags = qb.build_query_as::<Tag>().fetch_all(&self.pool).await?;

        // count total tags
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                COUNT(*)
            FROM tags AS t
            LEFT JOIN users AS u ON t.user_id = u.id
            "#,
        );
        // build conditions
        self.push_tag_condition(&mut qb, &tag_filter);

        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await?;

        Ok((tags, total))
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
