use crate::utils::usecase_error::UsecaseError;
use blog_domain::model::{
    articles::{
        article::{Article, NewArticle, UpdateArticle},
        i_article_repository::{ArticleFilter, IArticleRepository},
    },
    common::pagination::Pagination,
};
use sqlx::types::Uuid;

pub struct ArticleAppService<T: IArticleRepository> {
    repository: T,
}

impl<T: IArticleRepository> ArticleAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, user_id: Uuid, payload: NewArticle) -> anyhow::Result<Article> {
        self.repository.create(user_id, payload).await
    }

    pub async fn find(
        &self,
        article_id: i32,
        article_filter: ArticleFilter,
    ) -> anyhow::Result<Article> {
        self.repository.find(article_id, article_filter).await
    }

    pub async fn all(
        &self,
        article_filter: ArticleFilter,
        pagination: Pagination,
    ) -> anyhow::Result<Vec<Article>> {
        self.repository.all(article_filter, pagination).await
    }

    pub async fn update(&self, article_id: i32, payload: UpdateArticle) -> anyhow::Result<Article> {
        let pre_article = self
            .repository
            .find(article_id, ArticleFilter::default())
            .await?;

        if (payload.title.is_none() && pre_article.title.is_none())
            || (payload.body.is_none() && pre_article.body.is_none())
        {
            return Err(anyhow::anyhow!(UsecaseError::ValidationFailed(
                "title or body is required".to_string()
            )));
        }

        self.repository.update(article_id, payload).await
    }

    pub async fn delete(&self, article_id: i32) -> anyhow::Result<()> {
        self.repository.delete(article_id).await
    }
}
