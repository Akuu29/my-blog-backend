use async_trait::async_trait;
use blog_domain::model::{
    articles::{
        article_summary::{ArticleSummary, NewArticleSummary},
        i_article_summary_repository::IArticleSummaryRepository,
    },
    error::RepositoryError,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ArticleSummaryRepository {
    pool: sqlx::PgPool,
}

impl ArticleSummaryRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IArticleSummaryRepository for ArticleSummaryRepository {
    async fn upsert(
        &self,
        article_id: Uuid,
        new_summary: NewArticleSummary,
    ) -> Result<ArticleSummary, RepositoryError> {
        let summary = sqlx::query_as::<_, ArticleSummary>(
            r#"
            INSERT INTO article_summaries (article_id, content, lang, locale)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (article_id)
            DO UPDATE SET
                content    = EXCLUDED.content,
                lang       = EXCLUDED.lang,
                locale     = EXCLUDED.locale,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(article_id)
        .bind(&new_summary.content)
        .bind(new_summary.lang.to_string())
        .bind(&new_summary.locale)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Unknown(Box::new(e)))?;

        Ok(summary)
    }

    async fn find(&self, article_id: Uuid) -> Result<ArticleSummary, RepositoryError> {
        let summary = sqlx::query_as::<_, ArticleSummary>(
            r#"
            SELECT article_id, content, lang, locale, created_at, updated_at
            FROM article_summaries
            WHERE article_id = $1
            "#,
        )
        .bind(article_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unknown(Box::new(e)),
        })?;

        Ok(summary)
    }

    async fn delete(&self, article_id: Uuid) -> Result<(), RepositoryError> {
        let result = sqlx::query(
            r#"
            DELETE FROM article_summaries
            WHERE article_id = $1
            "#,
        )
        .bind(article_id)
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Unknown(Box::new(e)))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }

        Ok(())
    }
}
