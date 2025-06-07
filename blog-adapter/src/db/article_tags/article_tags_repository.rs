use crate::utils::repository_error::RepositoryError;
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
    async fn tx_begin(&self) -> anyhow::Result<sqlx::Transaction<'static, sqlx::Postgres>> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Unexpected(e.to_string()))?;

        Ok(tx)
    }

    async fn delete(&self, article_id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM article_tags
            WHERE article_id = $1;
            "#,
        )
        .bind(article_id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }

    async fn bulk_insert(&self, payload: ArticleAttachedTags) -> anyhow::Result<Vec<ArticleTag>> {
        let mut bulk_insert_query_string =
            String::from("INSERT INTO article_tags (article_id, tag_id) VALUES ");
        let mut params = Vec::new();
        let mut values = Vec::new();

        for (i, tag_id) in payload.tag_ids.iter().enumerate() {
            params.push(format!("(${}, ${})", i * 2 + 1, i * 2 + 2));
            values.push((payload.article_id, *tag_id));
        }

        bulk_insert_query_string.push_str(&params.join(","));
        bulk_insert_query_string.push_str(" RETURNING *;");
        let mut query = sqlx::query_as::<_, ArticleTag>(&bulk_insert_query_string);

        for (article_id, tag_id) in values {
            query = query.bind(article_id).bind(tag_id);
        }

        let article_tags = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Unexpected(e.to_string()))?;

        Ok(article_tags)
    }
}
