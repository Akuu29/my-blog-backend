use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[serde_as]
#[derive(Debug, Clone, Default, Deserialize, Validate)]
#[validate(schema(function = "validate_pagination"))]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub offset: Option<i32>,
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

fn validate_pagination(p: &Pagination) -> Result<(), ValidationError> {
    if p.offset.is_some() && p.cursor.is_some() {
        let mut err = ValidationError::new("offset_and_cursor_conflict");
        err.message = Some("cannot set both offset and cursor".into());
        return Err(err);
    }

    Ok(())
}
