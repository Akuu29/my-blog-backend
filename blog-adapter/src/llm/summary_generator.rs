use crate::llm::{
    config::LlmConfig,
    gemini_api::{
        BlockReason, Content, FinishReason, GeminiRequest, GeminiResponse, Part, SystemInstruction,
    },
    prompts,
};
use async_trait::async_trait;
use blog_app::llm::{error::SummaryGeneratorError, i_summary_generator::ISummaryGenerator};
use blog_domain::model::articles::article_summary::SummaryLanguage;
use reqwest::StatusCode;

const MAX_SUMMARY_CHARS: usize = 5000;

#[derive(Clone)]
pub struct SummaryGenerator {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl SummaryGenerator {
    pub fn new(config: LlmConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(90))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            api_key: config.api_key,
            model: config.model,
            base_url: config.base_url,
        }
    }

    fn system_instruction(lang: SummaryLanguage) -> &'static str {
        match lang {
            SummaryLanguage::Ja => prompts::SYSTEM_INSTRUCTION_JA,
            SummaryLanguage::En => prompts::SYSTEM_INSTRUCTION_EN,
            SummaryLanguage::Zh => prompts::SYSTEM_INSTRUCTION_ZH,
        }
    }

    async fn call_api(&self, request: GeminiRequest) -> Result<String, SummaryGeneratorError> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    SummaryGeneratorError::Timeout
                } else {
                    SummaryGeneratorError::ApiError(e.to_string())
                }
            })?;

        match response.status() {
            StatusCode::OK => {
                let gemini_response = response
                    .json::<GeminiResponse>()
                    .await
                    .map_err(|e| SummaryGeneratorError::ApiError(e.to_string()))?;

                if let Some(feedback) = gemini_response.prompt_feedback {
                    return match feedback.block_reason {
                        BlockReason::Safety
                        | BlockReason::Blocklist
                        | BlockReason::ProhibitedContent => {
                            Err(SummaryGeneratorError::ContentPolicyViolation)
                        }
                        _ => Err(SummaryGeneratorError::ApiError(
                            "prompt blocked for unspecified reasons".to_string(),
                        )),
                    };
                }

                if let Some(reason) = gemini_response
                    .candidates
                    .first()
                    .and_then(|c| c.finish_reason.as_ref())
                {
                    match reason {
                        FinishReason::Safety
                        | FinishReason::Blocklist
                        | FinishReason::ProhibitedContent => {
                            return Err(SummaryGeneratorError::ContentPolicyViolation);
                        }
                        _ => {}
                    }
                }

                let text = gemini_response
                    .candidates
                    .into_iter()
                    .next()
                    .and_then(|c| c.content)
                    .and_then(|content| content.parts.into_iter().next())
                    .map(|p| p.text)
                    .ok_or_else(|| SummaryGeneratorError::ApiError("empty response".to_string()))?;

                if text.chars().count() > MAX_SUMMARY_CHARS {
                    return Err(SummaryGeneratorError::ApiError(
                        "response exceeds maximum allowed length".to_string(),
                    ));
                }

                Ok(text)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                Err(SummaryGeneratorError::RateLimit { retry_after: None })
            }
            _ => {
                let body_text = response.text().await.unwrap_or_default();
                Err(SummaryGeneratorError::ApiError(body_text))
            }
        }
    }
}

#[async_trait]
impl ISummaryGenerator for SummaryGenerator {
    async fn generate(
        &self,
        title: &str,
        body: &str,
        lang: SummaryLanguage,
        // Use this when detailed settings for specific countries or regions are required.
        // Currently Unsupported.
        _locale: Option<&str>,
    ) -> Result<String, SummaryGeneratorError> {
        let request = GeminiRequest {
            system_instruction: SystemInstruction {
                parts: vec![Part {
                    text: Self::system_instruction(lang).to_string(),
                }],
            },
            contents: vec![Content {
                parts: vec![Part {
                    text: format!("title: {}\n\nbody:\n{}", title, body),
                }],
            }],
        };

        self.call_api(request).await
    }
}
