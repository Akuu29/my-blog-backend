use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow,
    types::chrono::{DateTime, Local},
};
use std::{fmt, str::FromStr};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[serde(rename_all = "lowercase")]
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
    #[serde(rename = "id")]
    pub public_id: Uuid,
    #[serde(rename = "userId")]
    pub user_public_id: Uuid,
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: ArticleStatus,
    #[serde(rename = "categoryId")]
    pub category_public_id: Option<Uuid>,
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
    #[serde(rename = "categoryId")]
    pub category_public_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticle {
    #[validate(length(min = 1, max = 85, message = "title length must be 1 to 85"))]
    pub title: Option<String>,
    #[validate(length(min = 1, message = "body length mut be 1 or more"))]
    pub body: Option<String>,
    pub status: Option<ArticleStatus>,
    #[serde(rename = "categoryId")]
    pub category_public_id: Option<Uuid>,
}
