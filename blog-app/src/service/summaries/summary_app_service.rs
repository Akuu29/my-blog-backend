use crate::{llm::i_summary_generator::ISummaryGenerator, service::error::UsecaseError};
use blog_domain::{
    model::articles::{
        article::ArticleStatus,
        article_summary::{ArticleSummary, NewArticleSummary, SummaryLanguage},
        i_article_repository::IArticleRepository,
        i_article_summary_repository::IArticleSummaryRepository,
    },
    service::articles::ArticleService,
};
use uuid::Uuid;

pub struct SummaryAppService<A, S, G>
where
    A: IArticleRepository,
    S: IArticleSummaryRepository,
    G: ISummaryGenerator,
{
    article_repository: A,
    article_service: ArticleService<A>,
    summary_repository: S,
    summary_generator: G,
}

impl<A, S, G> SummaryAppService<A, S, G>
where
    A: IArticleRepository,
    S: IArticleSummaryRepository,
    G: ISummaryGenerator,
{
    pub fn new(article_repository: A, summary_repository: S, summary_generator: G) -> Self {
        let article_service = ArticleService::new(article_repository.clone());
        Self {
            article_repository,
            article_service,
            summary_repository,
            summary_generator,
        }
    }

    pub async fn generate_with_auth(
        &self,
        user_id: Uuid,
        article_id: Uuid,
        lang: SummaryLanguage,
        locale: Option<String>,
    ) -> Result<ArticleSummary, UsecaseError> {
        let article = self
            .article_service
            .verify_ownership(article_id, user_id)
            .await?;

        if !matches!(
            article.status,
            ArticleStatus::Published | ArticleStatus::Private
        ) {
            return Err(UsecaseError::ValidationFailed(
                "summary can only be generated for published or private articles".to_string(),
            ));
        }

        let title = article
            .title
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| UsecaseError::ValidationFailed("article title is empty".to_string()))?;

        let body = article
            .body
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| UsecaseError::ValidationFailed("article body is empty".to_string()))?;

        let content = self
            .summary_generator
            .generate(title, body, lang, locale.as_deref())
            .await?;

        let summary = self
            .summary_repository
            .upsert(
                article_id,
                NewArticleSummary {
                    content,
                    lang,
                    locale,
                },
            )
            .await?;

        Ok(summary)
    }

    pub async fn find(&self, article_id: Uuid) -> Result<ArticleSummary, UsecaseError> {
        let summary = self.summary_repository.find(article_id).await?;
        Ok(summary)
    }

    pub async fn delete_with_auth(
        &self,
        user_id: Uuid,
        article_id: Uuid,
    ) -> Result<(), UsecaseError> {
        self.article_service
            .verify_ownership(article_id, user_id)
            .await?;

        self.summary_repository.delete(article_id).await?;

        Ok(())
    }
}
