use crate::{
    handler::{article, category, comment, image, tag, token, user},
    middleware::logging::logging_middleware,
    server::app_state::AppState,
    service::cookie_service::CookieService,
};
use axum::extract::DefaultBodyLimit;
use axum::{
    Extension, Router, middleware,
    routing::{delete, get, patch, post, put},
};
use blog_app::{
    query_service::{
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
    pub fn new<
        TokenRepo,
        UserRepo,
        ArticleRepo,
        CommentRepo,
        CategoryRepo,
        TagRep,
        ArticleByTagQS,
        TagsAttachedArticleQS,
        ImageRepo,
    >(
        cors_layer: CorsLayer,
        app_state: AppState,
        token_app_service: TokenAppService<TokenRepo>,
        user_app_service: UserAppService<UserRepo>,
        article_app_service: ArticleAppService<ArticleRepo, TagRep>,
        comment_app_service: CommentAppService<CommentRepo>,
        category_app_service: CategoryAppService<CategoryRepo>,
        tag_app_service: TagAppService<TagRep>,
        article_by_tag_query_service: ArticleByTagQS,
        tags_attached_article_query_service: TagsAttachedArticleQS,
        image_app_service: ImageAppService<ImageRepo>,
        cookie_service: CookieService,
    ) -> Self
    where
        TokenRepo: ITokenRepository,
        UserRepo: IUserRepository,
        ArticleRepo: IArticleRepository,
        CommentRepo: ICommentRepository,
        CategoryRepo: ICategoryRepository,
        TagRep: ITagRepository,
        ArticleByTagQS: IArticlesByTagQueryService,
        TagsAttachedArticleQS: ITagsAttachedArticleQueryService,
        ImageRepo: IImageRepository,
    {
        let token_router = Self::create_token_router::<TokenRepo, UserRepo>();
        let users_router = Self::create_users_router::<TokenRepo, UserRepo>();
        let articles_router =
            Self::create_articles_router::<TokenRepo, ArticleRepo, ArticleByTagQS, TagRep>(
                article_app_service,
                article_by_tag_query_service,
            );
        let comments_router =
            Self::create_comments_router::<CommentRepo, TokenRepo>(comment_app_service);
        let category_router =
            Self::create_category_router::<TokenRepo, CategoryRepo>(category_app_service);
        let tag_router = Self::create_tag_router::<TokenRepo, TagRep, TagsAttachedArticleQS>(
            tags_attached_article_query_service,
        );
        let image_router = Self::create_image_router::<TokenRepo, ImageRepo>(image_app_service);

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
            .layer(Extension(Arc::new(tag_app_service)))
            .layer(Extension(Arc::new(cookie_service)))
            .layer(DefaultBodyLimit::max(max_request_body_size))
            .layer(cors_layer)
            .layer(middleware::from_fn(logging_middleware))
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
            .route("/", get(user::all::<U>))
            .route("/signup", post(user::sign_up::<T, U>))
            .route("/signin", post(user::sign_in::<T, U>))
            .route(
                "/:user_id",
                get(user::find::<U>)
                    .patch(user::update::<U, T>)
                    .delete(user::delete::<U, T>),
            )
    }

    fn create_articles_router<T, U, V, X>(
        article_app_service: ArticleAppService<U, X>,
        article_by_tag_query_service: V,
    ) -> Router<AppState>
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
            .route_layer(Extension(Arc::new(article_app_service)))
            .route_layer(Extension(Arc::new(article_by_tag_query_service)))
    }

    fn create_comments_router<T, U>(comment_app_service: CommentAppService<T>) -> Router<AppState>
    where
        T: ICommentRepository,
        U: ITokenRepository,
    {
        Router::new()
            .route(
                "/",
                get(comment::all_comments::<T>).post(comment::create_comment::<T, U>),
            )
            .route(
                "/:id",
                get(comment::find_comment::<T>)
                    .patch(comment::update_comment::<T, U>)
                    .delete(comment::delete_comment::<T, U>),
            )
            .route_layer(Extension(Arc::new(comment_app_service)))
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
            .route_layer(Extension(Arc::new(category_app_service)))
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
            .route_layer(Extension(Arc::new(tags_attached_article_query_service)))
    }

    fn create_image_router<T, U>(image_app_service: ImageAppService<U>) -> Router<AppState>
    where
        T: ITokenRepository,
        U: IImageRepository,
    {
        Router::new()
            .route("/", post(image::create::<T, U>).get(image::all::<U>))
            .route(
                "/:image_id",
                get(image::find_data::<U>).delete(image::delete::<T, U>),
            )
            .route_layer(Extension(Arc::new(image_app_service)))
    }
}
