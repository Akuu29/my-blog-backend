use crate::repository::RepositoryError;
use async_trait::async_trait;
use blog_domain::{
    model::comment::{Comment, NewComment, UpdateComment},
    repository::comment::CommentRepository,
};

#[derive(Debug, Clone)]
pub struct CommentRepositoryForDb {
    pool: sqlx::PgPool,
}

impl CommentRepositoryForDb {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommentRepository for CommentRepositoryForDb {
    async fn create(&self, payload: NewComment) -> anyhow::Result<Comment> {
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            INSERT INTO comments (article_id, body)
            VALUES ($1, $2)
            RETURNING *;
            "#,
        )
        .bind(payload.article_id)
        .bind(payload.body)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Comment> {
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            SELECT * FROM comments
            WHERE id = $1;
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn find_by_article_id(&self, article_id: i32) -> anyhow::Result<Vec<Comment>> {
        let comments = sqlx::query_as::<_, Comment>(
            r#"
            SELECT * FROM comments
            WHERE article_id = $1;
            "#,
        )
        .bind(article_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    async fn update(&self, id: i32, payload: UpdateComment) -> anyhow::Result<Comment> {
        let pre_comment = self.find(id).await?;
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            UPDATE comments set body=$1, updated_at=now()
            WHERE id = $2
            RETURNING *;
            "#,
        )
        .bind(payload.body.unwrap_or(pre_comment.body))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM comments
            WHERE id = $1;
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}
