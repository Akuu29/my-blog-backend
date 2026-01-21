use super::ArticleServiceError;
use crate::model::articles::i_article_repository::{ArticleFilter, IArticleRepository};
use uuid::Uuid;

pub struct ArticleService<T>
where
    T: IArticleRepository,
{
    repository: T,
}

impl<T> ArticleService<T>
where
    T: IArticleRepository,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    /// Business Rule: A user can only modify or delete their own articles
    pub async fn verify_ownership(
        &self,
        article_id: Uuid,
        user_public_id: Uuid,
    ) -> Result<(), ArticleServiceError> {
        let article = self
            .repository
            .find(article_id, ArticleFilter::default())
            .await
            .map_err(|_| {
                // TODO Propagation of repository errors.
                ArticleServiceError::NotFound
            })?;

        if article.user_public_id != user_public_id {
            return Err(ArticleServiceError::Unauthorized);
        }

        Ok(())
    }
}
