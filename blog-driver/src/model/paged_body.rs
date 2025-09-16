use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedBody<T> {
    items: Vec<T>,
    // prev_cursor: Option<Uuid>,
    next_cursor: Option<Uuid>,
    // has_prev: bool,
    has_next: bool,
    total: i64,
}

impl<T> PagedBody<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<Uuid>, has_next: bool, total: i64) -> Self {
        Self {
            items,
            next_cursor,
            has_next,
            total,
        }
    }
}
