use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::users::{
    i_user_repository::IUserRepository,
    user::{NewUser, UpdateUser, User},
};
use sqlx::types::Uuid;

#[derive(Debug, Clone)]
pub struct UserRepository {
    pub pool: sqlx::PgPool,
}

impl UserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
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

    async fn find(&self, user_id: Uuid) -> anyhow::Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE id = $1;
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
            WHERE id = $1;
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
