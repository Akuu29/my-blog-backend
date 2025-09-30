use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ItemCount(i64);

impl ItemCount {
    pub fn new(count: i64) -> Self {
        Self(count)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}
