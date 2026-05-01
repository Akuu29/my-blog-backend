use crate::{config::ImageConfig, service::error::UsecaseError};
use blog_domain::{
    model::images::{
        i_image_repository::{IImageRepository, ImageFilter},
        image::{ImageData, ImageDataProps, NewImage},
    },
    service::images::ImageService,
};
use uuid::Uuid;

pub struct ImageAppService<T: IImageRepository> {
    repository: T,
    image_service: ImageService<T>,
    config: ImageConfig,
}

impl<T: IImageRepository> ImageAppService<T>
where
    T: Clone,
{
    /// Create a new ImageAppService
    ///
    /// # Design Decision: Why instantiate ImageService inside new()?
    ///
    /// Instead of passing ImageService as a parameter, we instantiate it here for:
    ///
    /// 1. **API Simplicity**: Caller only needs to provide repository
    /// 2. **Encapsulation**: AppService manages its own domain service dependencies
    /// 3. **Consistency**: ImageService always uses the same repository as AppService
    /// 4. **Appropriate for this architecture**:
    ///    - Domain services are simple (thin wrapper around repository)
    ///    - Repository is Arc-wrapped, so clone() is cheap (O(1) reference count increment)
    ///    - Testing is done at repository level, not domain service level
    ///
    /// Trade-off: Less flexible than dependency injection, but simpler for current needs.
    /// If domain services become complex or need multiple implementations, consider
    /// adding a `with_service()` constructor for testing/customization.
    pub fn new(repository: T, config: ImageConfig) -> Self {
        let image_service = ImageService::new(repository.clone());
        Self {
            repository,
            image_service,
            config,
        }
    }

    pub async fn create(&self, new_image: NewImage) -> Result<ImageDataProps, UsecaseError> {
        let mut image = self.repository.create(new_image).await?;
        let image_url = match image.storage_type.to_string().as_str() {
            "database" => {
                format!(
                    "{}://{}/images/{}",
                    self.config.gateway_protocol, self.config.gateway_domain, image.id
                )
            }
            _ => image.url.unwrap(),
        };

        image.url = Some(image_url);

        Ok(image)
    }

    pub async fn all(&self, filter: ImageFilter) -> Result<Vec<ImageDataProps>, UsecaseError> {
        let mut images = self.repository.all(filter).await?;

        for image in images.iter_mut() {
            let image_url = match image.storage_type.to_string().as_str() {
                "database" => {
                    format!(
                        "{}://{}/images/{}",
                        self.config.gateway_protocol, self.config.gateway_domain, image.id
                    )
                }
                _ => image.url.as_ref().unwrap().to_string(),
            };

            image.url = Some(image_url);
        }

        Ok(images)
    }

    pub async fn find_data(&self, image_id: Uuid) -> Result<ImageData, UsecaseError> {
        Ok(self.repository.find_data(image_id).await?)
    }

    pub async fn delete_with_auth(
        &self,
        image_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), UsecaseError> {
        // Verify image ownership
        self.image_service
            .verify_ownership(image_id, user_id)
            .await?;

        self.repository.delete(image_id).await?;

        Ok(())
    }
}
