use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_app::query_service::articles_by_tag::i_articles_by_tag_query_service::{
    ArticlesByTagFilter, IArticlesByTagQueryService,
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
        if let Some(user_public_id) = filter.user_public_id {
            qb.push(" AND u.public_id = ").push_bind(user_public_id);
        }
    }
}

#[async_trait]
impl IArticlesByTagQueryService for ArticlesByTagQueryService {
    async fn find_article_title_by_tag(
        &self,
        filter: ArticlesByTagFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Article>, ItemCount)> {
        // find articles
        let mut qb = QueryBuilder::new(
            r#"
            WITH tag_ids AS (
                SELECT id
                FROM tags
                WHERE public_id = ANY(
            "#,
        );
        qb.push_bind(&filter.tag_ids);
        qb.push(") ");

        qb.push(
            r#"
            )
            SELECT
                a.public_id,
                u.public_id as user_public_id,
                a.title,
                a.body,
                status,
                (SELECT public_id FROM categories WHERE id = category_id) as category_public_id,
                a.created_at,
                a.updated_at
            FROM articles AS a
            LEFT JOIN users AS u ON a.user_id = u.id
            WHERE EXISTS (
                SELECT 1
                FROM article_tags AS at
                WHERE at.article_id = a.id
                AND at.tag_id IN (SELECT id FROM tag_ids)
            )
            "#,
        );

        // build conditions
        self.push_article_condition(&mut qb, &filter);

        /*
        build paginated conditions.
        cursor, offset can only be used once,
        because each is validated to prevent conflicts.
        */
        if let Some(cursor) = pagination.cursor {
            // get the id of the article with the given public_id
            let cid_option = sqlx::query_scalar!(
                r#"
                SELECT id FROM articles WHERE public_id = $1
                "#,
                cursor
            )
            .fetch_optional(&self.pool)
            .await?;

            let cid = cid_option.ok_or(RepositoryError::NotFound)?;
            qb.push(" AND a.id < ").push_bind(cid);
        }

        qb.push(" ORDER BY a.id DESC");

        if let Some(offset) = pagination.offset {
            qb.push(" OFFSET ").push_bind(offset);
        }

        qb.push(" LIMIT ").push_bind(pagination.per_page);

        let articles = qb.build_query_as::<Article>().fetch_all(&self.pool).await?;

        // count total articles
        let mut qb = QueryBuilder::new(
            r#"
            WITH tag_ids AS (
                SELECT id
                FROM tags
                WHERE public_id = ANY(
            "#,
        );
        qb.push_bind(&filter.tag_ids);
        qb.push(") ");

        qb.push(
            r#"
            )
            SELECT COUNT(*)
            FROM articles AS a
            LEFT JOIN users AS u ON a.user_id = u.id
            WHERE EXISTS (
                SELECT 1
                FROM article_tags AS at
                WHERE at.article_id = a.id
                AND at.tag_id IN (SELECT id FROM tag_ids)
            )
            "#,
        );

        // build conditions
        self.push_article_condition(&mut qb, &filter);

        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await?;

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

        // Get test user public_id (UUID)
        let user_public_id = std::env::var("TEST_USER_ID").expect("undefined TEST_USER_ID");
        let user_uuid = uuid::Uuid::parse_str(&user_public_id).expect("invalid TEST_USER_ID UUID");

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
                        // Cleanup article_tags (junction table) first
                        for article_id in &article_ids {
                            let _ = sqlx::query(
                                "DELETE FROM article_tags WHERE article_id = (SELECT id FROM articles WHERE public_id = $1)"
                            )
                            .bind(article_id)
                            .execute(&pool)
                            .await;
                        }

                        // Then cleanup articles
                        for article_id in &article_ids {
                            let _ = sqlx::query("DELETE FROM articles WHERE public_id = $1")
                                .bind(article_id)
                                .execute(&pool)
                                .await;
                        }

                        // Finally cleanup tags
                        for tag_id in &tag_ids {
                            let _ = sqlx::query("DELETE FROM tags WHERE public_id = $1")
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

        // Get internal user_id
        let user_id = sqlx::query_scalar::<_, i32>("SELECT id FROM users WHERE public_id = $1")
            .bind(user_uuid)
            .fetch_one(&pool)
            .await
            .expect("failed to get user_id");

        // Create a tag
        let tag_public_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, $2) RETURNING public_id",
        )
        .bind(format!(
            "tag-{}",
            Uuid::new_v4().to_string()[0..8].to_string()
        ))
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("failed to create tag");
        guard.track_tag(tag_public_id);

        // Create article
        let article_public_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO articles (title, body, status, user_id) VALUES ($1, $2, $3, $4) RETURNING public_id",
        )
        .bind(format!("Article {}", Uuid::new_v4()))
        .bind("Test Body")
        .bind(ArticleStatus::Draft)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("failed to create article");
        guard.track_article(article_public_id);

        // Attach tag to article
        sqlx::query(
            r#"
            INSERT INTO article_tags (article_id, tag_id)
            VALUES (
                (SELECT id FROM articles WHERE public_id = $1),
                (SELECT id FROM tags WHERE public_id = $2)
            )
            "#,
        )
        .bind(article_public_id)
        .bind(tag_public_id)
        .execute(&pool)
        .await
        .expect("failed to attach tag to article");

        // Test: Find articles by this tag
        let filter = ArticlesByTagFilter {
            tag_ids: vec![tag_public_id],
            user_public_id: None,
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

        assert!(articles.iter().any(|a| a.public_id == article_public_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_articles_by_multiple_tags() {
        let (pool, query_service, user_uuid) = setup().await;
        let mut guard = TestDataGuard::new(&pool);

        let user_id = sqlx::query_scalar::<_, i32>("SELECT id FROM users WHERE public_id = $1")
            .bind(user_uuid)
            .fetch_one(&pool)
            .await
            .expect("failed to get user_id");

        // Create two tags
        let tag1_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, $2) RETURNING public_id",
        )
        .bind(format!(
            "tag1-{}",
            Uuid::new_v4().to_string()[0..8].to_string()
        ))
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_tag(tag1_id);

        let tag2_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, $2) RETURNING public_id",
        )
        .bind(format!(
            "tag2-{}",
            Uuid::new_v4().to_string()[0..8].to_string()
        ))
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_tag(tag2_id);

        // Create article with both tags
        let article_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO articles (title, body, status, user_id) VALUES ($1, $2, $3, $4) RETURNING public_id",
        )
        .bind(format!("Article {}", Uuid::new_v4()))
        .bind("Test Body")
        .bind(ArticleStatus::Draft)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_article(article_id);

        // Attach both tags
        for tag_id in &[tag1_id, tag2_id] {
            sqlx::query(
                "INSERT INTO article_tags (article_id, tag_id) VALUES ((SELECT id FROM articles WHERE public_id = $1), (SELECT id FROM tags WHERE public_id = $2))",
            )
            .bind(article_id)
            .bind(tag_id)
            .execute(&pool)
            .await
            .unwrap();
        }

        // Test: Find articles by both tags
        let filter = ArticlesByTagFilter {
            tag_ids: vec![tag1_id, tag2_id],
            user_public_id: None,
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

        assert!(articles.iter().any(|a| a.public_id == article_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_articles_by_tag_with_user_filter() {
        let (pool, query_service, user_uuid) = setup().await;
        let mut guard = TestDataGuard::new(&pool);

        let user_id = sqlx::query_scalar::<_, i32>("SELECT id FROM users WHERE public_id = $1")
            .bind(user_uuid)
            .fetch_one(&pool)
            .await
            .unwrap();

        // Create tag
        let tag_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, $2) RETURNING public_id",
        )
        .bind(format!(
            "tag-{}",
            Uuid::new_v4().to_string()[0..8].to_string()
        ))
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_tag(tag_id);

        // Create article by the user
        let article_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO articles (title, body, status, user_id) VALUES ($1, $2, $3, $4) RETURNING public_id",
        )
        .bind(format!("Article {}", Uuid::new_v4()))
        .bind("Test Body")
        .bind(ArticleStatus::Draft)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_article(article_id);

        // Attach tag
        sqlx::query(
            "INSERT INTO article_tags (article_id, tag_id) VALUES ((SELECT id FROM articles WHERE public_id = $1), (SELECT id FROM tags WHERE public_id = $2))",
        )
        .bind(article_id)
        .bind(tag_id)
        .execute(&pool)
        .await
        .unwrap();

        // Test: Find articles by tag AND user
        let filter = ArticlesByTagFilter {
            tag_ids: vec![tag_id],
            user_public_id: Some(user_uuid),
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

        assert!(articles.iter().any(|a| a.public_id == article_id));
    }
}
