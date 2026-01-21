use super::TagUsecaseError;
use blog_domain::{
    model::{
        common::{item_count::ItemCount, pagination::Pagination},
        tags::{
            i_tag_repository::{ITagRepository, TagFilter},
            tag::{NewTag, Tag},
        },
    },
    service::tags::TagService,
};
use sqlx::types::Uuid;

pub struct TagAppService<T: ITagRepository> {
    repository: T,
    tag_service: TagService<T>,
}

impl<T: ITagRepository> TagAppService<T> {
    pub fn new(repository: T) -> Self {
        let tag_service = TagService::new(repository.clone());
        Self {
            repository,
            tag_service,
        }
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

    pub async fn delete_with_auth(
        &self,
        user_id: Uuid,
        tag_id: Uuid,
    ) -> Result<(), TagUsecaseError> {
        // Verify tag ownership
        self.tag_service.verify_ownership(tag_id, user_id).await?;

        self.repository
            .delete(tag_id)
            .await
            .map_err(|e| TagUsecaseError::RepositoryError(e.to_string()))
    }
}
