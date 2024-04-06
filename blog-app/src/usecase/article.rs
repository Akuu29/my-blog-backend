use blog_domain::{
    model::article::{Article, NewArticle, UpdateArticle},
    repository::article::ArticleRepository,
};

pub struct ArticleUsecase<T: ArticleRepository> {
    repository: T,
}

impl<T: ArticleRepository> ArticleUsecase<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, payload: NewArticle) -> Article {
        self.repository.create(payload).await
    }

    pub async fn find(&self, id: i32) -> Option<Article> {
        self.repository.find(id).await
    }

    pub async fn all(&self) -> Vec<Article> {
        self.repository.all().await
    }

    pub async fn update(&self, id: i32, payload: UpdateArticle) -> Article {
        self.repository.update(id, payload).await
    }

    pub async fn delete(&self, id: i32) {
        self.repository.delete(id).await
    }
}
