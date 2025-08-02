use blog_domain::model::tags::i_tag_repository::{ITagRepository, TagFilter};
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
        let tag_ids_len = tag_ids.len();
        let tag_filter = TagFilter {
            tag_ids: Some(tag_ids),
            ..Default::default()
        };
        let tags = self.repository.all(tag_filter).await?;

        if tags.len() != tag_ids_len {
            return Err(anyhow::anyhow!("Tag not found"));
        }

        Ok(true)
    }
}
