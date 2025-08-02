use crate::{
    handler::{article, category, comment, image, tag, token, user},
    server::app_state::AppState,
    service::cookie_service::CookieService,
};
use axum::extract::DefaultBodyLimit;
use axum::{
    Extension, Router,
    routing::{delete, get, patch, post, put},
};
use blog_app::{
    query_service::{
        article_image::i_article_image_query_service::IArticleImageQueryService,
        articles_by_tag::i_articles_by_tag_query_service::IArticlesByTagQueryService,
        tags_attached_article::i_tags_attached_article_query_service::ITagsAttachedArticleQueryService,
    },
    service::{
        articles::article_app_service::ArticleAppService,
        categories::category_app_service::CategoryAppService,
        comments::comment_app_service::CommentAppService,
        images::image_app_service::ImageAppService, tags::tag_app_service::TagAppService,
        tokens::token_app_service::TokenAppService, users::user_app_service::UserAppService,
    },
};
use blog_domain::model::{
    articles::i_article_repository::IArticleRepository,
    categories::i_category_repository::ICategoryRepository,
    comments::i_comment_repository::ICommentRepository,
    images::i_image_repository::IImageRepository, tags::i_tag_repository::ITagRepository,
    tokens::i_token_repository::ITokenRepository, users::i_user_repository::IUserRepository,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub struct AppRouter {
    pub router: Router,
}

impl AppRouter {
    pub fn new<T, U, V, W, X, Y, B, C, D, E>(
        cors_layer: CorsLayer,
        app_state: AppState,
        token_app_service: TokenAppService<T>,
        user_app_service: UserAppService<U>,
        article_app_service: ArticleAppService<V, Y>,
        comment_app_service: CommentAppService<W>,
        category_app_service: CategoryAppService<X>,
        tag_app_service: TagAppService<Y>,
        article_by_tag_query_service: B,
        tags_attached_article_query_service: C,
        image_app_service: ImageAppService<D>,
        article_image_query_service: E,
        cookie_service: CookieService,
    ) -> Self
    where
        T: ITokenRepository,
        U: IUserRepository,
        V: IArticleRepository,
        W: ICommentRepository,
        X: ICategoryRepository,
        Y: ITagRepository,
        B: IArticlesByTagQueryService,
        C: ITagsAttachedArticleQueryService,
        D: IImageRepository,
        E: IArticleImageQueryService,
    {
        let token_router = Self::create_token_router::<T, U>();
        let users_router = Self::create_users_router::<T, U>();
        let articles_router =
            Self::create_articles_router::<T, V, B, Y>(article_by_tag_query_service);
        let comments_router = Self::create_comments_router::<W>(comment_app_service);
        let category_router = Self::create_category_router::<T, X>(category_app_service);
        let tag_router = Self::create_tag_router::<T, Y, C>(tags_attached_article_query_service);
        let image_router =
            Self::create_image_router::<D, T, E>(image_app_service, article_image_query_service);

        let max_request_body_size = std::env::var("MAX_REQUEST_BODY_SIZE")
            .expect("undefined MAX_REQUEST_BODY_SIZE")
            .parse::<usize>()
            .unwrap();

        let router = Router::new()
            .route("/hello-world", get(|| async { "Hello, world!" }))
            .nest("/token", token_router)
            .nest("/users", users_router)
            .nest("/articles", articles_router)
            .nest("/comments", comments_router)
            .nest("/categories", category_router)
            .nest("/tags", tag_router)
            .nest("/images", image_router)
            .layer(Extension(Arc::new(token_app_service)))
            .layer(Extension(Arc::new(user_app_service)))
            .layer(Extension(Arc::new(article_app_service)))
            .layer(Extension(Arc::new(tag_app_service)))
            .layer(Extension(Arc::new(cookie_service)))
            .layer(DefaultBodyLimit::max(max_request_body_size))
            .layer(cors_layer)
            .with_state(app_state);

        Self { router }
    }

    fn create_token_router<T, U>() -> Router<AppState>
    where
        T: ITokenRepository,
        U: IUserRepository,
    {
        Router::new()
            .route("/refresh", get(token::refresh_access_token::<T, U>))
            .route("/reset", get(token::reset_refresh_token))
    }

    fn create_users_router<T, U>() -> Router<AppState>
    where
        T: ITokenRepository,
        U: IUserRepository,
    {
        Router::new()
            .route("/signup", post(user::sign_up::<T, U>))
            .route("/signin", post(user::sign_in::<T, U>))
            .route(
                "/:user_id",
                get(user::find::<U>)
                    .patch(user::update::<U>)
                    .delete(user::delete::<U>),
            )
    }

    fn create_articles_router<T, U, V, X>(article_by_tag_query_service: V) -> Router<AppState>
    where
        T: ITokenRepository,
        U: IArticleRepository,
        V: IArticlesByTagQueryService,
        X: ITagRepository,
    {
        Router::new()
            .route(
                "/",
                get(article::all_articles::<U, X>).post(article::create_article::<U, T, X>),
            )
            .route(
                "/:article_id",
                get(article::find_article::<U, X>)
                    .patch(article::update_article::<U, T, X>)
                    .delete(article::delete_article::<U, T, X>),
            )
            .route("/:article_id/tags", put(article::attach_tags::<T, U, X>))
            .route("/tags", get(article::find_articles_by_tag::<V>))
            .layer(Extension(Arc::new(article_by_tag_query_service)))
    }

    fn create_comments_router<T>(comment_app_service: CommentAppService<T>) -> Router<AppState>
    where
        T: ICommentRepository,
    {
        Router::new()
            .route("/", post(comment::create_comment::<T>))
            .route(
                "/:id",
                get(comment::find_comment::<T>)
                    .patch(comment::update_comment::<T>)
                    .delete(comment::delete_comment::<T>),
            )
            .route(
                "/related/:article_id",
                get(comment::find_by_article_id::<T>),
            )
            .layer(Extension(Arc::new(comment_app_service)))
    }

    fn create_category_router<T, U>(category_app_service: CategoryAppService<U>) -> Router<AppState>
    where
        T: ITokenRepository,
        U: ICategoryRepository,
    {
        Router::new()
            .route(
                "/",
                get(category::all_categories::<U>).post(category::create_category::<U, T>),
            )
            .route(
                "/:category_id",
                patch(category::update_category::<U, T>).delete(category::delete_category::<U, T>),
            )
            .layer(Extension(Arc::new(category_app_service)))
    }

    fn create_tag_router<T, U, V>(tags_attached_article_query_service: V) -> Router<AppState>
    where
        T: ITokenRepository,
        U: ITagRepository,
        V: ITagsAttachedArticleQueryService,
    {
        Router::new()
            .route("/", post(tag::create::<U, T>).get(tag::all::<U>))
            .route("/:tag_id", delete(tag::delete::<U, T>))
            .route(
                "/article/:article_id",
                get(tag::find_tags_by_article_id::<V>),
            )
            .layer(Extension(Arc::new(tags_attached_article_query_service)))
    }

    fn create_image_router<T, U, E>(
        image_app_service: ImageAppService<T>,
        article_image_query_service: E,
    ) -> Router<AppState>
    where
        T: IImageRepository,
        U: ITokenRepository,
        E: IArticleImageQueryService,
    {
        Router::new()
            .route("/", post(image::create::<T, U>).get(image::all::<T>))
            .route(
                "/:image_id",
                get(image::find_data::<T>).delete(image::delete::<T, U, E>),
            )
            .layer(Extension(Arc::new(image_app_service)))
            .layer(Extension(Arc::new(article_image_query_service)))
    }
}
