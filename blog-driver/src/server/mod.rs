use axum::http::Method;
use axum_extra::extract::cookie::Key;
use blog_adapter::{
    db::{
        article_tags::article_tags_repository::ArticleTagsRepository,
        articles::article_repository::ArticleRepository,
        categories::category_repository::CategoryRepository,
        comments::comment_repository::CommentRepository, images::image_repository::ImageRepository,
        tags::tag_repository::TagRepository, users::user_repository::UserRepository,
    },
    idp::tokens::token_repository::TokenRepository,
    query_service::{
        articles_by_category::articles_category_query_service::ArticlesByCategoryQueryService,
        articles_by_tag::articles_tag_query_service::ArticlesByTagQueryService,
        tags_attached_article::tags_attached_article_query_service::TagsAttachedArticleQueryService,
    },
};
use blog_app::service::{
    article_tags::article_tags_app_service::ArticleTagsAppService,
    articles::article_app_service::ArticleAppService,
    categories::category_app_service::CategoryAppService,
    comments::comment_app_service::CommentAppService, images::image_app_service::ImageAppService,
    tags::tag_app_service::TagAppService, tokens::token_app_service::TokenAppService,
    users::user_app_service::UserAppService,
};
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE},
    HeaderValue,
};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

mod app_router;
use app_router::AppRouter;
mod app_state;
use app_state::AppState;

pub async fn run() {
    let log_level = std::env::var("RUST_LOG").unwrap_or("info".to_string());
    std::env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
    tracing::debug!("start connecting to database");
    let pool = PgPool::connect(&database_url).await.expect(&format!(
        "failed to connect to database, url is {}",
        database_url
    ));

    let http_client = reqwest::Client::new();

    // app services
    let article_app_service = ArticleAppService::new(ArticleRepository::new(pool.clone()));
    let comment_app_service = CommentAppService::new(CommentRepository::new(pool.clone()));
    let user_app_service = UserAppService::new(UserRepository::new(pool.clone()));
    let category_app_service = CategoryAppService::new(CategoryRepository::new(pool.clone()));
    let tag_app_service = TagAppService::new(TagRepository::new(pool.clone()));
    let article_tags_app_service =
        ArticleTagsAppService::new(ArticleTagsRepository::new(pool.clone()));
    let token_app_service = TokenAppService::new(TokenRepository::new(http_client.clone()));
    let image_app_service = ImageAppService::new(ImageRepository::new(pool.clone()));

    // query services
    let articles_by_category_query_service = ArticlesByCategoryQueryService::new(pool.clone());
    let article_by_tag_query_service = ArticlesByTagQueryService::new(pool.clone());
    let tags_attached_article_query_service = TagsAttachedArticleQueryService::new(pool.clone());

    let client_addr = std::env::var("CLIENT_ADDR").expect("undefined CLIENT_ADDR");
    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origin(client_addr.parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, COOKIE]);

    let master_key = std::env::var("MASTER_KEY").expect("undefined MASTER_KEY");
    let key = Key::from(master_key.as_bytes());
    let app_state = AppState::new(key);

    let app_router = AppRouter::new(
        cors_layer,
        app_state,
        token_app_service,
        user_app_service,
        article_app_service,
        comment_app_service,
        category_app_service,
        tag_app_service,
        article_tags_app_service,
        articles_by_category_query_service,
        article_by_tag_query_service,
        tags_attached_article_query_service,
        image_app_service,
    );
    let domain = std::env::var("DOMAIN").expect("undefined ADDR");
    let lister = tokio::net::TcpListener::bind(&domain)
        .await
        .expect("failed to bind");

    tracing::debug!("listening on {}", &domain);

    axum::serve(lister, app_router.router).await.unwrap();
}
