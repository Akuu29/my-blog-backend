use blog_domain::model::images::{
    i_image_repository::{IImageRepository, ImageFilter},
    image::{ImageData, ImageDataProps, NewImage},
};
use uuid::Uuid;

pub struct ImageAppService<T: IImageRepository> {
    repository: T,
}

impl<T: IImageRepository> ImageAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, new_image: NewImage) -> anyhow::Result<ImageDataProps> {
        let mut image = self.repository.create(new_image).await?;
        let image_url = match image.storage_type.to_string().as_str() {
            "database" => {
                format!(
                    "{}://{}/images/{}",
                    std::env::var("GATEWAY_PROTOCOL").expect("undefined GATEWAY_PROTOCOL"),
                    std::env::var("GATEWAY_DOMAIN").expect("undefined GATEWAY_DOMAIN"),
                    image.public_id
                )
            }
            _ => image.url.unwrap(),
        };

        image.url = Some(image_url);

        Ok(image)
    }

    pub async fn all(&self, filter: ImageFilter) -> anyhow::Result<Vec<ImageDataProps>> {
        let mut images = self.repository.all(filter).await?;

        for image in images.iter_mut() {
            let image_url = match image.storage_type.to_string().as_str() {
                "database" => {
                    format!(
                        "{}://{}/images/{}",
                        std::env::var("GATEWAY_PROTOCOL").expect("undefined GATEWAY_PROTOCOL"),
                        std::env::var("GATEWAY_DOMAIN").expect("undefined GATEWAY_DOMAIN"),
                        image.public_id
                    )
                }
                _ => image.url.as_ref().unwrap().to_string(),
            };

            image.url = Some(image_url);
        }

        Ok(images)
    }

    pub async fn find_data(&self, image_id: Uuid) -> anyhow::Result<ImageData> {
        self.repository.find_data(image_id).await
    }

    pub async fn delete(&self, image_id: Uuid) -> anyhow::Result<()> {
        self.repository.delete(image_id).await
    }
}
