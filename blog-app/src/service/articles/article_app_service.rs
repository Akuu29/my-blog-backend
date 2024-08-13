use blog_domain::model::articles::{
    article::{Article, NewArticle, UpdateArticle},
    i_article_repository::IArticleRepository,
};

pub struct ArticleAppService<T: IArticleRepository> {
    repository: T,
}

impl<T: IArticleRepository> ArticleAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, user_id: i32, payload: NewArticle) -> anyhow::Result<Article> {
        self.repository.create(user_id, payload).await
    }

    pub async fn find(&self, id: i32) -> anyhow::Result<Article> {
        self.repository.find(id).await
    }

    pub async fn all(&self) -> anyhow::Result<Vec<Article>> {
        self.repository.all().await
    }

    pub async fn update(&self, id: i32, payload: UpdateArticle) -> anyhow::Result<Article> {
        self.repository.update(id, payload).await
    }

    pub async fn delete(&self, id: i32) -> anyhow::Result<()> {
        self.repository.delete(id).await
    }
}
