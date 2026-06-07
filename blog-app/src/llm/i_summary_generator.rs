use crate::llm::error::SummaryGeneratorError;
use async_trait::async_trait;
use blog_domain::model::articles::article_summary::SummaryLanguage;

#[async_trait]
pub trait ISummaryGenerator: Clone + Send + Sync + 'static {
    async fn generate(
        &self,
        title: &str,
        body: &str,
        lang: SummaryLanguage,
        locale: Option<&str>,
    ) -> Result<String, SummaryGeneratorError>;
}
