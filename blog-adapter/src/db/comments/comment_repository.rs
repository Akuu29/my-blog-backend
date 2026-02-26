use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::{
    comments::{
        comment::{Comment, NewComment, UpdateComment},
        i_comment_repository::{CommentFilter, ICommentRepository},
    },
    common::{item_count::ItemCount, pagination::Pagination},
};
use sqlx::QueryBuilder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CommentRepository {
    pool: sqlx::PgPool,
}

impl CommentRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Push comment where conditions to the query builder
    fn push_comment_condition(
        &self,
        qb: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
        filter: &CommentFilter,
    ) -> bool {
        let mut has_condition = false;
        let mut push_condition = |qb: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>| {
            if !has_condition {
                qb.push(" WHERE ");
                has_condition = true;
            } else {
                qb.push(" AND ");
            }
        };

        if let Some(article_id) = filter.article_id {
            push_condition(qb);
            qb.push("c.article_id = ").push_bind(article_id);
        }

        if let Some(user_id) = filter.user_id {
            push_condition(qb);
            qb.push("c.user_id = ").push_bind(user_id);
        }

        if let Some(user_name) = filter.user_name.clone() {
            push_condition(qb);
            qb.push("(u.name ILIKE ")
                .push_bind(format!("%{}%", user_name))
                .push(" OR c.user_name ILIKE ")
                .push_bind(format!("%{}%", user_name))
                .push(")");
        }

        has_condition
    }
}

#[async_trait]
impl ICommentRepository for CommentRepository {
    async fn create(
        &self,
        user_id: Option<Uuid>,
        user_name: String,
        payload: NewComment,
    ) -> anyhow::Result<Comment> {
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            INSERT INTO comments (article_id, body, user_id, user_name)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, user_name, article_id, body, created_at, updated_at
            "#,
        )
        .bind(payload.article_id)
        .bind(payload.body)
        .bind(user_id)
        .bind(user_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn find(
        &self,
        comment_id: Uuid,
        comment_filter: CommentFilter,
    ) -> anyhow::Result<Comment> {
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                c.id,
                c.user_id,
                COALESCE(u.name, c.user_name) AS user_name,
                c.article_id,
                c.body,
                c.created_at,
                c.updated_at
            FROM comments AS c
            LEFT JOIN users AS u ON c.user_id = u.id
            WHERE c.id =
            "#,
        );

        qb.push_bind(comment_id);

        // Apply additional filters
        if let Some(article_id) = comment_filter.article_id {
            qb.push(" AND c.article_id = ").push_bind(article_id);
        }

        if let Some(user_id) = comment_filter.user_id {
            qb.push(" AND c.user_id = ").push_bind(user_id);
        }

        if let Some(user_name) = comment_filter.user_name {
            qb.push(" AND (u.name ILIKE ")
                .push_bind(format!("%{}%", user_name))
                .push(" OR c.user_name ILIKE ")
                .push_bind(format!("%{}%", user_name))
                .push(")");
        }

        let comment = qb
            .build_query_as::<Comment>()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => RepositoryError::NotFound,
                e => RepositoryError::Unexpected(e.to_string()),
            })?;

        Ok(comment)
    }

    async fn all(
        &self,
        comment_filter: CommentFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Comment>, ItemCount)> {
        // Find comments with pagination
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                c.id,
                c.user_id,
                COALESCE(u.name, c.user_name) AS user_name,
                c.article_id,
                c.body,
                c.created_at,
                c.updated_at
            FROM comments AS c
            LEFT JOIN users AS u ON c.user_id = u.id
            "#,
        );

        // Build filter conditions
        let has_condition = self.push_comment_condition(&mut qb, &comment_filter);

        // Handle cursor-based pagination
        if let Some(cursor) = pagination.cursor {
            let cursor_ts_option =
                sqlx::query_scalar::<_, sqlx::types::chrono::DateTime<sqlx::types::chrono::Local>>("SELECT created_at FROM comments WHERE id = $1")
                    .bind(cursor)
                    .fetch_optional(&self.pool)
                    .await?;

            let cursor_ts = cursor_ts_option.ok_or(RepositoryError::NotFound)?;
            if has_condition {
                qb.push(" AND ");
            } else {
                qb.push(" WHERE ");
            }
            qb.push("(c.created_at, c.id) < (").push_bind(cursor_ts).push(", ").push_bind(cursor).push(")");
        }

        qb.push(" ORDER BY c.created_at DESC, c.id DESC");

        if let Some(offset) = pagination.offset {
            qb.push(" OFFSET ").push_bind(offset);
        }

        qb.push(" LIMIT ").push_bind(pagination.per_page);

        let comments = qb.build_query_as::<Comment>().fetch_all(&self.pool).await?;

        // Count total comments
        let mut qb = QueryBuilder::new(
            r#"
            SELECT COUNT(*)
            FROM comments AS c
            LEFT JOIN users AS u ON c.user_id = u.id
            "#,
        );

        self.push_comment_condition(&mut qb, &comment_filter);

        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await?;

        Ok((comments, total))
    }

    async fn update(&self, comment_id: Uuid, payload: UpdateComment) -> anyhow::Result<Comment> {
        let pre_comment = self.find(comment_id, CommentFilter::default()).await?;
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            UPDATE comments
            SET body = $1, updated_at = now()
            WHERE id = $2
            RETURNING
                id,
                user_id,
                COALESCE(
                    (SELECT name FROM users WHERE id = comments.user_id),
                    user_name
                ) AS user_name,
                article_id,
                body,
                created_at,
                updated_at
            ;
            "#,
        )
        .bind(payload.body.unwrap_or(pre_comment.body))
        .bind(comment_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn delete(&self, comment_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM comments
            WHERE id = $1
            ;
            "#,
        )
        .bind(comment_id)
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
    use blog_domain::model::comments::i_comment_repository::CommentFilter;
    use blog_domain::model::common::pagination::Pagination;
    use dotenv::dotenv;
    use serial_test::serial;
    use sqlx::PgPool;

    // Test helper functions
    async fn setup() -> (PgPool, CommentRepository, Uuid, Uuid) {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let repository = CommentRepository::new(pool.clone());

        // Get test user id (UUID)
        let user_id =
            Uuid::parse_str(&std::env::var("TEST_USER_ID").expect("undefined TEST_USER_ID"))
                .expect("invalid TEST_USER_ID");

        // Ensure the test user exists (required by articles/comments foreign keys)
        sqlx::query(
            r#"
            INSERT INTO users (id, name)
            VALUES ($1, 'test-user')
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("failed to ensure test user exists");

        // Create a test article to use for comments
        let article_id = sqlx::query_scalar::<_, Uuid>(
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
        article_id: Uuid,
        user_id: Uuid,
        body: &str,
    ) -> Comment {
        let payload = NewComment {
            article_id,
            body: body.to_string(),
            user_name: None,
        };
        repository
            .create(Some(user_id), "test-user".to_string(), payload)
            .await
            .unwrap()
    }

    async fn create_test_comment_guest(
        repository: &CommentRepository,
        article_id: Uuid,
        user_name: &str,
        body: &str,
    ) -> Comment {
        let payload = NewComment {
            article_id,
            body: body.to_string(),
            user_name: Some(user_name.to_string()),
        };
        repository
            .create(None, user_name.to_string(), payload)
            .await
            .unwrap()
    }

    struct TestCommentGuard {
        pool: PgPool,
        repository: CommentRepository,
        comment_ids: Vec<Uuid>,
        article_id: Option<Uuid>,
        runtime_handle: tokio::runtime::Handle,
    }

    impl TestCommentGuard {
        fn new(
            pool: &PgPool,
            repository: &CommentRepository,
            article_id: Option<Uuid>,
        ) -> Self {
            Self {
                pool: pool.clone(),
                repository: repository.clone(),
                comment_ids: Vec::new(),
                article_id,
                runtime_handle: tokio::runtime::Handle::current(),
            }
        }

        fn track(&mut self, comment_id: Uuid) {
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
    async fn test_create_comment_logged_in_user() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let body = "test comment body";
        let user_name = "test-user";
        let payload = NewComment {
            article_id,
            body: body.to_string(),
            user_name: None,
        };

        let comment = repository
            .create(Some(user_id), user_name.to_string(), payload)
            .await
            .unwrap();
        guard.track(comment.id);

        assert_eq!(comment.article_id, article_id);
        assert_eq!(comment.body, body);
        assert_eq!(comment.user_id, Some(user_id));
        assert_eq!(comment.user_name, user_name);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_create_comment_guest_user() {
        let (pool, repository, _user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let body = "guest comment body";
        let guest_name = "Anonymous Guest";
        let payload = NewComment {
            article_id,
            body: body.to_string(),
            user_name: Some(guest_name.to_string()),
        };

        let comment = repository
            .create(None, guest_name.to_string(), payload)
            .await
            .unwrap();
        guard.track(comment.id);

        assert_eq!(comment.article_id, article_id);
        assert_eq!(comment.body, body);
        assert_eq!(comment.user_id, None);
        assert_eq!(comment.user_name, guest_name);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_comment() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let comment = create_test_comment(
            &repository,
            article_id,
            user_id,
            "test find comment",
        )
        .await;
        guard.track(comment.id);

        let found_comment = repository
            .find(comment.id, CommentFilter::default())
            .await
            .unwrap();

        assert_eq!(found_comment.id, comment.id);
        assert_eq!(found_comment.article_id, article_id);
        assert_eq!(found_comment.body, comment.body);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_comments_with_pagination() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let comment1 =
            create_test_comment(&repository, article_id, user_id, "comment 1").await;
        guard.track(comment1.id);

        let comment2 =
            create_test_comment(&repository, article_id, user_id, "comment 2").await;
        guard.track(comment2.id);

        let comment3 = create_test_comment_guest(
            &repository,
            article_id,
            "Guest User",
            "guest comment",
        )
        .await;
        guard.track(comment3.id);

        let (comments, total) = repository
            .all(
                CommentFilter {
                    article_id: Some(article_id),
                    ..Default::default()
                },
                Pagination {
                    per_page: 10,
                    ..Pagination::default()
                },
            )
            .await
            .unwrap();

        assert!(comments.len() >= 3);
        assert!(total.value() >= 3);
        assert!(comments.iter().any(|c| c.id == comment1.id));
        assert!(comments.iter().any(|c| c.id == comment2.id));
        assert!(comments.iter().any(|c| c.id == comment3.id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_update_comment() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let comment = create_test_comment(
            &repository,
            article_id,
            user_id,
            "original body",
        )
        .await;
        guard.track(comment.id);

        let update_payload = UpdateComment {
            body: Some("updated body".to_string()),
        };

        let updated_comment = repository
            .update(comment.id, update_payload)
            .await
            .unwrap();

        assert_eq!(updated_comment.id, comment.id);
        assert_eq!(updated_comment.body, "updated body");
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_delete_comment() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let comment =
            create_test_comment(&repository, article_id, user_id, "to delete").await;
        guard.track(comment.id);

        repository.delete(comment.id).await.unwrap();

        let result = repository
            .find(comment.id, CommentFilter::default())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_comment_with_filter() {
        let (pool, repository, user_id, article_id) = setup().await;
        let mut guard = TestCommentGuard::new(&pool, &repository, Some(article_id));

        let comment = create_test_comment(
            &repository,
            article_id,
            user_id,
            "comment with filter",
        )
        .await;
        guard.track(comment.id);

        // Test with user_id filter
        let found_comment = repository
            .find(
                comment.id,
                CommentFilter {
                    user_id: Some(user_id),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(found_comment.user_id, Some(user_id));
    }
}
