use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::{
    common::{item_count::ItemCount, pagination::Pagination},
    users::{
        i_user_repository::{IUserRepository, UserFilter},
        user::{NewUser, UpdateUser, User},
    },
};
use sqlx::{QueryBuilder, types::Uuid};

#[derive(Debug, Clone)]
pub struct UserRepository {
    pub pool: sqlx::PgPool,
}

impl UserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// push user where conditions to the query builder
    fn push_user_condition(
        &self,
        qb: &mut QueryBuilder<'_, sqlx::Postgres>,
        filter: &UserFilter,
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

        if let Some(name_contains) = filter.name_contains.clone() {
            push_condition(qb);
            qb.push("name ILIKE ")
                .push_bind(format!("%{}%", name_contains));
        }

        return has_condition;
    }
}

#[async_trait]
impl IUserRepository for UserRepository {
    async fn create(&self, payload: NewUser) -> anyhow::Result<User> {
        let mut tx = self.pool.begin().await?;

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (name, role)
            VALUES ($1, $2)
            RETURNING *;
            "#,
        )
        .bind(payload.name)
        .bind(payload.role)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO user_identities (
                user_id,
                provider_id,
                provider_user_id,
                provider_email_ciphertext,
                provider_email_cipher_nonce,
                provider_email_cipher_meta,
                provider_email_hash,
                is_primary
            )
            VALUES (
                (SELECT id FROM users WHERE public_id = $1),
                (SELECT id FROM identity_providers WHERE name = $2),
                $3,
                $4,
                $5,
                $6,
                $7,
                $8
            );
            "#,
        )
        .bind(user.public_id)
        .bind(payload.identity.provider_name)
        .bind(payload.identity.provider_user_id)
        .bind(payload.identity.provider_email_cipher.ciphertext)
        .bind(payload.identity.provider_email_cipher.nonce)
        .bind(payload.identity.provider_email_cipher.meta)
        .bind(payload.identity.provider_email_hash.0)
        .bind(payload.identity.is_primary)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    async fn all(
        &self,
        user_filter: UserFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<User>, ItemCount)> {
        // find users
        let mut qb = QueryBuilder::new(
            r#"
            SELECT
                public_id,
                name,
                role,
                is_active,
                last_login_at,
                created_at,
                updated_at
            FROM users
            "#,
        );

        // build conditions
        let has_condition = self.push_user_condition(&mut qb, &user_filter);

        /*
        build paginated conditions.
        cursor, offset can only be used once,
        because each is validated to prevent conflicts.
        */
        if let Some(cursor) = pagination.cursor {
            // get the id of the user with the given public_id
            let cid_option = sqlx::query_scalar!(
                r#"
                SELECT id FROM users WHERE public_id = $1
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
            qb.push("id < ").push_bind(cid);
        }

        qb.push(" ORDER BY id DESC");

        if let Some(offset) = pagination.offset {
            qb.push(" OFFSET ").push_bind(offset);
        }

        qb.push(" LIMIT ").push_bind(pagination.per_page);

        let users = qb.build_query_as::<User>().fetch_all(&self.pool).await?;

        // count total users
        let mut qb = QueryBuilder::new(
            r#"
            SELECT COUNT(*) FROM users
            "#,
        );
        // build conditions
        self.push_user_condition(&mut qb, &user_filter);

        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await?;

        Ok((users, total))
    }

    async fn find(&self, user_id: Uuid) -> anyhow::Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT
                public_id,
                name,
                role,
                is_active,
                last_login_at,
                created_at,
                updated_at
            FROM users
            WHERE public_id = $1;
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_user_identity(
        &self,
        provider_name: &str,
        idp_sub: &str,
    ) -> anyhow::Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT
                u.public_id,
                u.name,
                u.role,
                u.is_active,
                u.last_login_at,
                u.created_at,
                u.updated_at
            FROM users AS u
            JOIN user_identities AS ui ON u.id = ui.user_id
            WHERE ui.provider_id = (SELECT id FROM identity_providers WHERE name = $1)
            AND ui.provider_user_id = $2
            AND ui.is_primary = true;
            "#,
        )
        .bind(provider_name)
        .bind(idp_sub)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(user)
    }

    async fn update(&self, user_id: Uuid, payload: UpdateUser) -> anyhow::Result<User> {
        let pre_user = self.find(user_id).await?;
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users set name = $1
            WHERE public_id = $2
            RETURNING *;
            "#,
        )
        .bind(payload.name.unwrap_or(pre_user.name))
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn delete(&self, user_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM users
            WHERE public_id = $1;
            "#,
        )
        .bind(user_id)
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
    use blog_domain::model::{
        common::pagination::Pagination,
        users::user::{NewUser, UpdateUser, UserRole},
    };
    use dotenv::dotenv;
    use serial_test::serial;
    use sqlx::PgPool;

    // Test helper functions
    async fn setup() -> (PgPool, UserRepository) {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let repository = UserRepository::new(pool.clone());

        // Ensure "test-idp" identity provider exists for tests
        let _ = sqlx::query(
            r#"
            INSERT INTO identity_providers (name)
            VALUES ('test-idp')
            ON CONFLICT (name) DO NOTHING
            "#,
        )
        .execute(&pool)
        .await;

        (pool, repository)
    }

    async fn create_test_user(repository: &UserRepository, name_suffix: &str) -> User {
        let unique_email = format!("test-{}@example.com", name_suffix);
        let new_user = NewUser::new(
            "test-idp", // Assuming "google" identity provider exists in test DB
            &format!("test-idp-sub-{}", name_suffix),
            &unique_email,
            true,
        );
        repository.create(new_user).await.unwrap()
    }

    struct TestUserGuard {
        repository: UserRepository,
        user_ids: Vec<Uuid>,
        runtime_handle: tokio::runtime::Handle,
    }

    impl TestUserGuard {
        fn new(repository: &UserRepository) -> Self {
            Self {
                repository: repository.clone(),
                user_ids: Vec::new(),
                runtime_handle: tokio::runtime::Handle::current(),
            }
        }

        fn track(&mut self, user_id: Uuid) {
            self.user_ids.push(user_id);
        }
    }

    impl Drop for TestUserGuard {
        fn drop(&mut self) {
            let repository = self.repository.clone();
            let user_ids = self.user_ids.clone();
            let handle = self.runtime_handle.clone();

            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::task::block_in_place(|| {
                    handle.block_on(async move {
                        // Cleanup test users
                        for user_id in &user_ids {
                            let _ = repository.delete(*user_id).await;
                        }
                    });
                });
            }));
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_create_user() {
        let (_, repository) = setup().await;
        let mut guard = TestUserGuard::new(&repository);

        let unique_email = format!("create-{}@example.com", Uuid::new_v4());
        let new_user = NewUser::new(
            "test-idp",
            &format!("test-idp-sub-{}", Uuid::new_v4()),
            &unique_email,
            true,
        );

        let user = repository.create(new_user).await.unwrap();
        guard.track(user.public_id);

        assert!(!user.name.is_empty());
        assert_eq!(user.role, UserRole::User);
        assert!(user.is_active);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_user() {
        let (_, repository) = setup().await;
        let mut guard = TestUserGuard::new(&repository);

        let user = create_test_user(&repository, &Uuid::new_v4().to_string()).await;
        guard.track(user.public_id);

        let found_user = repository.find(user.public_id).await.unwrap();

        assert_eq!(found_user.public_id, user.public_id);
        assert_eq!(found_user.name, user.name);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_by_user_identity() {
        let (_, repository) = setup().await;
        let mut guard = TestUserGuard::new(&repository);

        let provider_sub = format!("test-idp-sub-{}", Uuid::new_v4());
        let unique_email = format!("identity-{}@example.com", Uuid::new_v4());
        let new_user = NewUser::new("test-idp", &provider_sub, &unique_email, true);

        let user = repository.create(new_user).await.unwrap();
        guard.track(user.public_id);

        let found_user = repository
            .find_by_user_identity("test-idp", &provider_sub)
            .await
            .unwrap();

        assert_eq!(found_user.public_id, user.public_id);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_users() {
        let (_, repository) = setup().await;
        let mut guard = TestUserGuard::new(&repository);

        let user1 = create_test_user(&repository, &Uuid::new_v4().to_string()).await;
        guard.track(user1.public_id);

        let user2 = create_test_user(&repository, &Uuid::new_v4().to_string()).await;
        guard.track(user2.public_id);

        let filter = UserFilter {
            name_contains: None,
        };
        let pagination = Pagination {
            per_page: 10,
            cursor: None,
            offset: None,
        };

        let (users, _) = repository.all(filter, pagination).await.unwrap();

        assert!(users.iter().any(|u| u.public_id == user1.public_id));
        assert!(users.iter().any(|u| u.public_id == user2.public_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_users_with_cursor() {
        let (_, repository) = setup().await;
        let mut guard = TestUserGuard::new(&repository);

        let user1 = create_test_user(&repository, &Uuid::new_v4().to_string()).await;
        guard.track(user1.public_id);

        let user2 = create_test_user(&repository, &Uuid::new_v4().to_string()).await;
        guard.track(user2.public_id);

        let user3 = create_test_user(&repository, &Uuid::new_v4().to_string()).await;
        guard.track(user3.public_id);

        let filter = UserFilter {
            name_contains: None,
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
        let filter_with_cursor = UserFilter {
            name_contains: None,
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

        // Verify cursor works - second page should not contain the cursor user
        assert!(!second_page.iter().any(|u| u.public_id == cursor_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_update_user() {
        let (_, repository) = setup().await;
        let mut guard = TestUserGuard::new(&repository);

        let user = create_test_user(&repository, &Uuid::new_v4().to_string()).await;
        guard.track(user.public_id);

        let new_name = format!("Updated-{}", Uuid::new_v4().to_string()[0..8].to_string());
        let update_payload = UpdateUser {
            name: Some(new_name.clone()),
        };

        let updated_user = repository
            .update(user.public_id, update_payload)
            .await
            .unwrap();

        assert_eq!(updated_user.public_id, user.public_id);
        assert_eq!(updated_user.name, new_name);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_delete_user() {
        let (_, repository) = setup().await;
        let mut guard = TestUserGuard::new(&repository);

        let user = create_test_user(&repository, &Uuid::new_v4().to_string()).await;
        guard.track(user.public_id);

        repository.delete(user.public_id).await.unwrap();

        // Verify deletion
        let result = repository.find(user.public_id).await;
        assert!(result.is_err());
    }
}
