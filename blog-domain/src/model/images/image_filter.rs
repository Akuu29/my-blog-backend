use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct ImageFilter {
    #[serde(rename = "articleId")]
    pub article_id: Option<i32>,
}
