use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use uuid::Uuid;
use validator::Validate;

#[serde_as]
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub cursor: Option<Uuid>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_per_page")]
    #[validate(range(min = 1, max = 100, message = "per_page must be between 1 and 100"))]
    pub per_page: i32,
}

fn default_per_page() -> i32 {
    std::env::var("PER_PAGE")
        .unwrap_or("100".to_string())
        .parse::<i32>()
        .unwrap()
}
