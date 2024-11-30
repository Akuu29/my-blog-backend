use serde::Serialize;
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct ArticleByCategoryDto {
    pub article_id: i32,
    pub article_title: String,
    pub category_name: String,
}
