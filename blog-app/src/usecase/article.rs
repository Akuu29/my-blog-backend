use blog_domain::{
    model::article::{Article, NewArticle, UpdateArticle},
    repository::article::ArticleRepository,
};

pub struct ArticleUseCase<T: ArticleRepository> {
    repository: T,
}

impl<T: ArticleRepository> ArticleUseCase<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, payload: NewArticle) -> anyhow::Result<Article> {
        self.repository.create(payload).await
    }

    pub async fn find(&self, id: i32) -> anyhow::Result<Article> {
        self.repository.find(id).await
    }

    pub async fn all(&self) -> anyhow::Result<Vec<Article>> {
        self.repository.all().await
    }

    pub async fn update(&self, id: i32, payload: UpdateArticle) -> anyhow::Result<Article> {
        self.repository.update(id, payload).await
    }

    pub async fn delete(&self, id: i32) -> anyhow::Result<()> {
        self.repository.delete(id).await
    }
}
