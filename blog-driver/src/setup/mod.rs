use crate::handler::{
    article::{all_articles, create_article, delete_article, find_article, update_article},
    auth::{signin, signup},
    comment::{create_comment, delete_comment, find_by_article_id, find_comment, update_comment},
    token::verify_id_token,
    user::{create, delete, find, update},
};
use axum::{
    http::Method,
    routing::{get, post},
    Extension, Router,
};
use blog_adapter::{
    db::users::user_repository::UserRepository,
    repository::{
        article::ArticleRepositoryForDb, auth::AuthRepositoryForFirebase,
        comment::CommentRepositoryForDb, token::TokenRepositoryForFirebase,
    },
};
use blog_app::{
    repository::{auth::AuthRepository, token::TokenRepository},
    service::users::user_app_service::UserAppService,
    usecase::{
        article::ArticleUseCase, auth::AuthUseCase, comment::CommentUseCase, token::TokenUseCase,
    },
};
use blog_domain::model::{
    articles::i_article_repository::ArticleRepository,
    comments::i_comment_repository::CommentRepository, users::i_user_repository::IUserRepository,
};
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
    let user_app_service = UserAppService::new(UserRepository::new(pool.clone()));

    let client = reqwest::Client::new();
    let api_key = env::var("FIREBASE_API_KEY").expect("undefined FIREBASE_API_KEY");
    let auth_use_case = AuthUseCase::new(AuthRepositoryForFirebase::new(
        client.clone(),
        api_key.clone(),
    ));
    let token_use_case = TokenUseCase::new(TokenRepositoryForFirebase::new(client.clone()));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let router = create_router(
        cors,
        auth_use_case,
        token_use_case,
        user_app_service,
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
    T: TokenRepository,
    U: IUserRepository,
    V: ArticleRepository,
    W: CommentRepository,
>(
    cors_layer: CorsLayer,
    auth_use_case: AuthUseCase<S>,
    token_use_case: TokenUseCase<T>,
    user_app_service: UserAppService<U>,
    article_use_case: ArticleUseCase<V>,
    comment_use_case: CommentUseCase<W>,
) -> Router {
    let auth_router = Router::new()
        .route("/signup", post(signup::<S>))
        .route("/signin", post(signin::<S>))
        .layer(Extension(Arc::new(auth_use_case)));

    let token_router = Router::new().route("/verify-id-token", get(verify_id_token::<T>));

    let users_router = Router::new()
        .route("/", post(create::<U>))
        .route(
            "/:id",
            get(find::<U>).patch(update::<U>).delete(delete::<U>),
        )
        .layer(Extension(Arc::new(user_app_service)));

    let articles_router = Router::new()
        .route("/", get(all_articles::<V>).post(create_article::<V, T>))
        .route(
            "/:id",
            get(find_article::<V>)
                .patch(update_article::<V, T>)
                .delete(delete_article::<V, T>),
        )
        .layer(Extension(Arc::new(article_use_case)));

    let comments_router = Router::new()
        .route("/", post(create_comment::<W>))
        .route(
            "/:id",
            get(find_comment::<W>)
                .patch(update_comment::<W>)
                .delete(delete_comment::<W>),
        )
        .route("/related/:article_id", get(find_by_article_id::<W>))
        .layer(Extension(Arc::new(comment_use_case)));

    Router::new()
        .route("/", get(root))
        .nest("/auth", auth_router)
        .nest("/token", token_router)
        .nest("/users", users_router)
        .nest("/articles", articles_router)
        .nest("/comments", comments_router)
        .layer(Extension(Arc::new(token_use_case)))
        .layer(cors_layer)
}

async fn root() -> &'static str {
    "Hello, world!"
}
