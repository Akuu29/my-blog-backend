use crate::db::utils::RepositoryError;
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
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (name, email, idp_sub)
            VALUES ($1, $2, $3)
            RETURNING *;
            "#,
        )
        .bind(payload.name)
        .bind(payload.email)
        .bind(payload.idp_sub)
        .fetch_one(&self.pool)
        .await?;

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

    // TODO Bad approach because it's not scalable
    async fn find_by_idp_sub(&self, idp_sub: &str) -> anyhow::Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE idp_sub = $1;
            "#,
        )
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
            UPDATE users set name = $1, email = $2
            WHERE id = $3
            RETURNING *;
            "#,
        )
        .bind(payload.name.unwrap_or(pre_user.name))
        .bind(payload.email.unwrap_or(pre_user.email))
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
