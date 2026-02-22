//! DeepSeek provider — uses OpenAI-compatible API.

use async_trait::async_trait;
use bizclaw_core::config::BizClawConfig;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::provider::{GenerateParams, Provider};
use bizclaw_core::types::{Message, ModelInfo, ProviderResponse, ToolDefinition};

pub struct DeepSeekProvider {
    api_key: String,
    client: reqwest::Client,
}

impl DeepSeekProvider {
    pub fn new(config: &BizClawConfig) -> Result<Self> {
        let api_key = if config.api_key.is_empty() {
            std::env::var("DEEPSEEK_API_KEY").unwrap_or_default()
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
impl Provider for DeepSeekProvider {
    fn name(&self) -> &str {
        "deepseek"
    }

    async fn chat(
        &self,
        messages: &[Message],
        _tools: &[ToolDefinition],
        params: &GenerateParams,
    ) -> Result<ProviderResponse> {
        if self.api_key.is_empty() {
            return Err(BizClawError::ApiKeyMissing("deepseek".into()));
        }

        let body = serde_json::json!({"model": params.model, "messages": messages, "temperature": params.temperature, "max_tokens": params.max_tokens});
        let resp = self
            .client
            .post("https://api.deepseek.com/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Provider(format!("DeepSeek error: {e}")))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| BizClawError::Provider(format!("Read: {e}")))?;
        if !status.is_success() {
            return Err(BizClawError::Provider(format!("DeepSeek {status}: {text}")));
        }
        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| BizClawError::Provider(format!("JSON: {e}")))?;

        Ok(ProviderResponse {
            content: json["choices"][0]["message"]["content"]
                .as_str()
                .map(String::from),
            tool_calls: vec![],
            finish_reason: Some("stop".into()),
            usage: None,
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                id: "deepseek-chat".into(),
                name: "DeepSeek Chat".into(),
                provider: "deepseek".into(),
                context_length: 128000,
                max_output_tokens: Some(8192),
            },
            ModelInfo {
                id: "deepseek-reasoner".into(),
                name: "DeepSeek R1".into(),
                provider: "deepseek".into(),
                context_length: 64000,
                max_output_tokens: Some(8192),
            },
        ])
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(!self.api_key.is_empty())
    }
}
