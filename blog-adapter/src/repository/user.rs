use async_trait::async_trait;
use blog_app::repository::user::UserRepository;

#[derive(Debug, Clone)]
pub struct UserRepositoryForFirebase {
    client: reqwest::Client,
    api_key: String,
}

impl UserRepositoryForFirebase {
    pub fn new(client: reqwest::Client, api_key: String) -> Self {
        Self { client, api_key }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryForFirebase {
    async fn update(&self) -> anyhow::Result<()> {
        todo!()
    }
    async fn delete(&self) -> anyhow::Result<()> {
        todo!()
    }
}
