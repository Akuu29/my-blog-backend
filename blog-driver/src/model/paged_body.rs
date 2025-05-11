use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedBody<T> {
    items: Vec<T>,
    next_cursor: Option<i32>,
}

impl<T> PagedBody<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<i32>) -> Self {
        Self { items, next_cursor }
    }
}
