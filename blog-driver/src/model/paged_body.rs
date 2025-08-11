use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedBody<T> {
    items: Vec<T>,
    next_cursor: Option<Uuid>,
}

impl<T> PagedBody<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<Uuid>) -> Self {
        Self { items, next_cursor }
    }
}
