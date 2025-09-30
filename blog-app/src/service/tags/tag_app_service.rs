use blog_domain::model::{
    common::{item_count::ItemCount, pagination::Pagination},
    tags::{
        i_tag_repository::{ITagRepository, TagFilter},
        tag::{NewTag, Tag},
    },
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

    pub async fn all(
        &self,
        tag_filter: TagFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Tag>, ItemCount)> {
        self.repository.all(tag_filter, pagination).await
    }

    pub async fn delete(&self, tag_id: Uuid) -> anyhow::Result<()> {
        self.repository.delete(tag_id).await
    }
}
