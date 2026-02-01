use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::{
    categories::{
        category::{Category, NewCategory, UpdateCategory},
        i_category_repository::{CategoryFilter, ICategoryRepository},
    },
    common::{item_count::ItemCount, pagination::Pagination},
};
use sqlx::query_builder::QueryBuilder;
use uuid::Uuid;

#[derive(Clone)]
pub struct CategoryRepository {
    pool: sqlx::PgPool,
}

impl CategoryRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    fn push_category_condition(
        &self,
        qb: &mut QueryBuilder<'_, sqlx::Postgres>,
        filter: &CategoryFilter,
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

        if let Some(public_id) = filter.public_id {
            push_condition(qb);
            qb.push("c.public_id = ").push_bind(public_id);
        }

        if let Some(name) = filter.name.clone() {
            push_condition(qb);
            qb.push("c.name = ").push_bind(name);
        }

        if let Some(user_public_id) = filter.user_public_id {
            push_condition(qb);
            qb.push("u.public_id = ").push_bind(user_public_id);
        }

        return has_condition;
    }
}

#[async_trait]
impl ICategoryRepository for CategoryRepository {
    async fn find(&self, category_id: Uuid) -> anyhow::Result<Category> {
        let category = sqlx::query_as::<_, Category>(
            r#"
            SELECT
                c.public_id,
                u.public_id AS user_public_id,
                c.name,
                c.created_at,
                c.updated_at
            FROM categories AS c
            JOIN users AS u ON c.user_id = u.id
            WHERE c.public_id = $1
            ;
            "#,
        )
        .bind(category_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound.into(),
            e => anyhow::anyhow!(e),
        })?;

        Ok(category)
    }

    async fn create(&self, user_id: Uuid, payload: NewCategory) -> anyhow::Result<Category> {
        let category = sqlx::query_as::<_, Category>(
            r#"
            INSERT INTO categories (
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

        Ok(category)
    }

    async fn all(
        &self,
        category_filter: CategoryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Category>, ItemCount)> {
        // find categories
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                c.public_id,
                u.public_id AS user_public_id,
                c.name,
                c.created_at,
                c.updated_at
            FROM categories AS c
            LEFT JOIN users AS u ON c.user_id = u.id
            "#,
        );

        // build conditions
        let has_condition = self.push_category_condition(&mut qb, &category_filter);

        /*
        build paginated conditions.
        cursor, offset can only be used once,
        because each is validated to prevent conflicts.
        */
        if let Some(cursor) = pagination.cursor {
            let cid_option =
                sqlx::query_scalar::<_, i32>("SELECT id FROM categories WHERE public_id = $1")
                    .bind(cursor)
                    .fetch_optional(&self.pool)
                    .await?;

            let cid = cid_option.ok_or(RepositoryError::NotFound)?;
            if has_condition {
                qb.push(" AND ");
            } else {
                qb.push(" WHERE ");
            }
            qb.push("c.id < ").push_bind(cid);
        }

        qb.push(" ORDER BY c.id DESC");

        if let Some(offset) = pagination.offset {
            qb.push(" OFFSET ").push_bind(offset);
        }

        qb.push(" LIMIT ").push_bind(pagination.per_page);

        let categories = qb
            .build_query_as::<Category>()
            .fetch_all(&self.pool)
            .await?;

        // count total categories
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                COUNT(*)
            FROM categories AS c
            LEFT JOIN users AS u ON c.user_id = u.id
            "#,
        );
        // build conditions
        self.push_category_condition(&mut qb, &category_filter);
        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await?;

        Ok((categories, total))
    }

    async fn update(&self, category_id: Uuid, payload: UpdateCategory) -> anyhow::Result<Category> {
        let category = sqlx::query_as::<_, Category>(
            r#"
            UPDATE categories
            SET
                name = $1
            WHERE public_id = $2
            RETURNING
                public_id,
                (SELECT public_id FROM users WHERE id = categories.user_id) AS user_public_id,
                name,
                created_at,
                updated_at
            ;
            "#,
        )
        .bind(payload.name)
        .bind(category_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(category)
    }

    async fn delete(&self, category_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM categories
            WHERE public_id = $1
            ;
            "#,
        )
        .bind(category_id)
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
    async fn setup() -> (PgPool, CategoryRepository, Uuid) {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let repository = CategoryRepository::new(pool.clone());

        // Get test user public_id (UUID)
        let user_public_id = std::env::var("TEST_USER_ID").expect("undefined TEST_USER_ID");
        let user_uuid = uuid::Uuid::parse_str(&user_public_id).expect("invalid TEST_USER_ID UUID");

        (pool, repository, user_uuid)
    }

    async fn create_test_category(
        repository: &CategoryRepository,
        user_uuid: Uuid,
        name: &str,
    ) -> Category {
        let payload = NewCategory {
            name: name.to_string(),
        };
        repository.create(user_uuid, payload).await.unwrap()
    }

    struct TestCategoryGuard {
        repository: CategoryRepository,
        category_ids: Vec<Uuid>,
        runtime_handle: tokio::runtime::Handle,
    }

    impl TestCategoryGuard {
        fn new(repository: &CategoryRepository) -> Self {
            Self {
                repository: repository.clone(),
                category_ids: Vec::new(),
                runtime_handle: tokio::runtime::Handle::current(),
            }
        }

        fn track(&mut self, category_id: Uuid) {
            self.category_ids.push(category_id);
        }
    }

    impl Drop for TestCategoryGuard {
        fn drop(&mut self) {
            let repository = self.repository.clone();
            let category_ids = self.category_ids.clone();
            let handle = self.runtime_handle.clone();

            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::task::block_in_place(|| {
                    handle.block_on(async move {
                        // Cleanup test categories
                        for category_id in &category_ids {
                            let _ = repository.delete(*category_id).await;
                        }
                    });
                });
            }));
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_create_category() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestCategoryGuard::new(&repository);

        let unique_name = format!("test category {}", Uuid::new_v4());
        let payload = NewCategory {
            name: unique_name.clone(),
        };

        let category = repository.create(user_uuid, payload).await.unwrap();
        guard.track(category.public_id);

        assert_eq!(category.name, unique_name);
        assert_eq!(category.user_public_id, user_uuid);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_categories() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestCategoryGuard::new(&repository);

        let category1 = create_test_category(
            &repository,
            user_uuid,
            &format!("category 1 {}", Uuid::new_v4()),
        )
        .await;
        guard.track(category1.public_id);

        let category2 = create_test_category(
            &repository,
            user_uuid,
            &format!("category 2 {}", Uuid::new_v4()),
        )
        .await;
        guard.track(category2.public_id);

        let filter = CategoryFilter {
            public_id: None,
            name: None,
            user_public_id: Some(user_uuid),
        };
        let pagination = Pagination {
            per_page: 10,
            cursor: None,
            offset: None,
        };

        let (categories, _) = repository.all(filter, pagination).await.unwrap();

        assert!(
            categories
                .iter()
                .any(|c| c.public_id == category1.public_id)
        );
        assert!(
            categories
                .iter()
                .any(|c| c.public_id == category2.public_id)
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_categories_with_cursor() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestCategoryGuard::new(&repository);

        let category1 = create_test_category(
            &repository,
            user_uuid,
            &format!("category cursor 1 {}", Uuid::new_v4()),
        )
        .await;
        guard.track(category1.public_id);

        let category2 = create_test_category(
            &repository,
            user_uuid,
            &format!("category cursor 2 {}", Uuid::new_v4()),
        )
        .await;
        guard.track(category2.public_id);

        let category3 = create_test_category(
            &repository,
            user_uuid,
            &format!("category cursor 3 {}", Uuid::new_v4()),
        )
        .await;
        guard.track(category3.public_id);

        let filter = CategoryFilter {
            public_id: None,
            name: None,
            user_public_id: Some(user_uuid),
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
        let filter_with_cursor = CategoryFilter {
            public_id: None,
            name: None,
            user_public_id: Some(user_uuid),
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

        // Verify cursor works - second page should not contain the cursor category
        assert!(!second_page.iter().any(|c| c.public_id == cursor_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_update_category() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestCategoryGuard::new(&repository);

        let category = create_test_category(
            &repository,
            user_uuid,
            &format!("original name {}", Uuid::new_v4()),
        )
        .await;
        guard.track(category.public_id);

        let updated_name = format!("updated name {}", Uuid::new_v4());
        let update_payload = UpdateCategory {
            name: updated_name.clone(),
        };

        let updated_category = repository
            .update(category.public_id, update_payload)
            .await
            .unwrap();

        assert_eq!(updated_category.public_id, category.public_id);
        assert_eq!(updated_category.name, updated_name);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_category() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestCategoryGuard::new(&repository);

        let created_category = create_test_category(
            &repository,
            user_uuid,
            &format!("find test {}", Uuid::new_v4()),
        )
        .await;
        guard.track(created_category.public_id);

        let found_category = repository.find(created_category.public_id).await.unwrap();

        assert_eq!(found_category.public_id, created_category.public_id);
        assert_eq!(found_category.user_public_id, user_uuid);
        assert_eq!(found_category.name, created_category.name);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_category_not_found() {
        let (_, repository, _) = setup().await;

        let non_existent_id = Uuid::new_v4();
        let result = repository.find(non_existent_id).await;

        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_delete_category() {
        let (_, repository, user_uuid) = setup().await;
        let mut guard = TestCategoryGuard::new(&repository);

        let category = create_test_category(
            &repository,
            user_uuid,
            &format!("to delete {}", Uuid::new_v4()),
        )
        .await;
        guard.track(category.public_id);

        repository.delete(category.public_id).await.unwrap();

        // Verify deletion by trying to find it
        let result = repository.find(category.public_id).await;
        assert!(result.is_err());
    }
}
