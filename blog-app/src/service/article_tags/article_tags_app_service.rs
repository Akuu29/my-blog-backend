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
        let tx = self.repository.tx_begin().await?;

        let delete_article_tags_result = self.repository.delete(payload.article_id).await;
        if let Err(e) = delete_article_tags_result {
            tx.rollback().await?;
            return Err(anyhow::anyhow!(e));
        }

        let bulk_insert_article_tags_result = self.repository.bulk_insert(payload).await;
        if let Err(e) = bulk_insert_article_tags_result {
            tx.rollback().await?;
            return Err(anyhow::anyhow!(e));
        }

        tx.commit().await?;

        Ok(bulk_insert_article_tags_result.unwrap())
    }
}
