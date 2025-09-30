use blog_domain::model::common::pagination::Pagination;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
#[serde(bound(deserialize = ""))]
pub struct PagedFilterQueryParam<T>
where
    T: DeserializeOwned + Validate,
{
    #[serde(flatten)]
    #[validate(nested)]
    pub filter: T,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}
