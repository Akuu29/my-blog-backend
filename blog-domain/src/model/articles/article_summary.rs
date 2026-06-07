use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Local};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum SummaryLanguage {
    En,
    Ja,
    Zh,
}

impl fmt::Display for SummaryLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SummaryLanguage::En => "en",
            SummaryLanguage::Ja => "ja",
            SummaryLanguage::Zh => "zh",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ArticleSummary {
    pub article_id: Uuid,
    pub content: String,
    pub lang: SummaryLanguage,
    pub locale: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub struct NewArticleSummary {
    pub content: String,
    pub lang: SummaryLanguage,
    pub locale: Option<String>,
}
