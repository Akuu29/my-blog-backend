use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_domain::model::images::{
    i_image_repository::{IImageRepository, ImageFilter},
    image::{ImageData, ImageDataProps, NewImage},
};
use sqlx::QueryBuilder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ImageRepository {
    pool: sqlx::PgPool,
}

impl ImageRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IImageRepository for ImageRepository {
    async fn create(&self, new_image: NewImage) -> anyhow::Result<ImageDataProps> {
        let image = sqlx::query_as::<_, ImageDataProps>(
            r#"
            WITH sel_storage_type_id AS (
                    SELECT id FROM storage_types WHERE name = $5
                ),
                sel_article_id AS (
                    SELECT id FROM articles WHERE public_id = $6
                )
            INSERT INTO images (
                name,
                mime_type,
                data,
                url,
                storage_type_id,
                article_id
            )
            SELECT
                $1,
                $2,
                $3,
                $4,
                sel_storage_type_id.id,
                sel_article_id.id
            FROM sel_storage_type_id, sel_article_id
            RETURNING
                public_id,
                name,
                mime_type,
                url,
                $5 AS storage_type,
                $6 AS article_public_id,
                created_at,
                updated_at
            ;
            "#,
        )
        .bind(new_image.name)
        .bind(new_image.mime_type)
        .bind(new_image.data)
        .bind(new_image.url)
        .bind(new_image.storage_type.to_string())
        .bind(new_image.article_public_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(image)
    }

    async fn all(&self, filter: ImageFilter) -> anyhow::Result<Vec<ImageDataProps>> {
        let mut qb = QueryBuilder::new(
            r"
            SELECT
                i.public_id,
                i.name,
                i.mime_type,
                i.url,
                st.name AS storage_type,
                a.public_id AS article_public_id,
                i.created_at,
                i.updated_at
            FROM images AS i
            LEFT JOIN storage_types AS st
            ON i.storage_type_id = st.id
            LEFT JOIN articles AS a
            ON i.article_id = a.id
            ",
        );

        let mut first = true;
        let mut push_condition = |qb: &mut QueryBuilder<'_, sqlx::Postgres>| {
            if first {
                qb.push(" WHERE ");
                first = false;
            } else {
                qb.push(" AND ");
            }
        };

        if let Some(article_public_id) = filter.article_public_id {
            push_condition(&mut qb);
            qb.push("a.public_id = ").push_bind(article_public_id);
        }

        qb.push(" ORDER BY i.id DESC;");

        let images = qb
            .build_query_as::<ImageDataProps>()
            .fetch_all(&self.pool)
            .await?;

        Ok(images)
    }

    async fn find_data(&self, image_id: Uuid) -> anyhow::Result<ImageData> {
        let image_data = sqlx::query_as::<_, ImageData>(
            r#"
            SELECT
                mime_type,
                data
            FROM images
            WHERE public_id = $1
            ;
            "#,
        )
        .bind(image_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(image_data)
    }

    async fn delete(&self, image_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM images
            WHERE public_id = $1
            ;
            "#,
        )
        .bind(image_id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            e => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "database-test")]
mod test {
    use super::*;
    use dotenv::dotenv;
    use serial_test::serial;
    use sqlx::PgPool;

    // Test helper functions
    async fn setup() -> (PgPool, ImageRepository, Uuid, Uuid) {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
        let pool = PgPool::connect(&database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            database_url
        ));
        let repository = ImageRepository::new(pool.clone());

        // Get test user public_id (UUID)
        let user_public_id = std::env::var("TEST_USER_ID").expect("undefined TEST_USER_ID");
        let user_uuid = uuid::Uuid::parse_str(&user_public_id).expect("invalid TEST_USER_ID UUID");

        // Get internal user_id
        let user_id = sqlx::query_scalar::<_, i32>("SELECT id FROM users WHERE public_id = $1")
            .bind(user_uuid)
            .fetch_one(&pool)
            .await
            .expect("failed to get user_id from TEST_USER_ID");

        // Create a test article to use for images
        let article_public_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO articles (title, body, status, user_id)
            VALUES ($1, 'Test Body for Images', 'draft', $2)
            RETURNING public_id
            "#,
        )
        .bind(format!("Test Article for Images {}", Uuid::new_v4()))
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("failed to create test article");

        (pool, repository, user_uuid, article_public_id)
    }

    async fn create_test_image(
        repository: &ImageRepository,
        article_public_id: Uuid,
        name: &str,
    ) -> ImageDataProps {
        use blog_domain::model::images::image::StorageType;

        // Minimal valid 1x1 white PNG image
        let valid_png = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, // IHDR data + CRC
            0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
            0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, // Compressed data
            0x0D, 0x0A, 0x2D, 0xB4, // IDAT CRC
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
            0xAE, 0x42, 0x60, 0x82, // IEND CRC
        ];

        let new_image = NewImage {
            name: name.to_string(),
            mime_type: "image/png".to_string(),
            data: valid_png,
            url: None, // Database storage type stores data directly, not via URL
            storage_type: StorageType::Database,
            article_public_id,
        };
        repository.create(new_image).await.unwrap()
    }

    struct TestImageGuard {
        pool: PgPool,
        repository: ImageRepository,
        image_ids: Vec<Uuid>,
        article_public_id: Option<Uuid>,
        runtime_handle: tokio::runtime::Handle,
    }

    impl TestImageGuard {
        fn new(
            pool: &PgPool,
            repository: &ImageRepository,
            article_public_id: Option<Uuid>,
        ) -> Self {
            Self {
                pool: pool.clone(),
                repository: repository.clone(),
                image_ids: Vec::new(),
                article_public_id,
                runtime_handle: tokio::runtime::Handle::current(),
            }
        }

        fn track(&mut self, image_id: Uuid) {
            self.image_ids.push(image_id);
        }
    }

    impl Drop for TestImageGuard {
        fn drop(&mut self) {
            let pool = self.pool.clone();
            let repository = self.repository.clone();
            let image_ids = self.image_ids.clone();
            let article_public_id = self.article_public_id;
            let handle = self.runtime_handle.clone();

            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tokio::task::block_in_place(|| {
                    handle.block_on(async move {
                        // Cleanup test images
                        for image_id in &image_ids {
                            let _ = repository.delete(*image_id).await;
                        }

                        // Cleanup test article if created
                        if let Some(article_public_id) = article_public_id {
                            let _ = sqlx::query("DELETE FROM articles WHERE public_id = $1")
                                .bind(article_public_id)
                                .execute(&pool)
                                .await;
                        }
                    });
                });
            }));
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_create_image() {
        use blog_domain::model::images::image::StorageType;

        let (pool, repository, _, article_public_id) = setup().await;
        let mut guard = TestImageGuard::new(&pool, &repository, Some(article_public_id));

        let image_name = format!("test_image_{}.png", Uuid::new_v4());
        // Minimal valid 1x1 white PNG image
        let valid_png = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, // IHDR data + CRC
            0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
            0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, // Compressed data
            0x0D, 0x0A, 0x2D, 0xB4, // IDAT CRC
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
            0xAE, 0x42, 0x60, 0x82, // IEND CRC
        ];

        let new_image = NewImage {
            name: image_name.clone(),
            mime_type: "image/png".to_string(),
            data: valid_png,
            url: None, // Database storage type stores data directly, not via URL
            storage_type: StorageType::Database,
            article_public_id,
        };

        let image = repository.create(new_image).await.unwrap();
        guard.track(image.public_id);

        assert_eq!(image.name, image_name);
        assert_eq!(image.mime_type, "image/png");
        assert_eq!(image.storage_type, "database");
        assert_eq!(image.article_public_id, article_public_id);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_all_images() {
        let (pool, repository, _, article_public_id) = setup().await;
        let mut guard = TestImageGuard::new(&pool, &repository, Some(article_public_id));

        let image1 = create_test_image(
            &repository,
            article_public_id,
            &format!("image1_{}.png", Uuid::new_v4()),
        )
        .await;
        guard.track(image1.public_id);

        let image2 = create_test_image(
            &repository,
            article_public_id,
            &format!("image2_{}.png", Uuid::new_v4()),
        )
        .await;
        guard.track(image2.public_id);

        let filter = ImageFilter {
            article_public_id: Some(article_public_id),
        };

        let images = repository.all(filter).await.unwrap();

        assert!(images.iter().any(|i| i.public_id == image1.public_id));
        assert!(images.iter().any(|i| i.public_id == image2.public_id));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_find_data() {
        use blog_domain::model::images::image::StorageType;

        let (pool, repository, _, article_public_id) = setup().await;
        let mut guard = TestImageGuard::new(&pool, &repository, Some(article_public_id));

        // Minimal valid 1x1 white PNG image
        let test_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, // IHDR data + CRC
            0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
            0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, // Compressed data
            0x0D, 0x0A, 0x2D, 0xB4, // IDAT CRC
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
            0xAE, 0x42, 0x60, 0x82, // IEND CRC
        ];

        let new_image = NewImage {
            name: format!("test_find_{}.png", Uuid::new_v4()),
            mime_type: "image/png".to_string(),
            data: test_data.clone(),
            url: None, // Database storage type stores data directly, not via URL
            storage_type: StorageType::Database,
            article_public_id,
        };

        let image = repository.create(new_image).await.unwrap();
        guard.track(image.public_id);

        let image_data = repository.find_data(image.public_id).await.unwrap();

        assert_eq!(image_data.mime_type, "image/png");
        assert_eq!(image_data.data, test_data);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_delete_image() {
        let (pool, repository, _, article_public_id) = setup().await;
        let mut guard = TestImageGuard::new(&pool, &repository, Some(article_public_id));

        let image = create_test_image(
            &repository,
            article_public_id,
            &format!("to_delete_{}.png", Uuid::new_v4()),
        )
        .await;
        guard.track(image.public_id);

        repository.delete(image.public_id).await.unwrap();

        // Verify deletion
        let result = repository.find_data(image.public_id).await;
        assert!(result.is_err());
    }
}
