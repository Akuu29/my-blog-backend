use async_trait::async_trait;
use blog_domain::{
    model::article::{Article, NewArticle, UpdateArticle},
    repository::article::ArticleRepository,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use thiserror::Error;

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("Unexpected Error: [{0}]")]
    Unexpected(String),
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

type Articles = HashMap<i32, Article>;

#[derive(Debug, Clone)]
pub struct RepositoryForMemory {
    store: Arc<RwLock<Articles>>,
}

impl RepositoryForMemory {
    pub fn new() -> Self {
        Self {
            store: Arc::default(),
        }
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<Articles> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> RwLockReadGuard<Articles> {
        self.store.read().unwrap()
    }
}

#[async_trait]
impl ArticleRepository for RepositoryForMemory {
    async fn create(&self, payload: NewArticle) -> anyhow::Result<Article> {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        let article = Article {
            id,
            title: payload.title,
            body: payload.body,
            status: payload.status,
        };

        store.insert(id, article.clone());
        Ok(article)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Article> {
        let store = self.read_store_ref();
        let article = store
            .get(&id)
            .map(|article| article.clone())
            .ok_or(RepositoryError::NotFound(id))?;

        Ok(article)
    }

    async fn all(&self) -> anyhow::Result<Vec<Article>> {
        let store = self.read_store_ref();

        Ok(Vec::from_iter(
            store.values().map(|article| article.clone()),
        ))
    }

    async fn update(&self, id: i32, payload: UpdateArticle) -> anyhow::Result<Article> {
        let mut store = self.write_store_ref();
        let article = store.get(&id).unwrap();
        let title = payload.title.unwrap_or(article.title.clone());
        let body = payload.body.unwrap_or(article.body.clone());
        let status = payload.status.unwrap_or(article.status.clone());
        let article = Article {
            id,
            title,
            body,
            status,
        };

        store.insert(id, article.clone());
        Ok(article)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();

        store.remove(&id).unwrap();
        Ok(())
    }
}
