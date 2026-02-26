use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::{
    common::{item_count::ItemCount, pagination::Pagination},
    tags::{
        i_tag_repository::{ITagRepository, TagFilter},
        tag::{NewTag, Tag},
    },
};
use sqlx::QueryBuilder;
use uuid::Uuid;

#[derive(Clone)]
pub struct TagRepository {
    pool: sqlx::PgPool,
}

impl TagRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// push tag where conditions to the query builder
    fn push_tag_condition(
        &self,
        qb: &mut QueryBuilder<'_, sqlx::Postgres>,
        filter: &TagFilter,
    ) -> bool {
        let mut has_condition = false;
        let mut push_condition = |qb: &mut QueryBuilder<'_, sqlx::Postgres>| {
            if !has_condition {
                qb.push(" WHERE ");
                has_condition = true;
            } else {
                qb.push(" AND ");
            }
        };

        if let Some(user_id) = filter.user_id {
            push_condition(qb);
            qb.push("t.user_id = ").push_bind(user_id);
        }

        if let Some(tag_ids) = filter.tag_ids.clone() {
            push_condition(qb);
            qb.push("t.id = ANY(").push_bind(tag_ids).push(")");
        }

        return has_condition;
    }
}

#[async_trait]
impl ITagRepository for TagRepository {
    async fn create(&self, user_id: Uuid, payload: NewTag) -> anyhow::Result<Tag> {
        let tag = sqlx::query_as::<_, Tag>(
            r#"
            INSERT INTO tags (
                name,
                user_id
            )
            VALUES ($1, $2)
            RETURNING
                id,
                user_id,
                name,
                created_at,
                updated_at
            ;
            "#,
        )
        .bind(payload.name)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(tag)
    }

    async fn find(&self, tag_id: Uuid) -> anyhow::Result<Tag> {
        let tag = sqlx::query_as::<_, Tag>(
            r#"
            SELECT
                t.id,
                t.user_id,
                t.name,
                t.created_at,
                t.updated_at
            FROM tags AS t
            WHERE t.id = $1
            ;
            "#,
        )
        .bind(tag_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound.into(),
            e => anyhow::anyhow!(e),
        })?;

        Ok(tag)
    }

    async fn all(
        &self,
        tag_filter: TagFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Tag>, ItemCount)> {
        // find tags
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                t.id,
                t.user_id,
                t.name,
                t.created_at,
                t.updated_at
            FROM tags AS t
            "#,
        );

        // build conditions
        let has_condition = self.push_tag_condition(&mut qb, &tag_filter);

        /*
        build paginated conditions.
        cursor, offset can only be used once,
        because each is validated to prevent conflicts.
        */
        if let Some(cursor) = pagination.cursor {
            let cursor_ts_option = sqlx::query_scalar::<
                _,
                sqlx::types::chrono::DateTime<sqlx::types::chrono::Local>,
            >("SELECT created_at FROM tags WHERE id = $1")
            .bind(cursor)
            .fetch_optional(&self.pool)
            .await?;

            let cursor_ts = cursor_ts_option.ok_or(RepositoryError::NotFound)?;
            if has_condition {
                qb.push(" AND ");
            } else {
                qb.push(" WHERE ");
            }
            qb.push("(t.created_at, t.id) < (")
                .push_bind(cursor_ts)
                .push(", ")
                .push_bind(cursor)
                .push(")");
        }

        qb.push(" ORDER BY t.created_at DESC, t.id DESC");

        if let Some(offset) = pagination.offset {
            qb.push(" OFFSET ").push_bind(offset);
        }

        qb.push(" LIMIT ").push_bind(pagination.per_page);

        let tags = qb.build_query_as::<Tag>().fetch_all(&self.pool).await?;

        // count total tags
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                COUNT(*)
            FROM tags AS t
            "#,
        );
        // build conditions
        self.push_tag_condition(&mut qb, &tag_filter);

        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await?;

        Ok((tags, total))
    }

    async fn delete(&self, tag_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM tags
            WHERE id = $1
            ;
            "#,
        )
        .bind(tag_id)
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
    use blog_domain::model::common::pagination::Pagination;
    use dotenv::dotenv;
    use serial_test::serial;
    use sqlx::PgPool;

    // Test helper functions
    async fn setup() -> (PgPool, TagRepository, Uuid) {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let repository = TagRepository::new(pool.clone());

        // Get test user id (UUID)
        let user_id = std::env::var("TEST_USER_ID").expect("undefined TEST_USER_ID");
        let user_uuid = uuid::Uuid::parse_str(&user_id).expect("invalid TEST_USER_ID UUID");

        // Ensure the test user exists (required by tags.user_id foreign key)
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

        (pool, repository, user_uuid)
    }

    async fn create_test_tag(repository: &TagRepository, user_uuid: Uuid, name: &str) -> Tag {
        let payload = NewTag {
            name: name.to_string(),
        };
        repository.create(user_uuid, payload).await.unwrap()
    }

    struct TestTagGuard {
        repository: TagRepository,
        tag_ids: Vec<Uuid>,
        runtime_handle: tokio::runtime::Handle,
    }

    impl TestTagGuard {
        fn new(repository: &TagRepository) -> Self {
            Self {
                repository: repository.clone(),
                tag_ids: Vec::new(),
                runtime_handle: tokio::runtime::Handle::current(),
            }
        }

        fn track(&mut self, tag_id: Uuid) {
            self.tag_ids.push(tag_id);
        }
    }

    impl Drop for TestTagGuard {
        fn drop(&mut self) {
            let repository = self.repository.clone();
            let tag_ids = self.tag_ids.clone();
            let handle = self.runtime_handle.clone();

            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::task::block_in_place(|| {
                    handle.block_on(async move {
                        // Cleanup test tags
                        for tag_id in &tag_ids {
                            let _ = repository.delete(*tag_id).await;
                        }
                    });
                });
            }));
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_create_tag() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestTagGuard::new(&repository);

        // Tag name is limited to 15 characters
        let unique_name = format!("tag-{}", Uuid::new_v4().to_string()[0..8].to_string());
        let payload = NewTag {
            name: unique_name.clone(),
        };

        let tag = repository.create(user_uuid, payload).await.unwrap();
        guard.track(tag.id);

        assert_eq!(tag.name, unique_name);
        assert_eq!(tag.user_id, user_uuid);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_tags() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestTagGuard::new(&repository);

        let tag1 = create_test_tag(
            &repository,
            user_uuid,
            &format!("tag1-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(tag1.id);

        let tag2 = create_test_tag(
            &repository,
            user_uuid,
            &format!("tag2-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(tag2.id);

        let filter = TagFilter {
            user_id: Some(user_uuid),
            tag_ids: None,
        };
        let pagination = Pagination {
            per_page: 10,
            cursor: None,
            offset: None,
        };

        let (tags, _) = repository.all(filter, pagination).await.unwrap();

        assert!(tags.iter().any(|t| t.id == tag1.id));
        assert!(tags.iter().any(|t| t.id == tag2.id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_tags_with_cursor() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestTagGuard::new(&repository);

        let tag1 = create_test_tag(
            &repository,
            user_uuid,
            &format!("cur1-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(tag1.id);

        let tag2 = create_test_tag(
            &repository,
            user_uuid,
            &format!("cur2-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(tag2.id);

        let tag3 = create_test_tag(
            &repository,
            user_uuid,
            &format!("cur3-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(tag3.id);

        let filter = TagFilter {
            user_id: Some(user_uuid),
            tag_ids: None,
        };

        // Get first page
        let pagination = Pagination {
            per_page: 2,
            cursor: None,
            offset: None,
        };
        let (first_page, _) = repository.all(filter, pagination).await.unwrap();
        assert!(first_page.len() >= 2);

        // Get second page using cursor
        let cursor_id = first_page[1].id;
        let filter_with_cursor = TagFilter {
            user_id: Some(user_uuid),
            tag_ids: None,
        };
        let pagination_with_cursor = Pagination {
            per_page: 2,
            cursor: Some(cursor_id),
            offset: None,
        };
        let (second_page, _) = repository
            .all(filter_with_cursor, pagination_with_cursor)
            .await
            .unwrap();

        // Verify cursor works - second page should not contain the cursor tag
        assert!(!second_page.iter().any(|t| t.id == cursor_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_tag() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestTagGuard::new(&repository);

        let created_tag = create_test_tag(
            &repository,
            user_uuid,
            &format!("find-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(created_tag.id);

        let found_tag = repository.find(created_tag.id).await.unwrap();

        assert_eq!(found_tag.id, created_tag.id);
        assert_eq!(found_tag.user_id, user_uuid);
        assert_eq!(found_tag.name, created_tag.name);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_tag_not_found() {
        let (_, repository, _) = setup().await;

        let non_existent_id = Uuid::new_v4();
        let result = repository.find(non_existent_id).await;

        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_delete_tag() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestTagGuard::new(&repository);

        let tag = create_test_tag(
            &repository,
            user_uuid,
            &format!("del-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(tag.id);

        repository.delete(tag.id).await.unwrap();

        // Verify deletion by trying to find it
        let result = repository.find(tag.id).await;
        assert!(result.is_err());
    }
}
