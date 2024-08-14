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
    db::{articles::article_repository::ArticleRepository, users::user_repository::UserRepository},
    idp::tokens::token_repository::TokenRepository,
    repository::{auth::AuthRepositoryForFirebase, comment::CommentRepositoryForDb},
};
use blog_app::{
    repository::auth::AuthRepository,
    service::{
        articles::article_app_service::ArticleAppService,
        tokens::token_app_service::TokenAppService, users::user_app_service::UserAppService,
    },
    usecase::{auth::AuthUseCase, comment::CommentUseCase},
};
use blog_domain::model::{
    articles::i_article_repository::IArticleRepository,
    comments::i_comment_repository::CommentRepository,
    tokens::i_token_repository::ITokenRepository, users::i_user_repository::IUserRepository,
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
    let article_app_service = ArticleAppService::new(ArticleRepository::new(pool.clone()));
    let comment_use_case = CommentUseCase::new(CommentRepositoryForDb::new(pool.clone()));
    let user_app_service = UserAppService::new(UserRepository::new(pool.clone()));

    let client = reqwest::Client::new();
    let api_key = env::var("FIREBASE_API_KEY").expect("undefined FIREBASE_API_KEY");
    let auth_use_case = AuthUseCase::new(AuthRepositoryForFirebase::new(
        client.clone(),
        api_key.clone(),
    ));
    let token_app_service = TokenAppService::new(TokenRepository::new(client.clone()));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let router = create_router(
        cors,
        auth_use_case,
        token_app_service,
        user_app_service,
        article_app_service,
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
    T: ITokenRepository,
    U: IUserRepository,
    V: IArticleRepository,
    W: CommentRepository,
>(
    cors_layer: CorsLayer,
    auth_use_case: AuthUseCase<S>,
    token_app_service: TokenAppService<T>,
    user_app_service: UserAppService<U>,
    article_app_service: ArticleAppService<V>,
    comment_use_case: CommentUseCase<W>,
) -> Router {
    let auth_router = Router::new()
        .route("/signup", post(signup::<S>))
        .route("/signin", post(signin::<S>))
        .layer(Extension(Arc::new(auth_use_case)));

    let token_router = Router::new().route("/verify", get(verify_id_token::<T, U>));

    let users_router = Router::new().route("/", post(create::<U>)).route(
        "/:id",
        get(find::<U>).patch(update::<U>).delete(delete::<U>),
    );

    let articles_router = Router::new()
        .route("/", get(all_articles::<V>).post(create_article::<V, T>))
        .route(
            "/:id",
            get(find_article::<V>)
                .patch(update_article::<V, T>)
                .delete(delete_article::<V, T>),
        )
        .layer(Extension(Arc::new(article_app_service)));

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
        .layer(Extension(Arc::new(token_app_service)))
        .layer(Extension(Arc::new(user_app_service)))
        .layer(cors_layer)
}

async fn root() -> &'static str {
    "Hello, world!"
}
