use serde::{Deserialize, Serialize};
use sqlx::{
    types::chrono::{DateTime, Local},
    FromRow,
};

#[derive(Debug, Serialize, FromRow)]
pub struct ArticleTag {
    #[serde(rename = "articleId")]
    pub article_id: i32,
    #[serde(rename = "tagId")]
    pub tag_id: i32,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Local>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Deserialize)]
pub struct ArticleAttachedTags {
    #[serde(rename = "articleId")]
    pub article_id: i32,
    #[serde(rename = "tagIds")]
    pub tag_ids: Vec<i32>,
}

// #[derive(Debug, Deserialize)]
// pub struct NewArticleTag {
//     pub article_id: i32,
//     pub tag_id: i32,
// }

// #[derive(Debug, Deserialize)]
// pub struct Article {
//     pub tags: Vec<NewArticleTag>,
// }
