use crate::{service::tags::tag_service::TagService, utils::usecase_error::UsecaseError};
use blog_domain::model::{
    articles::{
        article::{Article, NewArticle, UpdateArticle},
        i_article_repository::{ArticleFilter, IArticleRepository},
    },
    common::{item_count::ItemCount, pagination::Pagination},
    tags::i_tag_repository::ITagRepository,
};
use sqlx::types::Uuid;

pub struct ArticleAppService<T, U>
where
    T: IArticleRepository,
    U: ITagRepository,
{
    article_repository: T,
    tag_service: TagService<U>,
}

impl<T: IArticleRepository, U: ITagRepository> ArticleAppService<T, U>
where
    T: IArticleRepository,
    U: ITagRepository,
{
    pub fn new(article_repository: T, tag_service: TagService<U>) -> Self {
        Self {
            article_repository,
            tag_service,
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

    pub async fn update(
        &self,
        user_id: Uuid,
        article_id: Uuid,
        payload: UpdateArticle,
    ) -> anyhow::Result<Article> {
        let pre_article = self
            .article_repository
            .find(article_id, ArticleFilter::default())
            .await?;

        // check ownership
        if pre_article.user_public_id != user_id {
            return Err(anyhow::anyhow!(UsecaseError::PermissionDenied(
                "You are not the owner of this article".to_string()
            )));
        }

        if (payload.title.is_none() && pre_article.title.is_none())
            || (payload.body.is_none() && pre_article.body.is_none())
        {
            return Err(anyhow::anyhow!(UsecaseError::ValidationFailed(
                "title or body is required".to_string()
            )));
        }

        self.article_repository.update(article_id, payload).await
    }

    pub async fn delete(&self, user_id: Uuid, article_id: Uuid) -> anyhow::Result<()> {
        let article = self
            .article_repository
            .find(article_id, ArticleFilter::default())
            .await?;

        // check ownership
        if article.user_public_id != user_id {
            return Err(anyhow::anyhow!(UsecaseError::PermissionDenied(
                "You are not the owner of this article".to_string()
            )));
        }

        self.article_repository.delete(article_id).await
    }

    pub async fn attach_tags(
        &self,
        user_id: Uuid,
        article_id: Uuid,
        tag_ids: Vec<Uuid>,
    ) -> anyhow::Result<()> {
        let article = self
            .article_repository
            .find(article_id, ArticleFilter::default())
            .await?;

        // check ownership
        if article.user_public_id != user_id {
            return Err(anyhow::anyhow!(UsecaseError::PermissionDenied(
                "You are not the owner of this article".to_string()
            )));
        }

        // check if the tags exists
        let exists_tags = self.tag_service.exists_tags(tag_ids.clone()).await?;
        if !exists_tags {
            return Err(anyhow::anyhow!(UsecaseError::ValidationFailed(
                "tag not found".to_string()
            )));
        }

        self.article_repository
            .attach_tags(article_id, tag_ids)
            .await
    }
}
