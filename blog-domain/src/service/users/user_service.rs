use crate::service::error::DomainServiceError;
use uuid::Uuid;

pub struct UserService;

impl UserService {
    /// Business Rule: Users can only modify or delete their own account
    pub fn verify_self(
        requested_user_id: Uuid,
        authenticated_user_id: Uuid,
    ) -> Result<(), DomainServiceError> {
        if requested_user_id != authenticated_user_id {
            return Err(DomainServiceError::Unauthorized);
        }

        Ok(())
    }
}
