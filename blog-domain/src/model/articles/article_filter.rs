use serde::Deserialize;
use sqlx::types::Uuid;
#[derive(Default, Deserialize)]
pub struct ArticleFilter {
    pub user_id: Option<Uuid>,
}
