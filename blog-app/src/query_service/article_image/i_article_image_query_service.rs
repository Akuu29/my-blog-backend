use async_trait::async_trait;
use sqlx::types::Uuid;

#[async_trait]
pub trait IArticleImageQueryService:
    Clone + std::marker::Send + std::marker::Sync + 'static
{
    async fn is_image_owned_by_user(&self, image_id: i32, user_id: Uuid) -> anyhow::Result<bool>;
}
