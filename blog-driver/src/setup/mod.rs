use crate::handler::{
    article::{all_articles, create_article, delete_article, find_article, update_article},
    comment::{create_comment, delete_comment, find_by_article_id, find_comment, update_comment},
};
use axum::{
    routing::{get, post},
    Extension, Router,
};
use blog_adapter::repository::{article::ArticleRepositoryForDb, comment::CommentRepositoryForDb};
use blog_app::usecase::{article::ArticleUseCase, comment::CommentUseCase};
use blog_domain::repository::{article::ArticleRepository, comment::CommentRepository};
use sqlx::PgPool;
use std::{env, sync::Arc};

pub async fn create_server() {
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("undefined DATABASE_URL");
    tracing::debug!("start connecting to database");
    let pool = PgPool::connect(&database_url).await.expect(&format!(
        "failed to connect to database, url is {}",
        database_url
    ));

    let article_use_case = ArticleUseCase::new(ArticleRepositoryForDb::new(pool.clone()));
    let comment_use_case = CommentUseCase::new(CommentRepositoryForDb::new(pool.clone()));
    let router = create_router(article_use_case, comment_use_case);
    let addr = &env::var("ADDR").expect("undefined ADDR");
    let lister = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    tracing::debug!("listening on {}", addr);

    axum::serve(lister, router).await.unwrap();
}

fn create_router<T: ArticleRepository, U: CommentRepository>(
    article_use_case: ArticleUseCase<T>,
    comment_use_case: CommentUseCase<U>,
) -> Router {
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
        .route("/comments", post(create_comment::<U>))
        .route(
            "/comments/:id",
            get(find_comment::<U>)
                .patch(update_comment::<U>)
                .delete(delete_comment::<U>),
        )
        .route(
            "/comments/related/:article_id",
            get(find_by_article_id::<U>),
        )
        .layer(Extension(Arc::new(article_use_case)))
        .layer(Extension(Arc::new(comment_use_case)))
}

async fn root() -> &'static str {
    "Hello, world!"
}
