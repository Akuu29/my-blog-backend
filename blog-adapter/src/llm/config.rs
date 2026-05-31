/// Configuration for LLM-based article summary generation.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// API key for the Gemini API.
    pub api_key: String,
    /// Model name to use for generation (e.g. `gemini-2.5-flash-lite`).
    pub model: String,
    /// Base URL of the Gemini API (e.g. `https://generativelanguage.googleapis.com/v1beta`).
    pub base_url: String,
}

impl LlmConfig {
    pub fn from_env() -> Self {
        Self {
            api_key: std::env::var("GEMINI_API_KEY").expect("Undefined GEMINI_API_KEY"),
            model: std::env::var("SUMMARY_MODEL").expect("Undefined SUMMARY_MODEL"),
            base_url: std::env::var("GEMINI_BASE_URL").expect("Undefined GEMINI_BASE_URL"),
        }
    }
}
