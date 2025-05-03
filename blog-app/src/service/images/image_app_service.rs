use blog_domain::model::images::{
    i_image_repository::IImageRepository,
    image::{Image, NewImage},
};

pub struct ImageAppService<T: IImageRepository> {
    repository: T,
}

impl<T: IImageRepository> ImageAppService<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub async fn create(&self, new_image: NewImage) -> anyhow::Result<Image> {
        let result = self.repository.create(new_image).await;

        let mut image = result.unwrap();
        let image_url = match image.storage_type.to_string().as_str() {
            "database" => {
                format!(
                    "{}://{}/images/{}",
                    std::env::var("PROTOCOL").unwrap(),
                    std::env::var("DOMAIN").unwrap(),
                    image.id
                )
            }
            _ => image.url.unwrap(),
        };

        image.url = Some(image_url);

        Ok(image)
    }

    pub async fn find(&self, image_id: i32) -> anyhow::Result<Image> {
        self.repository.find(image_id).await
    }

    pub async fn delete(&self, image_id: i32) -> anyhow::Result<()> {
        self.repository.delete(image_id).await
    }
}
