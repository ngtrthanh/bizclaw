//! Google Gemini provider — uses OpenAI-compatible endpoint.

use async_trait::async_trait;
use bizclaw_core::config::BizClawConfig;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::provider::{GenerateParams, Provider};
use bizclaw_core::types::{Message, ModelInfo, ProviderResponse, ToolDefinition};

pub struct GeminiProvider {
    api_key: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(config: &BizClawConfig) -> Result<Self> {
        let api_key = if config.api_key.is_empty() {
            std::env::var("GEMINI_API_KEY")
                .or_else(|_| std::env::var("GOOGLE_API_KEY"))
                .unwrap_or_default()
        } else {
            config.api_key.clone()
        };
        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl Provider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    async fn chat(
        &self,
        messages: &[Message],
        _tools: &[ToolDefinition],
        params: &GenerateParams,
    ) -> Result<ProviderResponse> {
        if self.api_key.is_empty() {
            return Err(BizClawError::ApiKeyMissing("gemini".into()));
        }

        let body = serde_json::json!({
            "model": params.model,
            "messages": messages,
            "temperature": params.temperature,
            "max_tokens": params.max_tokens,
        });

        let resp = self
            .client
            .post("https://generativelanguage.googleapis.com/v1beta/openai/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Provider(format!("Gemini error: {e}")))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| BizClawError::Provider(format!("Read error: {e}")))?;

        if !status.is_success() {
            return Err(BizClawError::Provider(format!(
                "Gemini API {status}: {text}"
            )));
        }

        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| BizClawError::Provider(format!("Invalid JSON: {e}")))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .map(String::from);

        Ok(ProviderResponse {
            content,
            tool_calls: vec![],
            finish_reason: Some("stop".into()),
            usage: None,
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                id: "gemini-2.5-pro".into(),
                name: "Gemini 2.5 Pro".into(),
                provider: "gemini".into(),
                context_length: 1048576,
                max_output_tokens: Some(65536),
            },
            ModelInfo {
                id: "gemini-2.5-flash".into(),
                name: "Gemini 2.5 Flash".into(),
                provider: "gemini".into(),
                context_length: 1048576,
                max_output_tokens: Some(65536),
            },
        ])
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(!self.api_key.is_empty())
    }
}
