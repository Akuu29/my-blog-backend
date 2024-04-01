use crate::model::article::{Article, NewArticle, UpdateArticle};
use async_trait::async_trait;

#[async_trait]
pub trait ArticleRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: NewArticle) -> Article;
    async fn find(&self, id: i32) -> Option<Article>;
    async fn all(&self) -> Vec<Article>;
    async fn update(&self, id: i32, payload: UpdateArticle) -> Article;
    async fn delete(&self, id: i32) -> ();
}
