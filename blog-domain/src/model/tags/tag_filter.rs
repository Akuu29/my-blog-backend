use serde::Deserialize;
use sqlx::types::Uuid;

#[derive(Default, Deserialize)]
pub struct TagFilter {
    pub user_id: Option<Uuid>,
    pub tag_ids: Option<Vec<i32>>,
}
