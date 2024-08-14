use crate::db::utils::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::comments::{
    comment::{Comment, NewComment, UpdateComment},
    i_comment_repository::ICommentRepository,
};

#[derive(Debug, Clone)]
pub struct CommentRepository {
    pool: sqlx::PgPool,
}

impl CommentRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ICommentRepository for CommentRepository {
    async fn create(&self, payload: NewComment) -> anyhow::Result<Comment> {
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            INSERT INTO comments (article_id, body, user_id)
            VALUES ($1, $2, $3)
            RETURNING *;
            "#,
        )
        .bind(payload.article_id)
        .bind(payload.body)
        .bind(payload.user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Comment> {
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            SELECT * FROM comments
            WHERE id = $1;
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    // TODO Bad approach because it's not scalable
    async fn find_by_article_id(&self, article_id: i32) -> anyhow::Result<Vec<Comment>> {
        let comments = sqlx::query_as::<_, Comment>(
            r#"
            SELECT * FROM comments
            WHERE article_id = $1
            ORDER BY id ASC;
            "#,
        )
        .bind(article_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(comments)
    }

    async fn update(&self, id: i32, payload: UpdateComment) -> anyhow::Result<Comment> {
        let pre_comment = self.find(id).await?;
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            UPDATE comments set body=$1, updated_at=now()
            WHERE id = $2
            RETURNING *;
            "#,
        )
        .bind(payload.body.unwrap_or(pre_comment.body))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(comment)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM comments
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
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_comment_repository_for_db() {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let repository = CommentRepository::new(pool);
        let payload = NewComment {
            article_id: 1,
            body: "test".to_string(),
            user_id: None,
        };

        // create
        let comment = repository.create(payload.clone()).await.unwrap();
        assert_eq!(comment.article_id, payload.article_id);
        assert_eq!(comment.body, payload.body);

        // find
        let comment = repository.find(comment.id).await.unwrap();
        assert_eq!(comment.article_id, payload.article_id);
        assert_eq!(comment.body, payload.body);

        // find_by_article_id
        let comments = repository
            .find_by_article_id(comment.article_id)
            .await
            .unwrap();
        assert_eq!(&comment, comments.last().unwrap());

        // update
        let payload = UpdateComment {
            body: Some("updated test".to_string()),
        };
        let comment = repository
            .update(comment.id, payload.clone())
            .await
            .unwrap();
        assert_eq!(comment.body, payload.body.unwrap());

        // delete
        repository.delete(comment.id).await.unwrap();
        let _ = repository.find(comment.id).await;
        let res = repository.find(comment.id).await;
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

    type Comments = HashMap<i32, Comment>;

    #[derive(Debug, Clone)]
    pub struct RepositoryForMemory {
        store: Arc<RwLock<Comments>>,
    }

    impl RepositoryForMemory {
        pub fn new() -> Self {
            Self {
                store: Arc::default(),
            }
        }

        fn write_store_ref(&self) -> RwLockWriteGuard<Comments> {
            self.store.write().unwrap()
        }

        fn read_store_ref(&self) -> RwLockReadGuard<Comments> {
            self.store.read().unwrap()
        }
    }

    #[async_trait]
    impl ICommentRepository for RepositoryForMemory {
        async fn create(&self, payload: NewComment) -> anyhow::Result<Comment> {
            let mut store = self.write_store_ref();
            let id = (store.len() + 1) as i32;
            let comment = Comment {
                id,
                article_id: payload.article_id,
                body: payload.body,
                created_at: Local::now(),
                updated_at: Local::now(),
            };

            store.insert(id, comment.clone());
            Ok(comment)
        }

        async fn find(&self, id: i32) -> anyhow::Result<Comment> {
            let store = self.read_store_ref();
            let comment = store
                .get(&id)
                .map(|comment| comment.clone())
                .ok_or(RepositoryError::NotFound)?;

            Ok(comment)
        }

        async fn find_by_article_id(&self, article_id: i32) -> anyhow::Result<Vec<Comment>> {
            let store = self.read_store_ref();
            let comments = store
                .values()
                .filter(|comment| comment.article_id == article_id)
                .cloned()
                .collect();

            Ok(comments)
        }

        async fn update(&self, id: i32, payload: UpdateComment) -> anyhow::Result<Comment> {
            let mut store = self.write_store_ref();
            let pre_comment = store.get(&id).ok_or(RepositoryError::NotFound)?.clone();
            let comment = Comment {
                id,
                article_id: pre_comment.article_id,
                body: payload.body.unwrap_or(pre_comment.body),
                created_at: pre_comment.created_at,
                updated_at: Local::now(),
            };

            store.insert(id, comment.clone());
            Ok(comment)
        }

        async fn delete(&self, id: i32) -> anyhow::Result<()> {
            let mut store = self.write_store_ref();
            store.remove(&id).ok_or(RepositoryError::NotFound)?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[tokio::test]
        async fn test_comment_repository_for_memory() {
            let repository = RepositoryForMemory::new();
            let payload = NewComment {
                article_id: 1,
                body: "test".to_string(),
                user_id: None,
            };

            // create
            let comment = repository.create(payload.clone()).await.unwrap();
            assert_eq!(comment.article_id, payload.article_id);
            assert_eq!(comment.body, payload.body);

            // find
            let comment = repository.find(comment.id).await.unwrap();
            assert_eq!(comment.article_id, payload.article_id);
            assert_eq!(comment.body, payload.body);

            // find_by_article_id
            let comments = repository
                .find_by_article_id(payload.article_id)
                .await
                .unwrap();
            assert_eq!(comments.len(), 1);
            assert_eq!(comments[0].article_id, payload.article_id);

            // update
            let payload = UpdateComment {
                body: Some("updated body".to_string()),
            };
            let comment = repository
                .update(comment.id, payload.clone())
                .await
                .unwrap();
            assert_eq!(comment.body, payload.body.unwrap());

            // delete
            repository.delete(comment.id).await.unwrap();
            let comments = repository.find(comment.id).await;
            assert!(comments.is_err());
        }
    }
}
