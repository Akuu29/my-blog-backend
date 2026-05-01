use async_trait::async_trait;
use blog_app::query_service::{
    articles_by_tag::i_articles_by_tag_query_service::{
        ArticlesByTagFilter, IArticlesByTagQueryService,
    },
    error::QueryServiceError,
};
use blog_domain::model::{
    articles::article::Article,
    common::{item_count::ItemCount, pagination::Pagination},
};
use sqlx::query_builder::QueryBuilder;

#[derive(Debug, Clone)]
pub struct ArticlesByTagQueryService {
    pool: sqlx::PgPool,
}

impl ArticlesByTagQueryService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// push article where conditions to the query builder
    fn push_article_condition(
        &self,
        qb: &mut QueryBuilder<'_, sqlx::Postgres>,
        filter: &ArticlesByTagFilter,
    ) {
        if let Some(user_id) = filter.user_id {
            qb.push(" AND a.user_id = ").push_bind(user_id);
        }
        if let Some(article_status) = filter.article_status {
            qb.push(" AND a.status = ").push_bind(article_status);
        }
    }
}

#[async_trait]
impl IArticlesByTagQueryService for ArticlesByTagQueryService {
    async fn find_article_title_by_tag(
        &self,
        filter: ArticlesByTagFilter,
        pagination: Pagination,
    ) -> Result<(Vec<Article>, ItemCount), QueryServiceError> {
        // find articles
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                a.id,
                a.user_id,
                a.title,
                a.body,
                a.status,
                a.category_id,
                a.created_at,
                a.updated_at
            FROM articles AS a
            WHERE EXISTS (
                SELECT 1
                FROM tagged_articles AS ta
                WHERE ta.article_id = a.id
                AND ta.tag_id = ANY(
            "#,
        );
        qb.push_bind(&filter.tag_ids);
        qb.push(") )");

        // build conditions
        self.push_article_condition(&mut qb, &filter);

        /*
        build paginated conditions.
        cursor, offset can only be used once,
        because each is validated to prevent conflicts.
        */
        if let Some(cursor) = pagination.cursor {
            let cursor_ts = sqlx::query_scalar::<
                _,
                sqlx::types::chrono::DateTime<sqlx::types::chrono::Local>,
            >("SELECT created_at FROM articles WHERE id = $1")
            .bind(cursor)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| QueryServiceError::Unknown(Box::new(e)))?;

            let cursor_ts = cursor_ts.ok_or(QueryServiceError::InvalidCursor)?;
            qb.push(" AND (a.created_at, a.id) < (")
                .push_bind(cursor_ts)
                .push(", ")
                .push_bind(cursor)
                .push(")");
        }

        qb.push(" ORDER BY a.created_at DESC, a.id DESC");

        if let Some(offset) = pagination.offset {
            qb.push(" OFFSET ").push_bind(offset);
        }

        qb.push(" LIMIT ").push_bind(pagination.per_page);

        let articles = qb
            .build_query_as::<Article>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| QueryServiceError::Unknown(Box::new(e)))?;

        // count total articles
        let mut qb = QueryBuilder::new(
            r#"
            SELECT COUNT(*)
            FROM articles AS a
            WHERE EXISTS (
                SELECT 1
                FROM tagged_articles AS ta
                WHERE ta.article_id = a.id
                AND ta.tag_id = ANY(
            "#,
        );
        qb.push_bind(&filter.tag_ids);
        qb.push(") )");

        // build conditions
        self.push_article_condition(&mut qb, &filter);

        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| QueryServiceError::Unknown(Box::new(e)))?;

        Ok((articles, total))
    }
}

#[cfg(test)]
#[cfg(feature = "database-test")]
mod test {
    use super::*;
    use blog_domain::model::{articles::article::ArticleStatus, common::pagination::Pagination};
    use dotenv::dotenv;
    use serial_test::serial;
    use sqlx::PgPool;
    use uuid::Uuid;

    // Test helper functions
    async fn setup() -> (PgPool, ArticlesByTagQueryService, Uuid) {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let query_service = ArticlesByTagQueryService::new(pool.clone());

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
    async fn test_find_articles_by_single_tag() {
        let (pool, query_service, user_uuid) = setup().await;
        let mut guard = TestDataGuard::new(&pool);

        // Create a tag
        let tag_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(format!(
            "tag-{}",
            Uuid::new_v4().to_string()[0..8].to_string()
        ))
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .expect("failed to create tag");
        guard.track_tag(tag_id);

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

        // Attach tag to article
        sqlx::query("INSERT INTO tagged_articles (article_id, tag_id) VALUES ($1, $2)")
            .bind(article_id)
            .bind(tag_id)
            .execute(&pool)
            .await
            .expect("failed to attach tag to article");

        // Test: Find articles by this tag
        let filter = ArticlesByTagFilter {
            tag_ids: vec![tag_id],
            user_id: None,
            article_status: None,
        };
        let pagination = Pagination {
            per_page: 10,
            cursor: None,
            offset: None,
        };

        let (articles, _) = query_service
            .find_article_title_by_tag(filter, pagination)
            .await
            .unwrap();

        assert!(articles.iter().any(|a| a.id == article_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_articles_by_multiple_tags() {
        let (pool, query_service, user_uuid) = setup().await;
        let mut guard = TestDataGuard::new(&pool);

        // Create two tags
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

        // Create article with both tags
        let article_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO articles (title, body, status, user_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(format!("Article {}", Uuid::new_v4()))
        .bind("Test Body")
        .bind(ArticleStatus::Draft)
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_article(article_id);

        // Attach both tags
        for tag_id in &[tag1_id, tag2_id] {
            sqlx::query("INSERT INTO tagged_articles (article_id, tag_id) VALUES ($1, $2)")
                .bind(article_id)
                .bind(tag_id)
                .execute(&pool)
                .await
                .unwrap();
        }

        // Test: Find articles by both tags
        let filter = ArticlesByTagFilter {
            tag_ids: vec![tag1_id, tag2_id],
            user_id: None,
            article_status: None,
        };
        let pagination = Pagination {
            per_page: 10,
            cursor: None,
            offset: None,
        };

        let (articles, _) = query_service
            .find_article_title_by_tag(filter, pagination)
            .await
            .unwrap();

        assert!(articles.iter().any(|a| a.id == article_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_articles_by_tag_with_user_filter() {
        let (pool, query_service, user_uuid) = setup().await;
        let mut guard = TestDataGuard::new(&pool);

        // Create tag
        let tag_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(format!(
            "tag-{}",
            Uuid::new_v4().to_string()[0..8].to_string()
        ))
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_tag(tag_id);

        // Create article by the user
        let article_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO articles (title, body, status, user_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(format!("Article {}", Uuid::new_v4()))
        .bind("Test Body")
        .bind(ArticleStatus::Draft)
        .bind(user_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_article(article_id);

        // Attach tag
        sqlx::query("INSERT INTO tagged_articles (article_id, tag_id) VALUES ($1, $2)")
            .bind(article_id)
            .bind(tag_id)
            .execute(&pool)
            .await
            .unwrap();

        // Test: Find articles by tag AND user
        let filter = ArticlesByTagFilter {
            tag_ids: vec![tag_id],
            user_id: Some(user_uuid),
            article_status: None,
        };
        let pagination = Pagination {
            per_page: 10,
            cursor: None,
            offset: None,
        };

        let (articles, _) = query_service
            .find_article_title_by_tag(filter, pagination)
            .await
            .unwrap();

        assert!(articles.iter().any(|a| a.id == article_id));
    }
}
