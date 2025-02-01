use blog_domain::model::tags::{
    i_tag_repository::ITagRepository,
    tag::{NewTag, Tag},
    tag_filter::TagFilter,
};
use sqlx::types::Uuid;

pub struct TagAppService<T: ITagRepository> {
    repository: T,
}

impl<T: ITagRepository> TagAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, user_id: Uuid, payload: NewTag) -> anyhow::Result<Tag> {
        self.repository.create(user_id, payload).await
    }

    pub async fn all(&self, tag_filter: TagFilter) -> anyhow::Result<Vec<Tag>> {
        self.repository.all(tag_filter).await
    }

    pub async fn delete(&self, tag_id: i32) -> anyhow::Result<()> {
        self.repository.delete(tag_id).await
    }
}
