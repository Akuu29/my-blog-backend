use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub status: i32,
    // pub user_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct NewArticle {
    pub title: String,
    pub body: String,
    pub status: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticle {
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<i32>,
}
