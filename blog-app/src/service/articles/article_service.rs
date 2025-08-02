use blog_domain::model::articles::i_article_repository::{ArticleFilter, IArticleRepository};
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

    pub async fn exists(&self, article_id: Uuid) -> anyhow::Result<bool> {
        self.repository
            .find(article_id, ArticleFilter::default())
            .await?;

        Ok(true)
    }
}
