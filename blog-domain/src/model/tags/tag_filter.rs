use serde::Deserialize;
use sqlx::types::Uuid;

#[derive(Deserialize)]
pub struct TagFilter {
    pub user_id: Option<Uuid>,
}
