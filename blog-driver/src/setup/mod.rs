use crate::handler::article::{
    all_articles, create_article, delete_article, find_article, update_article,
};
use axum::{routing::get, Extension, Router};
use blog_adapter::repository::RepositoryForMemory;
use blog_app::usecase::article::ArticleUsecase;
use blog_domain::repository::article::ArticleRepository;
use std::{env, sync::Arc};

pub async fn create_server() {
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let repository = RepositoryForMemory::new();
    let usecase = ArticleUsecase::new(repository);
    let router = create_router(usecase);
    let addr = &env::var("ADDR").expect("undefined ADDR");
    let lister = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    tracing::debug!("listening on {}", addr);

    axum::serve(lister, router).await.unwrap();
}

fn create_router<T: ArticleRepository>(usecase: ArticleUsecase<T>) -> Router {
    Router::new()
        .route("/", get(root))
        .route(
            "/articles",
            get(all_articles::<T>).post(create_article::<T>),
        )
        .route(
            "/articles/:id",
            get(find_article::<T>)
                .patch(update_article::<T>)
                .delete(delete_article::<T>),
        )
        .layer(Extension(Arc::new(usecase)))
}

async fn root() -> &'static str {
    "Hello, world!"
}
