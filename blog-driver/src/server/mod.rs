use crate::config::AppConfig;
use crate::service::cookie_service::CookieService;
use axum::http::Method;
use axum_extra::extract::cookie::Key;
use blog_adapter::{
    db::{
        articles::article_repository::ArticleRepository,
        categories::category_repository::CategoryRepository,
        comments::comment_repository::CommentRepository,
        images::image_repository::ImageRepository,
        query_service::{
            articles_by_tag::articles_tag_query_service::ArticlesByTagQueryService,
            tags_attached_article::tags_attached_article_query_service::TagsAttachedArticleQueryService,
        },
        tags::tag_repository::TagRepository,
        users::user_repository::UserRepository,
    },
    idp::tokens::token_repository::TokenRepository,
};
use blog_app::service::{
    articles::article_app_service::ArticleAppService,
    categories::category_app_service::CategoryAppService,
    comments::comment_app_service::CommentAppService, images::image_app_service::ImageAppService,
    tags::tag_app_service::TagAppService, tokens::token_app_service::TokenAppService,
    users::user_app_service::UserAppService,
};
use http::{
    HeaderValue,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE},
};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use std::str::FromStr;
use tower_http::cors::CorsLayer;

mod app_router;
use app_router::AppRouter;
mod app_state;
use app_state::AppState;

pub async fn run() {
    tracing_subscriber::fmt::init();
    let config = AppConfig::from_env();

    tracing::info!("start connecting to database");

    let pool = if !config.db_ca_cert.is_empty() {
        tracing::info!("connecting to database with SSL");
        let options = PgConnectOptions::from_str(&config.database_url)
            .expect("invalid DATABASE_URL")
            .ssl_mode(PgSslMode::VerifyFull)
            .ssl_root_cert_from_pem(config.db_ca_cert.into_bytes());
        PgPoolOptions::new()
            .connect_with(options)
            .await
            .expect("failed to connect to database with SSL")
    } else {
        tracing::info!("connecting to database without SSL");
        PgPool::connect(&config.database_url).await.expect(&format!(
            "failed to connect to database, url is {}",
            config.database_url
        ))
    };

    let http_client = reqwest::Client::new();

    // app services
    let article_app_service = ArticleAppService::new(
        ArticleRepository::new(pool.clone()),
        TagRepository::new(pool.clone()),
    );
    let comment_app_service = CommentAppService::new(CommentRepository::new(pool.clone()));
    let user_app_service = UserAppService::new(UserRepository::new(pool.clone()));
    let category_app_service = CategoryAppService::new(CategoryRepository::new(pool.clone()));
    let tag_app_service = TagAppService::new(TagRepository::new(pool.clone()));
    let token_app_service = TokenAppService::new(
        TokenRepository::new(http_client.clone(), config.firebase_config),
        config.token_config,
    );
    let image_app_service =
        ImageAppService::new(ImageRepository::new(pool.clone()), config.image_config);

    // query services
    let article_by_tag_query_service = ArticlesByTagQueryService::new(pool.clone());
    let tags_attached_article_query_service = TagsAttachedArticleQueryService::new(pool.clone());

    // cookie service (It is a driver layer service)
    let cookie_service = CookieService::new(config.cookie_config);

    let client_addrs = config
        .client_addrs
        .iter()
        .map(|addr| addr.parse::<HeaderValue>().unwrap())
        .collect::<Vec<HeaderValue>>();
    let cors_layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::HEAD,
        ])
        .allow_origin(client_addrs)
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, COOKIE]);

    let key = Key::from(config.master_key.as_bytes());
    let app_state = AppState::new(key);

    let app_router = AppRouter::new(
        cors_layer,
        app_state,
        config.max_request_body_size,
        config.email_config,
        token_app_service,
        user_app_service,
        article_app_service,
        comment_app_service,
        category_app_service,
        tag_app_service,
        article_by_tag_query_service,
        tags_attached_article_query_service,
        image_app_service,
        cookie_service,
    );

    tracing::info!("listening on {}", &config.internal_api_domain);

    let lister = tokio::net::TcpListener::bind(&config.internal_api_domain)
        .await
        .expect("failed to bind");

    axum::serve(lister, app_router.router).await.unwrap();
}
