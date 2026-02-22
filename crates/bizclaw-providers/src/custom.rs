//! Custom OpenAI-compatible provider.
//! Connects to any server that implements the OpenAI /v1/chat/completions API.
//! Usage: `default_provider = "custom:https://my-server.com/v1"`

use async_trait::async_trait;
use bizclaw_core::config::BizClawConfig;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::provider::{GenerateParams, Provider};
use bizclaw_core::types::{Message, ModelInfo, ProviderResponse, ToolDefinition};

pub struct CustomProvider {
    api_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl CustomProvider {
    pub fn new(config: &BizClawConfig, endpoint: &str) -> Result<Self> {
        let api_url = endpoint
            .strip_prefix("custom:")
            .unwrap_or(endpoint)
            .to_string();
        let api_key = if config.api_key.is_empty() {
            std::env::var("CUSTOM_API_KEY").unwrap_or_default()
        } else {
            config.api_key.clone()
        };

        Ok(Self {
            api_url,
            api_key,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl Provider for CustomProvider {
    fn name(&self) -> &str {
        "custom"
    }

    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        params: &GenerateParams,
    ) -> Result<ProviderResponse> {
        let mut body = serde_json::json!({
            "model": params.model,
            "messages": messages,
            "temperature": params.temperature,
            "max_tokens": params.max_tokens,
        });

        if !tools.is_empty() {
            let tool_defs: Vec<serde_json::Value> = tools
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters,
                        }
                    })
                })
                .collect();
            body["tools"] = serde_json::Value::Array(tool_defs);
        }

        let mut req = self
            .client
            .post(format!("{}/chat/completions", self.api_url))
            .header("Content-Type", "application/json");

        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = req.json(&body).send().await.map_err(|e| {
            BizClawError::Http(format!(
                "Custom provider connection failed ({}): {}",
                self.api_url, e
            ))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BizClawError::Provider(format!(
                "Custom API error {status}: {text}"
            )));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| BizClawError::Http(e.to_string()))?;

        let choice = json["choices"]
            .get(0)
            .ok_or_else(|| BizClawError::Provider("No choices in response".into()))?;

        let content = choice["message"]["content"].as_str().map(String::from);
        let tool_calls = if let Some(tc) = choice["message"]["tool_calls"].as_array() {
            tc.iter()
                .filter_map(|t| {
                    Some(bizclaw_core::types::ToolCall {
                        id: t["id"].as_str().unwrap_or("").to_string(),
                        r#type: "function".to_string(),
                        function: bizclaw_core::types::FunctionCall {
                            name: t["function"]["name"].as_str()?.to_string(),
                            arguments: t["function"]["arguments"].as_str()?.to_string(),
                        },
                    })
                })
                .collect()
        } else {
            vec![]
        };

        Ok(ProviderResponse {
            content,
            tool_calls,
            finish_reason: choice["finish_reason"].as_str().map(String::from),
            usage: json["usage"]
                .as_object()
                .map(|u| bizclaw_core::types::Usage {
                    prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
                }),
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let resp = self
            .client
            .get(format!("{}/models", self.api_url))
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await.unwrap_or_default();
                let models = json["data"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| {
                                Some(ModelInfo {
                                    id: m["id"].as_str()?.to_string(),
                                    name: m["id"].as_str()?.to_string(),
                                    provider: "custom".into(),
                                    context_length: 4096,
                                    max_output_tokens: Some(4096),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(models)
            }
            _ => Ok(vec![]),
        }
    }

    async fn health_check(&self) -> Result<bool> {
        let resp = self.client.get(&self.api_url).send().await;
        Ok(resp.is_ok())
    }
}
