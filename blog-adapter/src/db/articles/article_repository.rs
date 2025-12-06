use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::{
    articles::{
        article::{Article, NewArticle, UpdateArticle},
        i_article_repository::{ArticleFilter, IArticleRepository},
    },
    common::{item_count::ItemCount, pagination::Pagination},
};
use sqlx::QueryBuilder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ArticleRepository {
    pool: sqlx::PgPool,
}

impl ArticleRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// push article where conditions to the query builder
    fn push_article_condition(
        &self,
        qb: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
        filter: &ArticleFilter,
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

        if let Some(user_id) = filter.user_id {
            push_condition(qb);
            qb.push("u.public_id = ").push_bind(user_id);
        }

        if let Some(status) = filter.status {
            push_condition(qb);
            qb.push("a.status = ").push_bind(status);
        }

        if let Some(category_public_id) = filter.category_public_id {
            push_condition(qb);
            qb.push("c.public_id = ").push_bind(category_public_id);
        }

        if let Some(title_contains) = filter.title_contains.clone() {
            push_condition(qb);
            qb.push("a.title ILIKE ")
                .push_bind(format!("%{}%", title_contains));
        }

        return has_condition;
    }
}

#[async_trait]
impl IArticleRepository for ArticleRepository {
    async fn create(&self, user_id: Uuid, new_article: NewArticle) -> anyhow::Result<Article> {
        let article = sqlx::query_as::<_, Article>(
            r#"
            WITH category AS (
                    SELECT id FROM categories WHERE public_id = $4
                ),
                usr AS (
                    SELECT id FROM users WHERE public_id = $5
                )
            INSERT INTO articles (
                title,
                body,
                status,
                category_id,
                user_id
            )
            SELECT
                $1,
                $2,
                $3,
                category.id,
                usr.id
            FROM category
            RIGHT JOIN usr ON TRUE
            RETURNING
                public_id,
                $5 AS user_public_id,
                title,
                body,
                status,
                $4 AS category_public_id,
                created_at,
                updated_at
            "#,
        )
        .bind(new_article.title)
        .bind(new_article.body)
        .bind(new_article.status)
        .bind(new_article.category_public_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(article)
    }

    async fn find(
        &self,
        article_id: Uuid,
        article_filter: ArticleFilter,
    ) -> anyhow::Result<Article> {
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                a.public_id,
                u.public_id as user_public_id,
                a.title,
                a.body,
                a.status,
                c.public_id as category_public_id,
                a.created_at,
                a.updated_at
            FROM articles AS a
            LEFT JOIN categories AS c
            ON a.category_id = c.id
            LEFT JOIN users AS u
            ON a.user_id = u.id
            WHERE a.public_id =
            "#,
        );

        qb.push_bind(article_id);

        if let Some(user_id) = article_filter.user_id {
            qb.push(" AND u.public_id = ").push_bind(user_id);
        }

        if let Some(status) = article_filter.status {
            qb.push(" AND a.status = ").push_bind(status);
        }

        let article = qb
            .build_query_as::<Article>()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => RepositoryError::NotFound,
                e => RepositoryError::Unexpected(e.to_string()),
            })?;

        Ok(article)
    }

    async fn all(
        &self,
        article_filter: ArticleFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Article>, ItemCount)> {
        // find articles
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                a.public_id,
                u.public_id AS user_public_id,
                a.title,
                a.body,
                a.status,
                c.public_id AS category_public_id,
                a.created_at,
                a.updated_at
            FROM articles AS a
            LEFT JOIN categories AS c ON a.category_id = c.id
            LEFT JOIN users AS u ON a.user_id = u.id
            "#,
        );

        // build conditions
        let has_condition = self.push_article_condition(&mut qb, &article_filter);

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
            if has_condition {
                qb.push(" AND ");
            } else {
                qb.push(" WHERE ");
            }
            qb.push("a.id < ").push_bind(cid);
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
            SELECT
                COUNT(*)
            FROM articles AS a
            LEFT JOIN categories AS c ON a.category_id = c.id
            LEFT JOIN users AS u ON a.user_id = u.id
            "#,
        );
        // build conditions
        self.push_article_condition(&mut qb, &article_filter);

        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await?;

        Ok((articles, total))
    }

    async fn update(
        &self,
        article_id: Uuid,
        update_article: UpdateArticle,
    ) -> anyhow::Result<Article> {
        let pre_payload = self.find(article_id, ArticleFilter::default()).await?;
        let article = sqlx::query_as::<_, Article>(
            r#"
            UPDATE articles set
                title = $1,
                body = $2,
                status = $3,
                category_id = (SELECT id FROM categories WHERE public_id = $4),
                updated_at = now()
            WHERE public_id = $5
            RETURNING
                public_id,
                $6 AS user_public_id,
                title,
                body,
                status,
                $4 AS category_public_id,
                created_at,
                updated_at
            ;
            "#,
        )
        .bind(
            update_article
                .title
                .unwrap_or(pre_payload.title.unwrap_or_default()),
        )
        .bind(
            update_article
                .body
                .unwrap_or(pre_payload.body.unwrap_or_default()),
        )
        .bind(update_article.status.unwrap_or(pre_payload.status))
        .bind(update_article.category_public_id)
        .bind(article_id)
        .bind(pre_payload.user_public_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(article)
    }

    async fn delete(&self, article_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM articles
            WHERE public_id = $1
            ;
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

    async fn attach_tags(&self, article_id: Uuid, tag_ids: Vec<Uuid>) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // delete all tags for the article
        sqlx::query(
            r#"
            DELETE FROM article_tags
            WHERE article_id = (
                SELECT id FROM articles
                WHERE public_id = $1
            );
            "#,
        )
        .bind(article_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        // buck insert new tags
        sqlx::query(
            r#"
            WITH target_article AS (
                    SELECT id FROM articles
                    WHERE public_id = $1
                ),
                target_tags AS (
                    SELECT id FROM tags
                    WHERE public_id = ANY($2)
                )
            INSERT INTO article_tags (article_id, tag_id)
            SELECT target_article.id, u.tag_id
            FROM target_article
            CROSS JOIN UNNEST(ARRAY(SELECT id FROM target_tags)) AS u(tag_id)
            ;
            "#,
        )
        .bind(article_id)
        .bind(tag_ids)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "database-test")]
mod test {
    use super::*;
    use blog_domain::model::{
        articles::{article::ArticleStatus, i_article_repository::ArticleFilter},
        common::pagination::Pagination,
    };
    use dotenv::dotenv;
    use serial_test::serial;
    use sqlx::{PgPool, types::Uuid};

    // Test helper functions
    async fn setup() -> (PgPool, ArticleRepository, Uuid) {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let repository = ArticleRepository::new(pool.clone());
        let user_id =
            Uuid::parse_str(&std::env::var("TEST_USER_ID").expect("undefined TEST_USER_ID"))
                .expect("invalid TEST_USER_ID");
        (pool, repository, user_id)
    }

    async fn create_test_article(
        repository: &ArticleRepository,
        user_id: Uuid,
        title: &str,
        body: &str,
        status: ArticleStatus,
    ) -> Article {
        let payload = NewArticle {
            title: Some(title.to_string()),
            body: Some(body.to_string()),
            status,
            category_public_id: None,
        };
        repository.create(user_id, payload).await.unwrap()
    }

    struct TestArticleGuard {
        article_repository: ArticleRepository,
        article_ids: Vec<Uuid>,
        tag_ids: Vec<Uuid>,
        runtime_handle: tokio::runtime::Handle,
        pool: Option<PgPool>,
    }

    impl TestArticleGuard {
        fn new(article_repository: &ArticleRepository, pool: Option<&PgPool>) -> Self {
            Self {
                article_repository: article_repository.clone(),
                article_ids: Vec::new(),
                tag_ids: Vec::new(),
                runtime_handle: tokio::runtime::Handle::current(),
                pool: pool.cloned(),
            }
        }

        fn track_article(&mut self, article_id: Uuid) {
            self.article_ids.push(article_id);
        }

        fn track_tag(&mut self, tag_id: Uuid) {
            self.tag_ids.push(tag_id);
        }
    }

    impl Drop for TestArticleGuard {
        fn drop(&mut self) {
            // Clone the data needed for cleanup
            let article_repository = self.article_repository.clone();
            let article_ids = self.article_ids.clone();
            let tag_ids = self.tag_ids.clone();
            let handle = self.runtime_handle.clone();
            let pool = self.pool.clone();

            // Use block_in_place to allow blocking in async context
            // This tells the runtime: "I'm about to block, please move other tasks to different threads"
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::task::block_in_place(|| {
                    handle.block_on(async move {
                        // Cleanup test articles
                        for article_id in &article_ids {
                            let _ = article_repository.delete(*article_id).await;
                        }

                        // Cleanup test tags
                        if let Some(pool) = pool {
                            for tag_id in &tag_ids {
                                let _ = sqlx::query(
                                    r#"
                                    DELETE FROM tags
                                    WHERE public_id = $1
                                    ;
                                    "#,
                                )
                                .bind(tag_id)
                                .execute(&pool)
                                .await;
                            }
                        }
                    });
                });
            }));
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_create_article() {
        let (_, repository, user_id) = setup().await;
        let mut guard = TestArticleGuard::new(&repository, None);

        let unique_title = format!("test title {}", Uuid::new_v4());
        let payload = NewArticle {
            title: Some(unique_title.clone()),
            body: Some("test body".to_string()),
            status: ArticleStatus::Draft,
            category_public_id: None,
        };

        let article = repository.create(user_id, payload.clone()).await.unwrap();
        guard.track_article(article.public_id);

        assert_eq!(article.title, payload.title);
        assert_eq!(article.body, payload.body);
        assert_eq!(article.status, payload.status);
        assert_eq!(article.user_public_id, user_id);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_article() {
        let (_, repository, user_id) = setup().await;
        let mut guard = TestArticleGuard::new(&repository, None);

        let unique_title = format!("test title {}", Uuid::new_v4());
        let article = create_test_article(
            &repository,
            user_id,
            &unique_title,
            "test find body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article.public_id);

        let found_article = repository
            .find(article.public_id, ArticleFilter::default())
            .await
            .unwrap();

        assert_eq!(found_article.public_id, article.public_id);
        assert_eq!(found_article.title, article.title);
        assert_eq!(found_article.body, article.body);
        assert_eq!(found_article.status, article.status);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_article_with_user_filter() {
        let (_, repository, user_id) = setup().await;
        let mut guard = TestArticleGuard::new(&repository, None);

        let unique_title = format!("test title {}", Uuid::new_v4());
        let article = create_test_article(
            &repository,
            user_id,
            &unique_title,
            "test filter body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article.public_id);

        let found_article = repository
            .find(
                article.public_id,
                ArticleFilter {
                    user_id: Some(user_id),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(found_article.user_public_id, user_id);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_articles_with_pagination() {
        let (_, repository, user_id) = setup().await;
        let mut guard = TestArticleGuard::new(&repository, None);

        // Setup: create multiple test articles with unique titles
        let uuid_prefix = Uuid::new_v4();
        let article1 = create_test_article(
            &repository,
            user_id,
            &format!("first article {}", uuid_prefix),
            "first body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article1.public_id);

        let article2 = create_test_article(
            &repository,
            user_id,
            &format!("second article {}", uuid_prefix),
            "second body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article2.public_id);

        let article3 = create_test_article(
            &repository,
            user_id,
            &format!("third article {}", uuid_prefix),
            "third body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article3.public_id);

        // Test: fetch with pagination (per_page = 2)
        let (articles, total) = repository
            .all(
                ArticleFilter {
                    user_id: Some(user_id),
                    ..Default::default()
                },
                Pagination {
                    per_page: 2,
                    ..Pagination::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(articles.len(), 2);
        assert!(total.value() >= 3);
        // Articles should be in DESC order (newest first)
        assert_eq!(articles[0].public_id, article3.public_id);
        assert_eq!(articles[1].public_id, article2.public_id);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_articles_with_cursor() {
        let (_, repository, user_id) = setup().await;
        let mut guard = TestArticleGuard::new(&repository, None);

        // Setup: create multiple test articles
        let unique_title1 = format!("test title1 {}", Uuid::new_v4());
        let article1 = create_test_article(
            &repository,
            user_id,
            &unique_title1,
            "cursor first body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article1.public_id);

        let unique_title2 = format!("test title2 {}", Uuid::new_v4());
        let article2 = create_test_article(
            &repository,
            user_id,
            &unique_title2,
            "cursor second body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article2.public_id);

        let unique_title3 = format!("test title3 {}", Uuid::new_v4());
        let article3 = create_test_article(
            &repository,
            user_id,
            &unique_title3,
            "cursor third body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article3.public_id);

        // Test: fetch with cursor pagination
        let (articles, _) = repository
            .all(
                ArticleFilter {
                    user_id: Some(user_id),
                    ..Default::default()
                },
                Pagination {
                    cursor: Some(article3.public_id),
                    per_page: 2,
                    ..Pagination::default()
                },
            )
            .await
            .unwrap();

        // Should get articles before the cursor (article2 and article1)
        assert_eq!(articles.len(), 2);
        assert_eq!(articles[0].public_id, article2.public_id);
        assert_eq!(articles[1].public_id, article1.public_id);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_update_article() {
        let (_, repository, user_id) = setup().await;
        let mut guard = TestArticleGuard::new(&repository, None);

        // Setup: create a test article
        let unique_title = format!("test title {}", Uuid::new_v4());
        let article = create_test_article(
            &repository,
            user_id,
            &unique_title,
            "original body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article.public_id);

        // Test: update the article
        let update_unique_title = format!("updated title {}", Uuid::new_v4());
        let update_payload = UpdateArticle {
            title: Some(update_unique_title.to_string()),
            body: Some("updated body".to_string()),
            status: Some(ArticleStatus::Published),
            category_public_id: None,
        };

        let updated_article = repository
            .update(article.public_id, update_payload)
            .await
            .unwrap();

        assert_eq!(updated_article.title, Some(update_unique_title.to_string()));
        assert_eq!(updated_article.body, Some("updated body".to_string()));
        assert_eq!(updated_article.status, ArticleStatus::Published);
        assert_eq!(updated_article.public_id, article.public_id);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_delete_article() {
        let (_, repository, user_id) = setup().await;

        // Setup: create a test article
        let article = create_test_article(
            &repository,
            user_id,
            "to delete",
            "to delete body",
            ArticleStatus::Draft,
        )
        .await;

        repository.delete(article.public_id).await.unwrap();

        // Verify deletion
        let result = repository
            .find(article.public_id, ArticleFilter::default())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_attach_tags() {
        let (pool, repository, user_id) = setup().await;
        let mut guard = TestArticleGuard::new(&repository, Some(&pool));

        // Setup: create a test article
        let article = create_test_article(
            &repository,
            user_id,
            "article with tags",
            "article with tags body",
            ArticleStatus::Draft,
        )
        .await;
        guard.track_article(article.public_id);

        // Setup: create test tags
        let tag1_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, (SELECT id FROM users WHERE public_id = $2)) RETURNING public_id",
        )
        .bind("tag1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_tag(tag1_id);

        let tag2_id = sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO tags (name, user_id) VALUES ($1, (SELECT id FROM users WHERE public_id = $2)) RETURNING public_id",
        )
        .bind("tag2")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        guard.track_tag(tag2_id);

        // Test: attach tags to article
        repository
            .attach_tags(article.public_id, vec![tag1_id, tag2_id])
            .await
            .unwrap();

        // Verify: check tags are attached
        let attached_tags = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT t.public_id
            FROM article_tags at
            JOIN tags t ON at.tag_id = t.id
            JOIN articles a ON at.article_id = a.id
            WHERE a.public_id = $1
            ORDER BY t.public_id
            "#,
        )
        .bind(article.public_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(attached_tags.len(), 2);
        assert!(attached_tags.contains(&tag1_id));
        assert!(attached_tags.contains(&tag2_id));

        // Test: re-attach with different tags (should replace)
        repository
            .attach_tags(article.public_id, vec![tag1_id])
            .await
            .unwrap();

        let attached_tags_after = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT t.public_id
            FROM article_tags at
            JOIN tags t ON at.tag_id = t.id
            JOIN articles a ON at.article_id = a.id
            WHERE a.public_id = $1
            "#,
        )
        .bind(article.public_id)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(attached_tags_after.len(), 1);
        assert_eq!(attached_tags_after[0], tag1_id);
    }
}
