use crate::db::utils::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::articles::{
    article::{Article, NewArticle, UpdateArticle},
    article_filter::ArticleFilter,
    i_article_repository::IArticleRepository,
};
use sqlx::{types::Uuid, QueryBuilder};

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
    async fn create(&self, user_id: Uuid, payload: NewArticle) -> anyhow::Result<Article> {
        let article = sqlx::query_as::<_, Article>(
            r#"
            INSERT INTO articles (
                title,
                body,
                status,
                category_id,
                user_id
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5
            )
            RETURNING *;
            "#,
        )
        .bind(payload.title)
        .bind(payload.body)
        .bind(payload.status)
        .bind(payload.category_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(article)
    }

    async fn find(
        &self,
        article_id: i32,
        article_filter: Option<ArticleFilter>,
    ) -> anyhow::Result<Article> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT
                id,
                title,
                body,
                status,
                category_id,
                created_at,
                updated_at
            FROM articles
            WHERE id = $1
            "#,
        );

        let mut conditions = Vec::new();

        let mut user_id: Option<Uuid> = None;
        if let Some(article_filter) = article_filter {
            if article_filter.user_id.is_some() {
                conditions.push("user_id = $2");
                user_id = article_filter.user_id;
            }
        }

        if !conditions.is_empty() {
            query.push(" AND ").push(conditions.join(" AND "));
        }

        query.push(" ORDER BY id DESC; ");

        let article = query
            .build_query_as::<Article>()
            .bind(article_id)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(article)
    }

    async fn all(&self) -> anyhow::Result<Vec<Article>> {
        let articles = sqlx::query_as::<_, Article>(
            r#"
            SELECT
                id,
                title,
                body,
                status,
                category_id,
                created_at,
                updated_at
            FROM articles
            ORDER BY id DESC;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(articles)
    }

    async fn update(&self, article_id: i32, payload: UpdateArticle) -> anyhow::Result<Article> {
        let pre_payload = self.find(article_id, None).await?;
        let article = sqlx::query_as::<_, Article>(
            r#"
            UPDATE articles set
                title = $1,
                body = $2,
                status = $3,
                category_id = $4,
                updated_at = now()
            WHERE id = $5
            RETURNING *;
            "#,
        )
        .bind(payload.title.unwrap_or(pre_payload.title))
        .bind(payload.body.unwrap_or(pre_payload.body))
        .bind(payload.status.unwrap_or(pre_payload.status))
        .bind(payload.category_id)
        .bind(article_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(article)
    }

    async fn delete(&self, article_id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM articles
            WHERE id = $1;
            "#,
        )
        .bind(article_id)
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
    use sqlx::{types::Uuid, PgPool};

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
            category_id: Some(1),
        };

        // create
        let article = repository
            .create(Uuid::new_v4(), payload.clone())
            .await
            .unwrap();
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
            category_id: Some(2),
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
    use sqlx::types::Uuid;
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
        async fn create(&self, user_id: Uuid, payload: NewArticle) -> anyhow::Result<Article> {
            let mut store = self.write_store_ref();
            let id = (store.len() + 1) as i32;
            let article = Article {
                id,
                title: payload.title,
                body: payload.body,
                status: payload.status,
                category_id: payload.category_id,
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
            let category_id = payload.category_id.unwrap_or(article.category_id.unwrap());
            let created_at = article.created_at.clone();
            let article = Article {
                id,
                title,
                body,
                status,
                category_id: Some(category_id),
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
        use sqlx::types::Uuid;

        #[tokio::test]
        async fn test_article_repository_for_memory() {
            let repository = RepositoryForMemory::new();
            let payload = NewArticle {
                title: "title".to_string(),
                body: "body".to_string(),
                category_id: Some(1),
                status: ArticleStatus::Draft,
            };

            // create
            let article = repository
                .create(Uuid::new_v4(), payload.clone())
                .await
                .unwrap();
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
                category_id: Some(2),
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
