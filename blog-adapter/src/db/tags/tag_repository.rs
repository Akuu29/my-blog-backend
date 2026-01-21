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
            qb.push("u.public_id = ").push_bind(user_id);
        }

        if let Some(tag_ids) = filter.tag_ids.clone() {
            push_condition(qb);
            qb.push("t.public_id = ANY(").push_bind(tag_ids).push(")");
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
            VALUES (
                $1,
                (SELECT id FROM users WHERE public_id = $2)
            )
            RETURNING
                public_id,
                $2 AS user_public_id,
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
                t.public_id,
                u.public_id AS user_public_id,
                t.name,
                t.created_at,
                t.updated_at
            FROM tags AS t
            JOIN users AS u ON t.user_id = u.id
            WHERE t.public_id = $1
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
                t.public_id,
                u.public_id AS user_public_id,
                t.name,
                t.created_at,
                t.updated_at
            FROM tags AS t
            JOIN users AS u
            ON t.user_id = u.id
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
            let cid_option = sqlx::query_scalar!(
                r#"
                SELECT id FROM tags WHERE public_id = $1
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
            qb.push("t.id < ").push_bind(cid);
        }

        qb.push(" ORDER BY t.id DESC");

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
            LEFT JOIN users AS u ON t.user_id = u.id
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
            WHERE public_id = $1
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

        // Get test user public_id (UUID)
        let user_public_id = std::env::var("TEST_USER_ID").expect("undefined TEST_USER_ID");
        let user_uuid = uuid::Uuid::parse_str(&user_public_id).expect("invalid TEST_USER_ID UUID");

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
        guard.track(tag.public_id);

        assert_eq!(tag.name, unique_name);
        assert_eq!(tag.user_public_id, user_uuid);
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
        guard.track(tag1.public_id);

        let tag2 = create_test_tag(
            &repository,
            user_uuid,
            &format!("tag2-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(tag2.public_id);

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

        assert!(tags.iter().any(|t| t.public_id == tag1.public_id));
        assert!(tags.iter().any(|t| t.public_id == tag2.public_id));
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
        guard.track(tag1.public_id);

        let tag2 = create_test_tag(
            &repository,
            user_uuid,
            &format!("cur2-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(tag2.public_id);

        let tag3 = create_test_tag(
            &repository,
            user_uuid,
            &format!("cur3-{}", Uuid::new_v4().to_string()[0..8].to_string()),
        )
        .await;
        guard.track(tag3.public_id);

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
        let cursor_id = first_page[1].public_id;
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
        assert!(!second_page.iter().any(|t| t.public_id == cursor_id));
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
        guard.track(created_tag.public_id);

        let found_tag = repository.find(created_tag.public_id).await.unwrap();

        assert_eq!(found_tag.public_id, created_tag.public_id);
        assert_eq!(found_tag.user_public_id, user_uuid);
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
        guard.track(tag.public_id);

        repository.delete(tag.public_id).await.unwrap();

        // Verify deletion by trying to find it
        let result = repository.find(tag.public_id).await;
        assert!(result.is_err());
    }
}
