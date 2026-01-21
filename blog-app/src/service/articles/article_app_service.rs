use super::ArticleUsecaseError;
use blog_domain::{
    model::{
        articles::{
            article::{Article, NewArticle, UpdateArticle},
            i_article_repository::{ArticleFilter, IArticleRepository},
        },
        common::{item_count::ItemCount, pagination::Pagination},
        tags::i_tag_repository::{ITagRepository, TagFilter},
    },
    service::articles::ArticleService,
};
use sqlx::types::Uuid;
use std::collections::HashSet;

pub struct ArticleAppService<T, U>
where
    T: IArticleRepository,
    U: ITagRepository,
{
    article_repository: T,
    tag_repository: U,
    article_service: ArticleService<T>,
}

impl<T: IArticleRepository, U: ITagRepository> ArticleAppService<T, U>
where
    T: IArticleRepository,
    U: ITagRepository,
{
    pub fn new(article_repository: T, tag_repository: U) -> Self {
        let article_service = ArticleService::new(article_repository.clone());
        Self {
            article_repository,
            tag_repository,
            article_service,
        }
    }

    pub async fn create(&self, user_id: Uuid, payload: NewArticle) -> anyhow::Result<Article> {
        self.article_repository.create(user_id, payload).await
    }

    pub async fn find(
        &self,
        article_id: Uuid,
        article_filter: ArticleFilter,
    ) -> anyhow::Result<Article> {
        self.article_repository
            .find(article_id, article_filter)
            .await
    }

    pub async fn all(
        &self,
        article_filter: ArticleFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Article>, ItemCount)> {
        self.article_repository
            .all(article_filter, pagination)
            .await
    }

    pub async fn update_with_auth(
        &self,
        user_id: Uuid,
        article_id: Uuid,
        payload: UpdateArticle,
    ) -> Result<Article, ArticleUsecaseError> {
        // Verify article ownership
        self.article_service
            .verify_ownership(article_id, user_id)
            .await?;

        let pre_article = self
            .article_repository
            .find(article_id, ArticleFilter::default())
            .await
            .map_err(|e| ArticleUsecaseError::RepositoryError(e.to_string()))?;

        if (payload.title.is_none() && pre_article.title.is_none())
            || (payload.body.is_none() && pre_article.body.is_none())
        {
            return Err(ArticleUsecaseError::ValidationFailed(
                "title or body is required".to_string(),
            ));
        }

        self.article_repository
            .update(article_id, payload)
            .await
            .map_err(|e| ArticleUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn delete_with_auth(
        &self,
        user_id: Uuid,
        article_id: Uuid,
    ) -> Result<(), ArticleUsecaseError> {
        // Verify article ownership
        self.article_service
            .verify_ownership(article_id, user_id)
            .await?;

        self.article_repository
            .delete(article_id)
            .await
            .map_err(|e| ArticleUsecaseError::RepositoryError(e.to_string()))
    }

    pub async fn attach_tags_with_auth(
        &self,
        user_id: Uuid,
        article_id: Uuid,
        tag_ids: Vec<Uuid>,
    ) -> Result<(), ArticleUsecaseError> {
        // Verify article ownership
        self.article_service
            .verify_ownership(article_id, user_id)
            .await?;

        // Validate that all tags exist and belong to the user
        if !tag_ids.is_empty() {
            let unique_tag_ids: HashSet<Uuid> = tag_ids.iter().copied().collect();
            let tag_filter = TagFilter::new(
                Some(user_id),
                Some(unique_tag_ids.iter().copied().collect()),
            );
            let (_, total) = self
                .tag_repository
                .all(tag_filter, Pagination::default())
                .await
                .map_err(|e| ArticleUsecaseError::RepositoryError(e.to_string()))?;

            if total.value() as usize != unique_tag_ids.len() {
                return Err(ArticleUsecaseError::ValidationFailed(
                    "One or more tags not found or not owned by user".to_string(),
                ));
            }
        }

        self.article_repository
            .attach_tags(article_id, tag_ids)
            .await
            .map_err(|e| ArticleUsecaseError::RepositoryError(e.to_string()))
    }
}
