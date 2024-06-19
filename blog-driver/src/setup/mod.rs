use crate::handler::{
    article::{all_articles, create_article, delete_article, find_article, update_article},
    auth::{signin, signup},
    comment::{create_comment, delete_comment, find_by_article_id, find_comment, update_comment},
    user::{delete, update},
};
use axum::{
    http::Method,
    routing::{get, post, put},
    Extension, Router,
};
use blog_adapter::repository::{
    article::ArticleRepositoryForDb, auth::AuthRepositoryForFirebase,
    comment::CommentRepositoryForDb, user::UserRepositoryForFirebase,
};
use blog_app::{
    repository::{auth::AuthRepository, user::UserRepository},
    usecase::{
        article::ArticleUseCase, auth::AuthUseCase, comment::CommentUseCase, user::UserUseCase,
    },
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
    let article_use_case = ArticleUseCase::new(ArticleRepositoryForDb::new(pool.clone()));
    let comment_use_case = CommentUseCase::new(CommentRepositoryForDb::new(pool.clone()));

    let client = reqwest::Client::new();
    let api_key = env::var("FIREBASE_API_KEY").expect("undefined FIREBASE_API_KEY");
    let user_use_case = UserUseCase::new(UserRepositoryForFirebase::new(
        client.clone(),
        api_key.clone(),
    ));
    let auth_use_case = AuthUseCase::new(AuthRepositoryForFirebase::new(
        client.clone(),
        api_key.clone(),
    ));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let router = create_router(
        cors,
        auth_use_case,
        user_use_case,
        article_use_case,
        comment_use_case,
    );
    let addr = &env::var("ADDR").expect("undefined ADDR");
    let lister = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    tracing::debug!("listening on {}", addr);

    axum::serve(lister, router).await.unwrap();
}

fn create_router<
    S: AuthRepository,
    T: UserRepository,
    U: ArticleRepository,
    V: CommentRepository,
>(
    cors_layer: CorsLayer,
    auth_use_case: AuthUseCase<S>,
    user_use_case: UserUseCase<T>,
    article_use_case: ArticleUseCase<U>,
    comment_use_case: CommentUseCase<V>,
) -> Router {
    let auth_router = Router::new()
        .route("/signup", post(signup::<S>))
        .route("/signin", post(signin::<S>))
        .layer(Extension(Arc::new(auth_use_case)));

    let users_router = Router::new()
        .route("/", put(update::<T>).delete(delete::<T>))
        .layer(Extension(Arc::new(user_use_case)));

    let articles_router = Router::new()
        .route("/", get(all_articles::<U>).post(create_article::<U>))
        .route(
            "/:id",
            get(find_article::<U>)
                .patch(update_article::<U>)
                .delete(delete_article::<U>),
        )
        .layer(Extension(Arc::new(article_use_case)));

    let comments_router = Router::new()
        .route("/", post(create_comment::<V>))
        .route(
            "/:id",
            get(find_comment::<V>)
                .patch(update_comment::<V>)
                .delete(delete_comment::<V>),
        )
        .route("/related/:article_id", get(find_by_article_id::<V>))
        .layer(Extension(Arc::new(comment_use_case)));

    Router::new()
        .route("/", get(root))
        .nest("/auth", auth_router)
        .nest("/users", users_router)
        .nest("/articles", articles_router)
        .nest("/comments", comments_router)
        .layer(cors_layer)
}

async fn root() -> &'static str {
    "Hello, world!"
}
