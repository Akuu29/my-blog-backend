use blog_domain::model::article_tags::{
    article_tags::{ArticleAttachedTags, ArticleTag},
    i_article_tags_repository::IArticleTagsRepository,
};

pub struct ArticleTagsAppService<T: IArticleTagsRepository> {
    repository: T,
}

impl<T: IArticleTagsRepository> ArticleTagsAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn attach_tags_to_article(
        &self,
        payload: ArticleAttachedTags,
    ) -> anyhow::Result<Vec<ArticleTag>> {
        self.repository.delete_insert(payload).await
    }
}
