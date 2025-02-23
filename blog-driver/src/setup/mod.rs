use crate::handler::{
    article::{
        all_articles, create_article, delete_article, find_article, find_articles_by_tag,
        update_article,
    },
    article_tags,
    category::{
        all_categories, create_category, delete_category, find_articles_by_category,
        update_category,
    },
    comment::{create_comment, delete_comment, find_by_article_id, find_comment, update_comment},
    tag,
    token::{refresh_access_token, verify_id_token},
    user,
};
use axum::{
    http::Method,
    routing::{delete, get, patch, post},
    Extension, Router,
};
use axum_extra::extract::cookie::Key;
use blog_adapter::{
    db::{
        article_tags::article_tags_repository::ArticleTagsRepository,
        articles::article_repository::ArticleRepository,
        categories::category_repository::CategoryRepository,
        comments::comment_repository::CommentRepository, tags::tag_repository::TagRepository,
        users::user_repository::UserRepository,
    },
    idp::tokens::token_repository::TokenRepository,
    query_service::{
        articles_by_category::articles_category_query_service::ArticlesByCategoryQueryService,
        articles_by_tag::articles_tag_query_service::ArticlesByTagQueryService,
    },
};
use blog_app::{
    query_service::{
        articles_by_category::i_articles_by_category_query_service::IArticlesByCategoryQueryService,
        articles_by_tag::i_articles_by_tag_query_service::IArticlesByTagQueryService,
    },
    service::{
        article_tags::article_tags_app_service::ArticleTagsAppService,
        articles::article_app_service::ArticleAppService,
        categories::category_app_service::CategoryAppService,
        comments::comment_app_service::CommentAppService, tags::tag_app_service::TagAppService,
        tokens::token_app_service::TokenAppService, users::user_app_service::UserAppService,
    },
};
use blog_domain::model::{
    article_tags::i_article_tags_repository::IArticleTagsRepository,
    articles::i_article_repository::IArticleRepository,
    categories::i_category_repository::ICategoryRepository,
    comments::i_comment_repository::ICommentRepository, tags::i_tag_repository::ITagRepository,
    tokens::i_token_repository::ITokenRepository, users::i_user_repository::IUserRepository,
};
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE},
    HeaderValue,
};
use sqlx::PgPool;
use std::{env, sync::Arc};
use tower_http::cors::CorsLayer;

mod app_state;
use app_state::AppState;

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
    let comment_app_service = CommentAppService::new(CommentRepository::new(pool.clone()));
    let user_app_service = UserAppService::new(UserRepository::new(pool.clone()));
    let category_app_service = CategoryAppService::new(CategoryRepository::new(pool.clone()));
    let tag_app_service = TagAppService::new(TagRepository::new(pool.clone()));
    let article_tags_app_service =
        ArticleTagsAppService::new(ArticleTagsRepository::new(pool.clone()));

    let article_by_category_query_service = ArticlesByCategoryQueryService::new(pool.clone());
    let article_by_tag_query_service = ArticlesByTagQueryService::new(pool.clone());
    let client = reqwest::Client::new();
    let token_app_service = TokenAppService::new(TokenRepository::new(client.clone()));

    let client_addr = env::var("CLIENT_ADDR").expect("undefined CLIENT_ADDR");
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_origin(client_addr.parse::<HeaderValue>().unwrap())
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, COOKIE]);

    let app_state = AppState::new(Key::generate());

    let router = create_router(
        cors,
        app_state,
        token_app_service,
        user_app_service,
        article_app_service,
        comment_app_service,
        category_app_service,
        article_by_category_query_service,
        tag_app_service,
        article_tags_app_service,
        article_by_tag_query_service,
    );
    let addr = &env::var("ADDR").expect("undefined ADDR");
    let lister = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    tracing::debug!("listening on {}", addr);

    axum::serve(lister, router).await.unwrap();
}

fn create_router<
    T: ITokenRepository,
    U: IUserRepository,
    V: IArticleRepository,
    W: ICommentRepository,
    X: ICategoryRepository,
    Y: IArticlesByCategoryQueryService,
    Z: ITagRepository,
    A: IArticleTagsRepository,
    B: IArticlesByTagQueryService,
>(
    cors_layer: CorsLayer,
    app_state: AppState,
    token_app_service: TokenAppService<T>,
    user_app_service: UserAppService<U>,
    article_app_service: ArticleAppService<V>,
    comment_app_service: CommentAppService<W>,
    category_app_service: CategoryAppService<X>,
    articles_by_category_query_service: Y,
    tag_app_service: TagAppService<Z>,
    article_tags_app_service: ArticleTagsAppService<A>,
    article_by_tag_query_service: B,
) -> Router {
    let token_router = Router::new()
        .route("/verify", get(verify_id_token::<T, U>))
        .route("/refresh", get(refresh_access_token::<T, U>));

    let users_router = Router::new()
        .route("/protected", post(user::create::<U, T>))
        .route(
            "/:user_id",
            get(user::find::<U>)
                .patch(user::update::<U>)
                .delete(user::delete::<U>),
        );

    let articles_router = Router::new()
        .route("/", get(all_articles::<V>).post(create_article::<V, T>))
        .route(
            "/:article_id",
            get(find_article::<V>)
                .patch(update_article::<V, T>)
                .delete(delete_article::<V, T>),
        )
        .route("/by-tag", get(find_articles_by_tag::<B>))
        .layer(Extension(Arc::new(article_app_service)))
        .layer(Extension(Arc::new(article_by_tag_query_service)));

    let comments_router = Router::new()
        .route("/", post(create_comment::<W>))
        .route(
            "/:id",
            get(find_comment::<W>)
                .patch(update_comment::<W>)
                .delete(delete_comment::<W>),
        )
        .route("/related/:article_id", get(find_by_article_id::<W>))
        .layer(Extension(Arc::new(comment_app_service)));

    let category_router = Router::new()
        .route("/", get(all_categories::<X>).post(create_category::<X, T>))
        .route(
            "/:category_id",
            patch(update_category::<X, T>).delete(delete_category::<X, T>),
        )
        .route(
            "/:category_name/articles",
            get(find_articles_by_category::<Y>),
        )
        .layer(Extension(Arc::new(category_app_service)))
        .layer(Extension(Arc::new(articles_by_category_query_service)));

    let tag_router = Router::new()
        .route("/", post(tag::create::<Z, T>).get(tag::all::<Z>))
        .route("/:tag_id", delete(tag::delete::<Z, T>))
        .layer(Extension(Arc::new(tag_app_service)));

    let article_tags_router = Router::new()
        .route("/", post(article_tags::attach_tags_to_article::<A, T>))
        .layer(Extension(Arc::new(article_tags_app_service)));

    Router::new()
        .route("/", get(root))
        .nest("/token", token_router)
        .nest("/users", users_router)
        .nest("/articles", articles_router)
        .nest("/comments", comments_router)
        .nest("/categories", category_router)
        .nest("/tags", tag_router)
        .nest("/article-tags", article_tags_router)
        .layer(Extension(Arc::new(token_app_service)))
        .layer(Extension(Arc::new(user_app_service)))
        .layer(cors_layer)
        .with_state(app_state)
}

async fn root() -> &'static str {
    "Hello, world!"
}
