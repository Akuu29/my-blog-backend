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
                t.id,
                t.user_id,
                t.name,
                t.created_at,
                t.updated_at
            FROM tags AS t
            INNER JOIN tagged_articles AS ta ON t.id = ta.tag_id
            WHERE ta.article_id = $1
            "#,
        )
        .bind(article_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }
}

#[cfg(test)]
#[cfg(feature = "database-test")]
mod test {
    use super::*;
    use blog_domain::model::articles::article::ArticleStatus;
    use dotenv::dotenv;
    use serial_test::serial;
    use sqlx::PgPool;

    // Test helper functions
    async fn setup() -> (PgPool, TagsAttachedArticleQueryService, Uuid) {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let query_service = TagsAttachedArticleQueryService::new(pool.clone());

        // Get test user UUID
        let user_id = std::env::var("TEST_USER_ID").expect("undefined TEST_USER_ID");
        let user_uuid = uuid::Uuid::parse_str(&user_id).expect("invalid TEST_USER_ID UUID");

        // Ensure the test user exists (required by foreign key constraints)
        sqlx::query(
            r#"
            INSERT INTO users (id, name)
            VALUES ($1, 'test-user')
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(user_uuid)
        .execute(&pool)
        .await
        .expect("failed to ensure test user exists");

        (pool, query_service, user_uuid)
    }

    struct TestDataGuard {
        pool: PgPool,
        article_ids: Vec<Uuid>,
        tag_ids: Vec<Uuid>,
        runtime_handle: tokio::runtime::Handle,
    }

    impl TestDataGuard {
        fn new(pool: &PgPool) -> Self {
            Self {
                pool: pool.clone(),
                article_ids: Vec::new(),
                tag_ids: Vec::new(),
                runtime_handle: tokio::runtime::Handle::current(),
            }
        }

        fn track_article(&mut self, article_id: Uuid) {
            self.article_ids.push(article_id);
        }

        fn track_tag(&mut self, tag_id: Uuid) {
            self.tag_ids.push(tag_id);
        }
    }

    impl Drop for TestDataGuard {
        fn drop(&mut self) {
            let pool = self.pool.clone();
            let article_ids = self.article_ids.clone();
            let tag_ids = self.tag_ids.clone();
            let handle = self.runtime_handle.clone();

            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::task::block_in_place(|| {
                    handle.block_on(async move {
                        // Cleanup tagged_articles (junction table) first
                        for article_id in &article_ids {
                            let _ =
                                sqlx::query("DELETE FROM tagged_articles WHERE article_id = $1")
                                    .bind(article_id)
                                    .execute(&pool)
                                    .await;
                        }

                        // Then cleanup articles
                        for article_id in &article_ids {
                            let _ = sqlx::query("DELETE FROM articles WHERE id = $1")
                                .bind(article_id)
                                .execute(&pool)
                                .await;
                        }

                        // Finally cleanup tags
                        for tag_id in &tag_ids {
                            let _ = sqlx::query("DELETE FROM tags WHERE id = $1")
                                .bind(tag_id)
                                .execute(&pool)
                                .await;
                        }
                    });
                });
            }));
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_tags_by_article_id() {
        let (pool, query_service, user_uuid) = setup().await;
        let mut guard = TestDataGuard::new(&pool);

        // Create article
        let article_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO articles (title, body, status, user_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(format!("Article {}", Uuid::new_v4()))
        .bind("Test Body")
        .bind(ArticleStatus::Draft)
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .expect("failed to create article");
        guard.track_article(article_id);

        // Create multiple tags
        let tag1_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(format!(
            "tag1-{}",
            Uuid::new_v4().to_string()[0..8].to_string()
        ))
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_tag(tag1_id);

        let tag2_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(format!(
            "tag2-{}",
            Uuid::new_v4().to_string()[0..8].to_string()
        ))
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_tag(tag2_id);

        let tag3_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(format!(
            "tag3-{}",
            Uuid::new_v4().to_string()[0..8].to_string()
        ))
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_tag(tag3_id);

        // Attach tags to article
        for tag_id in &[tag1_id, tag2_id, tag3_id] {
            sqlx::query("INSERT INTO tagged_articles (article_id, tag_id) VALUES ($1, $2)")
                .bind(article_id)
                .bind(tag_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        // Test: Find tags by article ID
        let tags = query_service
            .find_tags_by_article_id(article_id)
            .await
            .unwrap();

        assert_eq!(tags.len(), 3);
        assert!(tags.iter().any(|t| t.id == tag1_id));
        assert!(tags.iter().any(|t| t.id == tag2_id));
        assert!(tags.iter().any(|t| t.id == tag3_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_tags_for_article_with_no_tags() {
        let (pool, query_service, user_uuid) = setup().await;
        let mut guard = TestDataGuard::new(&pool);

        // Create article without any tags
        let article_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO articles (title, body, status, user_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(format!("Article {}", Uuid::new_v4()))
        .bind("Test Body")
        .bind(ArticleStatus::Draft)
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .expect("failed to create article");
        guard.track_article(article_id);

        // Test: Should return empty vector
        let tags = query_service
            .find_tags_by_article_id(article_id)
            .await
            .unwrap();

        assert!(tags.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_tags_for_nonexistent_article() {
        let (_, query_service, _) = setup().await;

        // Test: Nonexistent article should return empty vector
        let nonexistent_article_id = Uuid::new_v4();
        let tags = query_service
            .find_tags_by_article_id(nonexistent_article_id)
            .await
            .unwrap();

        assert!(tags.is_empty());
    }
}
