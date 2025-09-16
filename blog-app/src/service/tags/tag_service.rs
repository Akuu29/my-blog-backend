use blog_domain::model::{
    common::pagination::Pagination,
    tags::i_tag_repository::{ITagRepository, TagFilter},
};
use std::collections::HashSet;
use uuid::Uuid;

pub struct TagService<T>
where
    T: ITagRepository,
{
    repository: T,
}

impl<T> TagService<T>
where
    T: ITagRepository,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn exists_tags(&self, tag_ids: Vec<Uuid>) -> anyhow::Result<bool> {
        if tag_ids.is_empty() {
            return Ok(true);
        }

        let unique_tag_ids = tag_ids.into_iter().collect::<HashSet<Uuid>>();
        let tag_filter = TagFilter::new(None, Some(unique_tag_ids.iter().copied().collect()));
        let (_, total) = self
            .repository
            .all(tag_filter, Pagination::default())
            .await?;

        Ok(total.value() as usize == unique_tag_ids.len())
    }
}
