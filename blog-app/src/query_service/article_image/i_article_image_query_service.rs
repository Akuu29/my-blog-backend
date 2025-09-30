use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait IArticleImageQueryService:
    Clone + std::marker::Send + std::marker::Sync + 'static
{
    async fn is_image_owned_by_user(&self, image_id: Uuid, user_id: Uuid) -> anyhow::Result<bool>;
}
