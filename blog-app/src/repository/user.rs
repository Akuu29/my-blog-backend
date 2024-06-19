use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn update(&self) -> anyhow::Result<()>;
    async fn delete(&self) -> anyhow::Result<()>;
}
