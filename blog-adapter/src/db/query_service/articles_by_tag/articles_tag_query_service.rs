use crate::utils::repository_error::RepositoryError;
use async_trait::async_trait;
use blog_app::query_service::articles_by_tag::i_articles_by_tag_query_service::{
    ArticlesByTagFilter, IArticlesByTagQueryService,
};
use blog_domain::model::{
    articles::article::Article,
    common::{item_count::ItemCount, pagination::Pagination},
};
use sqlx::query_builder::QueryBuilder;

#[derive(Debug, Clone)]
pub struct ArticlesByTagQueryService {
    pool: sqlx::PgPool,
}

impl ArticlesByTagQueryService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// push article where conditions to the query builder
    fn push_article_condition(
        &self,
        qb: &mut QueryBuilder<'_, sqlx::Postgres>,
        filter: &ArticlesByTagFilter,
    ) {
        if let Some(user_public_id) = filter.user_public_id {
            qb.push(" AND u.public_id = ").push_bind(user_public_id);
        }
    }
}

#[async_trait]
impl IArticlesByTagQueryService for ArticlesByTagQueryService {
    async fn find_article_title_by_tag(
        &self,
        filter: ArticlesByTagFilter,
        pagination: Pagination,
    ) -> anyhow::Result<(Vec<Article>, ItemCount)> {
        // find articles
        let mut qb = QueryBuilder::new(
            r#"
            WITH tag_ids AS (
                SELECT id
                FROM tags
                WHERE public_id IN (
            "#,
        );
        let mut separated = qb.separated(",");
        for tag_id in filter.tag_ids.iter() {
            separated.push_bind(tag_id);
        }
        separated.push_unseparated(") ");

        qb.push(
            r#"
            )
            SELECT
                a.public_id,
                a.title,
                a.body,
                status,
                (SELECT public_id FROM categories WHERE id = category_id) as category_public_id,
                a.created_at,
                a.updated_at
            FROM articles AS a
            LEFT JOIN users AS u ON a.user_id = u.id
            WHERE EXISTS (
                SELECT 1
                FROM article_tags AS at
                WHERE at.article_id = a.id
                AND at.tag_id IN (SELECT id FROM tag_ids)
            )
            "#,
        );

        // build conditions
        self.push_article_condition(&mut qb, &filter);

        if let Some(cursor) = pagination.cursor {
            // get the id of the article with the given public_id
            let cid_option = sqlx::query_scalar!(
                r#"
                SELECT id FROM articles WHERE public_id = $1
                "#,
                cursor
            )
            .fetch_optional(&self.pool)
            .await?;

            let cid = cid_option.ok_or(RepositoryError::NotFound)?;
            qb.push(" AND a.id < ").push_bind(cid);
        }

        qb.push(" ORDER BY a.id DESC LIMIT ")
            .push_bind(pagination.per_page);

        let articles = qb.build_query_as::<Article>().fetch_all(&self.pool).await?;

        // count total articles
        let mut qb = QueryBuilder::new(
            r#"
            WITH tag_ids AS (
                SELECT id
                FROM tags
                WHERE public_id IN (
            "#,
        );
        let mut separated = qb.separated(",");
        for tag_id in filter.tag_ids.iter() {
            separated.push_bind(tag_id);
        }
        separated.push_unseparated(") ");

        qb.push(
            r#"
            )
            SELECT COUNT(*)
            FROM articles AS a
            LEFT JOIN users AS u ON a.user_id = u.id
            WHERE EXISTS (
                SELECT 1
                FROM article_tags AS at
                WHERE at.article_id = a.id
                AND at.tag_id IN (SELECT id FROM tag_ids)
            )
            "#,
        );

        // build conditions
        self.push_article_condition(&mut qb, &filter);

        let total = qb
            .build_query_as::<ItemCount>()
            .fetch_one(&self.pool)
            .await?;

        Ok((articles, total))
    }
}
