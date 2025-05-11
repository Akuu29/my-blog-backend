use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Local},
    FromRow,
};
use std::{fmt, str::FromStr};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "article_status", rename_all = "lowercase")]
pub enum ArticleStatus {
    Draft,
    Private,
    Published,
    Deleted,
}

impl FromStr for ArticleStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(ArticleStatus::Draft),
            "private" => Ok(ArticleStatus::Private),
            "published" => Ok(ArticleStatus::Published),
            "deleted" => Ok(ArticleStatus::Deleted),
            _ => Err("Invalid article status".to_string()),
        }
    }
}

impl fmt::Display for ArticleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            ArticleStatus::Draft => "draft",
            ArticleStatus::Private => "private",
            ArticleStatus::Published => "published",
            ArticleStatus::Deleted => "deleted",
        };
        write!(f, "{}", value)
    }
}

#[derive(Debug, Clone, Serialize, FromRow, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub id: i32,
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: ArticleStatus,
    pub category_id: Option<i32>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NewArticle {
    #[validate(length(min = 1, max = 85, message = "title length must be 1 to 85"))]
    pub title: Option<String>,
    #[validate(length(min = 1, message = "body length mut be 1 or more"))]
    pub body: Option<String>,
    pub status: ArticleStatus,
    pub category_id: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticle {
    #[validate(length(min = 1, max = 85, message = "title length must be 1 to 85"))]
    pub title: Option<String>,
    #[validate(length(min = 1, message = "body length mut be 1 or more"))]
    pub body: Option<String>,
    pub status: Option<ArticleStatus>,
    pub category_id: Option<i32>,
}
