use async_trait::async_trait;
use blog_app::query_service::tags_attached_article::i_tags_attached_article_query_service::ITagsAttachedArticleQueryService;
use blog_domain::model::tags::tag::Tag;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TagsAttachedArticleQueryService {
    pool: sqlx::PgPool,
}

impl TagsAttachedArticleQueryService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ITagsAttachedArticleQueryService for TagsAttachedArticleQueryService {
    async fn find_tags_by_article_id(&self, article_id: Uuid) -> anyhow::Result<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>(
            r#"
            SELECT
                t.public_id AS public_id,
                t.name AS name,
                t.created_at AS created_at,
                t.updated_at AS updated_at
            FROM tags AS t
            INNER JOIN article_tags AS at ON t.id = at.tag_id
            WHERE at.article_id = (SELECT id FROM articles WHERE public_id = $1)
            "#,
        )
        .bind(article_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }
}
