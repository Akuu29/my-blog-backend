use crate::db::utils::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::articles::{
    article::{Article, NewArticle, UpdateArticle},
    i_article_repository::IArticleRepository,
};

#[derive(Debug, Clone)]
pub struct ArticleRepository {
    pool: sqlx::PgPool,
}

impl ArticleRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IArticleRepository for ArticleRepository {
    async fn create(&self, user_id: i32, payload: NewArticle) -> anyhow::Result<Article> {
        let article = sqlx::query_as::<_, Article>(
            r#"
            INSERT INTO articles (title, body, status, user_id)
            VALUES ($1, $2, $3, $4)
            RETURNING *;
            "#,
        )
        .bind(payload.title)
        .bind(payload.body)
        .bind(payload.status)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(article)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Article> {
        let article = sqlx::query_as::<_, Article>(
            r#"
            SELECT * FROM articles
            WHERE id = $1;
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(article)
    }

    async fn all(&self) -> anyhow::Result<Vec<Article>> {
        let articles = sqlx::query_as::<_, Article>(
            r#"
            SELECT * FROM articles
            ORDER BY id DESC;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(articles)
    }

    async fn update(&self, id: i32, payload: UpdateArticle) -> anyhow::Result<Article> {
        let pre_payload = self.find(id).await?;
        let article = sqlx::query_as::<_, Article>(
            r#"
            UPDATE articles set title = $1, body = $2, status = $3, updated_at = now()
            WHERE id = $4
            RETURNING *;
            "#,
        )
        .bind(payload.title.unwrap_or(pre_payload.title))
        .bind(payload.body.unwrap_or(pre_payload.body))
        .bind(payload.status.unwrap_or(pre_payload.status))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(article)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM articles
            WHERE id = $1;
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "database-test")]
mod test {
    use super::*;
    use blog_domain::model::articles::article::ArticleStatus;
    use dotenv::dotenv;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_article_repository_for_db() {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let repository = ArticleRepository::new(pool);
        let payload = NewArticle {
            title: "title".to_string(),
            body: "body".to_string(),
            status: ArticleStatus::Draft,
        };

        // create
        let article = repository.create(34, payload.clone()).await.unwrap();
        assert_eq!(article.title, payload.title);
        assert_eq!(article.body, payload.body);
        assert_eq!(article.status, payload.status);

        // find
        let article = repository.find(article.id).await.unwrap();
        assert_eq!(article.title, payload.title);
        assert_eq!(article.body, payload.body);
        assert_eq!(article.status, payload.status);

        // all
        let articles = repository.all().await.unwrap();
        assert_eq!(&article, articles.first().unwrap());

        // update
        let payload = UpdateArticle {
            title: Some("new title".to_string()),
            body: Some("new body".to_string()),
            status: Some(ArticleStatus::Published),
        };
        let article = repository
            .update(article.id, payload.clone())
            .await
            .unwrap();
        assert_eq!(article.title, payload.title.unwrap());
        assert_eq!(article.body, payload.body.unwrap());
        assert_eq!(article.status, payload.status.unwrap());

        // delete
        repository.delete(article.id).await.unwrap();
        let _ = repository.all().await.unwrap();
        let res = repository.find(article.id).await;
        assert!(res.is_err());
    }
}

#[cfg(test)]
pub mod test_util {
    use super::*;
    use chrono::Local;
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    };

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
    impl IArticleRepository for RepositoryForMemory {
        async fn create(&self, user_id: i32, payload: NewArticle) -> anyhow::Result<Article> {
            let mut store = self.write_store_ref();
            let id = (store.len() + 1) as i32;
            let article = Article {
                id,
                title: payload.title,
                body: payload.body,
                status: payload.status,
                created_at: Local::now(),
                updated_at: Local::now(),
            };

            store.insert(id, article.clone());
            Ok(article)
        }

        async fn find(&self, id: i32) -> anyhow::Result<Article> {
            let store = self.read_store_ref();
            let article = store
                .get(&id)
                .map(|article| article.clone())
                .ok_or(RepositoryError::NotFound)?;

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
            let created_at = article.created_at.clone();
            let article = Article {
                id,
                title,
                body,
                status,
                created_at,
                updated_at: Local::now(),
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

    #[cfg(test)]
    mod test {
        use super::*;
        use blog_domain::model::articles::article::ArticleStatus;

        #[tokio::test]
        async fn test_article_repository_for_memory() {
            let repository = RepositoryForMemory::new();
            let payload = NewArticle {
                title: "title".to_string(),
                body: "body".to_string(),
                status: ArticleStatus::Draft,
            };

            // create
            let article = repository.create(32, payload.clone()).await.unwrap();
            assert_eq!(article.title, payload.title);
            assert_eq!(article.body, payload.body);
            assert_eq!(article.status, payload.status);

            // find
            let article = repository.find(article.id).await.unwrap();
            assert_eq!(article.title, payload.title);
            assert_eq!(article.body, payload.body);
            assert_eq!(article.status, payload.status);

            // all
            let articles = repository.all().await.unwrap();
            assert_eq!(articles.len(), 1);

            // update
            let payload = UpdateArticle {
                title: Some("new title".to_string()),
                body: Some("new body".to_string()),
                status: Some(ArticleStatus::Published),
            };
            let article = repository
                .update(article.id, payload.clone())
                .await
                .unwrap();
            assert_eq!(article.title, payload.title.unwrap());
            assert_eq!(article.body, payload.body.unwrap());
            assert_eq!(article.status, payload.status.unwrap());

            // delete
            repository.delete(article.id).await.unwrap();
            let articles = repository.all().await.unwrap();
            assert_eq!(articles.len(), 0);
        }
    }
}
