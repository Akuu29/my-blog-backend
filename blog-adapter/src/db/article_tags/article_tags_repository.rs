use crate::db::utils::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::article_tags::{
    article_tags::{ArticleAttachedTags, ArticleTag},
    i_article_tags_repository::IArticleTagsRepository,
};

#[derive(Clone)]
pub struct ArticleTagsRepository {
    pool: sqlx::PgPool,
}

impl ArticleTagsRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IArticleTagsRepository for ArticleTagsRepository {
    async fn delete_insert(&self, payload: ArticleAttachedTags) -> anyhow::Result<Vec<ArticleTag>> {
        // transaction
        let tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
            DELETE FROM article_tags
            WHERE article_id = $1;
            "#,
            payload.article_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        let values: Vec<String> = payload
            .tag_ids
            .iter()
            .map(|tag_id| format!("({}, {})", payload.article_id, tag_id))
            .collect();

        let create_result = sqlx::query_as::<_, ArticleTag>(&format!(
            r#"
                INSERT INTO article_tags (
                    article_id,
                    tag_id
                )
                VALUES {}
                RETURNING *;
                "#,
            values.join(", ")
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Unexpected(e.to_string()));

        match create_result {
            Ok(article_tags) => {
                tx.commit()
                    .await
                    .map_err(|e| RepositoryError::Unexpected(e.to_string()))?;
                Ok(article_tags)
            }
            Err(e) => {
                tx.rollback()
                    .await
                    .map_err(|e| RepositoryError::Unexpected(e.to_string()))?;
                Err(anyhow::anyhow!(e))
            }
        }
    }
}
