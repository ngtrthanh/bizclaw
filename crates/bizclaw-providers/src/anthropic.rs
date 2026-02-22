//! Anthropic Claude provider implementation.

use async_trait::async_trait;
use bizclaw_core::config::BizClawConfig;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::provider::{GenerateParams, Provider};
use bizclaw_core::types::{Message, ModelInfo, ProviderResponse, Role, ToolDefinition};

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(config: &BizClawConfig) -> Result<Self> {
        let api_key = if config.api_key.is_empty() {
            std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()
        } else {
            config.api_key.clone()
        };

        Ok(Self {
            api_key,
            client: reqwest::Client::new(),
        })
    }

    /// Convert messages to Anthropic format.
    /// Anthropic uses a separate `system` parameter and `messages` array.
    fn format_messages(messages: &[Message]) -> (Option<String>, Vec<serde_json::Value>) {
        let mut system_prompt = None;
        let mut formatted = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    system_prompt = Some(msg.content.clone());
                }
                Role::User => {
                    formatted.push(serde_json::json!({
                        "role": "user",
                        "content": msg.content,
                    }));
                }
                Role::Assistant => {
                    formatted.push(serde_json::json!({
                        "role": "assistant",
                        "content": msg.content,
                    }));
                }
                Role::Tool => {
                    formatted.push(serde_json::json!({
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": msg.tool_call_id.as_deref().unwrap_or(""),
                            "content": msg.content,
                        }]
                    }));
                }
            }
        }

        (system_prompt, formatted)
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        params: &GenerateParams,
    ) -> Result<ProviderResponse> {
        if self.api_key.is_empty() {
            return Err(BizClawError::ApiKeyMissing("anthropic".into()));
        }

        let (system_prompt, formatted_messages) = Self::format_messages(messages);

        let model = if params.model.is_empty() {
            "claude-sonnet-4-20250514"
        } else {
            &params.model
        };

        let mut body = serde_json::json!({
            "model": model,
            "messages": formatted_messages,
            "max_tokens": params.max_tokens,
            "temperature": params.temperature,
        });

        if let Some(sys) = &system_prompt {
            body["system"] = serde_json::Value::String(sys.clone());
        }

        if !tools.is_empty() {
            let tool_defs: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.parameters,
                    })
                })
                .collect();
            body["tools"] = serde_json::Value::Array(tool_defs);
        }

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BizClawError::Provider(format!(
                "Anthropic API error {status}: {text}"
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BizClawError::Http(e.to_string()))?;

        // Parse Anthropic response format
        let mut content_text = String::new();
        let mut tool_calls = Vec::new();

        if let Some(content_blocks) = json["content"].as_array() {
            for block in content_blocks {
                match block["type"].as_str() {
                    Some("text") => {
                        if let Some(text) = block["text"].as_str() {
                            content_text.push_str(text);
                        }
                    }
                    Some("tool_use") => {
                        if let (Some(id), Some(name)) =
                            (block["id"].as_str(), block["name"].as_str())
                        {
                            tool_calls.push(bizclaw_core::types::ToolCall {
                                id: id.to_string(),
                                r#type: "function".to_string(),
                                function: bizclaw_core::types::FunctionCall {
                                    name: name.to_string(),
                                    arguments: block["input"].to_string(),
                                },
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        let usage = json["usage"]
            .as_object()
            .map(|u| bizclaw_core::types::Usage {
                prompt_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                completion_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                    as u32,
                total_tokens: (u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                    + u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0))
                    as u32,
            });

        Ok(ProviderResponse {
            content: if content_text.is_empty() {
                None
            } else {
                Some(content_text)
            },
            tool_calls,
            finish_reason: json["stop_reason"].as_str().map(String::from),
            usage,
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                id: "claude-sonnet-4-20250514".into(),
                name: "Claude Sonnet 4".into(),
                provider: "anthropic".into(),
                context_length: 200000,
                max_output_tokens: Some(8192),
            },
            ModelInfo {
                id: "claude-3-5-haiku-20241022".into(),
                name: "Claude 3.5 Haiku".into(),
                provider: "anthropic".into(),
                context_length: 200000,
                max_output_tokens: Some(8192),
            },
            ModelInfo {
                id: "claude-3-5-sonnet-20241022".into(),
                name: "Claude 3.5 Sonnet".into(),
                provider: "anthropic".into(),
                context_length: 200000,
                max_output_tokens: Some(8192),
            },
        ])
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(!self.api_key.is_empty())
    }
}
