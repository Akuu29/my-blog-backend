/*
This implementation is behind the current specifications.
When releasing features, review the implementation again.
*/

use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::comments::{
    comment::{Comment, NewComment, UpdateComment},
    i_comment_repository::ICommentRepository,
};

#[derive(Debug, Clone)]
pub struct CommentRepository {
    pool: sqlx::PgPool,
}

impl CommentRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ICommentRepository for CommentRepository {
    async fn create(&self, payload: NewComment) -> anyhow::Result<Comment> {
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            INSERT INTO comments (article_id, body, user_id)
            VALUES ($1, $2, $3)
            RETURNING *;
            "#,
        )
        .bind(payload.article_id)
        .bind(payload.body)
        .bind(payload.user_id)
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

    // TODO Bad approach because it's not scalable
    async fn find_by_article_id(&self, article_id: i32) -> anyhow::Result<Vec<Comment>> {
        let comments = sqlx::query_as::<_, Comment>(
            r#"
            SELECT * FROM comments
            WHERE article_id = $1
            ORDER BY id ASC;
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
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "database-test")]
mod test {
    use super::*;
    use dotenv::dotenv;
    use serial_test::serial;
    use sqlx::PgPool;

    // Test helper functions
    async fn setup() -> (PgPool, CommentRepository, i32, i32) {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let repository = CommentRepository::new(pool.clone());

        // Get test user public_id (UUID) and convert to internal id
        let user_public_id = std::env::var("TEST_USER_ID").expect("undefined TEST_USER_ID");
        let user_id = sqlx::query_scalar::<_, i32>("SELECT id FROM users WHERE public_id = $1")
            .bind(uuid::Uuid::parse_str(&user_public_id).expect("invalid TEST_USER_ID UUID"))
            .fetch_one(&pool)
            .await
            .expect("failed to get user_id from TEST_USER_ID");

        // Create a test article to use for comments
        let article_id = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO articles (title, body, status, user_id)
            VALUES ('Test Article for Comments', 'Test Body', 'draft', $1)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("failed to create test article");

        (pool, repository, user_id, article_id)
    }

    async fn create_test_comment(
        repository: &CommentRepository,
        article_id: i32,
        user_id: i32,
        body: &str,
    ) -> Comment {
        let payload = NewComment {
            article_id,
            body: body.to_string(),
            user_id: Some(user_id),
        };
        repository.create(payload).await.unwrap()
    }

    struct TestCommentGuard {
        pool: PgPool,
        repository: CommentRepository,
        comment_ids: Vec<i32>,
        article_id: Option<i32>,
        runtime_handle: tokio::runtime::Handle,
    }

    impl TestCommentGuard {
        fn new(pool: &PgPool, repository: &CommentRepository, article_id: Option<i32>) -> Self {
            Self {
                pool: pool.clone(),
                repository: repository.clone(),
                comment_ids: Vec::new(),
                article_id,
                runtime_handle: tokio::runtime::Handle::current(),
            }
        }

        fn track(&mut self, comment_id: i32) {
            self.comment_ids.push(comment_id);
        }
    }

    impl Drop for TestCommentGuard {
        fn drop(&mut self) {
            // Clone the data needed for cleanup
            let pool = self.pool.clone();
            let repository = self.repository.clone();
            let comment_ids = self.comment_ids.clone();
            let article_id = self.article_id;
            let handle = self.runtime_handle.clone();

            // Use block_in_place to allow blocking in async context
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::task::block_in_place(|| {
                    handle.block_on(async move {
                        // Cleanup test comments
                        for comment_id in &comment_ids {
                            let _ = repository.delete(*comment_id).await;
                        }

                        // Cleanup test article if created
                        if let Some(article_id) = article_id {
                            let _ = sqlx::query("DELETE FROM articles WHERE id = $1")
                                .bind(article_id)
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
    async fn test_create_comment() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let body = "test comment body";
        let payload = NewComment {
            article_id,
            body: body.to_string(),
            user_id: Some(user_id),
        };

        let comment = repository.create(payload.clone()).await.unwrap();
        guard.track(comment.id);

        assert_eq!(comment.article_id, article_id);
        assert_eq!(comment.body, body);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_comment() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let comment =
            create_test_comment(&repository, article_id, user_id, "test find comment").await;
        guard.track(comment.id);

        let found_comment = repository.find(comment.id).await.unwrap();

        assert_eq!(found_comment.id, comment.id);
        assert_eq!(found_comment.article_id, article_id);
        assert_eq!(found_comment.body, comment.body);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_by_article_id() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let comment1 = create_test_comment(&repository, article_id, user_id, "comment 1").await;
        guard.track(comment1.id);

        let comment2 = create_test_comment(&repository, article_id, user_id, "comment 2").await;
        guard.track(comment2.id);

        let comments = repository.find_by_article_id(article_id).await.unwrap();

        assert!(comments.iter().any(|c| c.id == comment1.id));
        assert!(comments.iter().any(|c| c.id == comment2.id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_update_comment() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let comment = create_test_comment(&repository, article_id, user_id, "original body").await;
        guard.track(comment.id);

        let update_payload = UpdateComment {
            body: Some("updated body".to_string()),
        };

        let updated_comment = repository.update(comment.id, update_payload).await.unwrap();

        assert_eq!(updated_comment.id, comment.id);
        assert_eq!(updated_comment.body, "updated body");
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_delete_comment() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let comment = create_test_comment(&repository, article_id, user_id, "to delete").await;
        guard.track(comment.id);

        repository.delete(comment.id).await.unwrap();

        let result = repository.find(comment.id).await;
        assert!(result.is_err());
    }
}
