use super::UserUsecaseError;
use blog_domain::{
    model::{
        common::{item_count::ItemCount, pagination::Pagination},
        users::{
            i_user_repository::{IUserRepository, UserFilter},
            user::{NewUser, UpdateUser, User},
        },
    },
    service::users::UserService,
};
use sqlx::types::Uuid;

pub struct UserAppService<T: IUserRepository> {
    repository: T,
}

impl<T: IUserRepository> UserAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, payload: NewUser) -> Result<User, UserUsecaseError> {
        self.repository
            .create(payload)
            .await
            .map_err(|e| UserUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn all(
        &self,
        user_filter: UserFilter,
        pagination: Pagination,
    ) -> Result<(Vec<User>, ItemCount), UserUsecaseError> {
        self.repository
            .all(user_filter, pagination)
            .await
            .map_err(|e| UserUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn find(&self, user_id: Uuid) -> Result<User, UserUsecaseError> {
        self.repository
            .find(user_id)
            .await
            .map_err(|e| UserUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn find_by_user_identity(
        &self,
        provider_name: &str,
        idp_sub: &str,
    ) -> Result<User, UserUsecaseError> {
        self.repository
            .find_by_user_identity(provider_name, idp_sub)
            .await
            .map_err(|e| UserUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn update_with_auth(
        &self,
        user_id: Uuid,
        authenticated_user_id: Uuid,
        payload: UpdateUser,
    ) -> Result<User, UserUsecaseError> {
        // Verify that the user is acting on their own account
        UserService::verify_self(user_id, authenticated_user_id)?;

        self.repository
            .update(user_id, payload)
            .await
            .map_err(|e| UserUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn delete_with_auth(
        &self,
        user_id: Uuid,
        authenticated_user_id: Uuid,
    ) -> Result<(), UserUsecaseError> {
        // Verify that the user is acting on their own account
        UserService::verify_self(user_id, authenticated_user_id)?;

        self.repository
            .delete(user_id)
            .await
            .map_err(|e| UserUsecaseError::RepositoryError(e.to_string()))?;

        Ok(())
    }
}
