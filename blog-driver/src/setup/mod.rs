use crate::handler::{
    article::{all_articles, create_article, delete_article, find_article, update_article},
    comment::{create_comment, delete_comment, find_by_article_id, find_comment, update_comment},
    user::{signin, signup},
};
use axum::{
    http::Method,
    routing::{get, post},
    Extension, Router,
};
use blog_adapter::repository::{
    article::ArticleRepositoryForDb, comment::CommentRepositoryForDb,
    user::UserRepositoryForFirebase,
};
use blog_app::{
    repository::user::UserRepository,
    usecase::{article::ArticleUseCase, comment::CommentUseCase, user::UserUseCase},
};
use blog_domain::repository::{article::ArticleRepository, comment::CommentRepository};
use sqlx::PgPool;
use std::{env, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

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

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let article_use_case = ArticleUseCase::new(ArticleRepositoryForDb::new(pool.clone()));
    let comment_use_case = CommentUseCase::new(CommentRepositoryForDb::new(pool.clone()));

    let client = reqwest::Client::new();
    let api_key = env::var("FIREBASE_API_KEY").expect("undefined FIREBASE_API_KEY");
    let user_use_case = UserUseCase::new(UserRepositoryForFirebase::new(client, api_key));

    let router = create_router(cors, article_use_case, comment_use_case, user_use_case);
    let addr = &env::var("ADDR").expect("undefined ADDR");
    let lister = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    tracing::debug!("listening on {}", addr);

    axum::serve(lister, router).await.unwrap();
}

fn create_router<T: ArticleRepository, U: CommentRepository, S: UserRepository>(
    cors_layer: CorsLayer,
    article_use_case: ArticleUseCase<T>,
    comment_use_case: CommentUseCase<U>,
    user_use_case: UserUseCase<S>,
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
        .route("/users/signup", post(signup::<S>))
        .route("/users/signin", post(signin::<S>))
        .layer(cors_layer)
        .layer(Extension(Arc::new(article_use_case)))
        .layer(Extension(Arc::new(comment_use_case)))
        .layer(Extension(Arc::new(user_use_case)))
}

async fn root() -> &'static str {
    "Hello, world!"
}
