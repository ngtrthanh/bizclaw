//! llama.cpp server provider implementation.
//! Connects to a running llama-server via OpenAI-compatible API.

use async_trait::async_trait;
use bizclaw_core::config::BizClawConfig;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::provider::{GenerateParams, Provider};
use bizclaw_core::types::{Message, ModelInfo, ProviderResponse, ToolDefinition};

pub struct LlamaCppProvider {
    api_url: String,
    client: reqwest::Client,
}

impl LlamaCppProvider {
    pub fn new(config: &BizClawConfig) -> Result<Self> {
        let api_url = std::env::var("LLAMACPP_HOST")
            .unwrap_or_else(|_| "http://localhost:8080".into());

        let _ = config;

        Ok(Self {
            api_url,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl Provider for LlamaCppProvider {
    fn name(&self) -> &str { "llamacpp" }

    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        params: &GenerateParams,
    ) -> Result<ProviderResponse> {
        // llama-server supports OpenAI-compatible /v1/chat/completions
        let formatted_messages: Vec<serde_json::Value> = messages.iter().map(|m| {
            serde_json::json!({
                "role": m.role.to_string(),
                "content": m.content,
            })
        }).collect();

        let mut body = serde_json::json!({
            "messages": formatted_messages,
            "temperature": params.temperature,
            "max_tokens": params.max_tokens,
            "top_p": params.top_p,
            "stream": false,
        });

        if !params.stop.is_empty() {
            body["stop"] = serde_json::Value::Array(
                params.stop.iter().map(|s| serde_json::Value::String(s.clone())).collect()
            );
        }

        if !tools.is_empty() {
            let tool_defs: Vec<serde_json::Value> = tools.iter().map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    }
                })
            }).collect();
            body["tools"] = serde_json::Value::Array(tool_defs);
        }

        let resp = self.client
            .post(format!("{}/v1/chat/completions", self.api_url))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Http(format!("llama.cpp connection failed ({}): {}", self.api_url, e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BizClawError::Provider(format!("llama.cpp API error {status}: {text}")));
        }

        let json: serde_json::Value = resp.json().await
            .map_err(|e| BizClawError::Http(e.to_string()))?;

        // Parse OpenAI-compatible response format
        let choice = json["choices"].get(0)
            .ok_or_else(|| BizClawError::Provider("No choices in response".into()))?;

        let content = choice["message"]["content"].as_str().map(String::from);
        let tool_calls = if let Some(tc) = choice["message"]["tool_calls"].as_array() {
            tc.iter().filter_map(|t| {
                Some(bizclaw_core::types::ToolCall {
                    id: t["id"].as_str().unwrap_or("").to_string(),
                    r#type: "function".to_string(),
                    function: bizclaw_core::types::FunctionCall {
                        name: t["function"]["name"].as_str()?.to_string(),
                        arguments: t["function"]["arguments"].as_str()?.to_string(),
                    },
                })
            }).collect()
        } else {
            vec![]
        };

        Ok(ProviderResponse {
            content,
            tool_calls,
            finish_reason: choice["finish_reason"].as_str().map(String::from),
            usage: json["usage"].as_object().map(|u| bizclaw_core::types::Usage {
                prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
                total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
            }),
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // llama-server serves a single model, try /v1/models
        let resp = self.client
            .get(format!("{}/v1/models", self.api_url))
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await.unwrap_or_default();
                let models = json["data"].as_array()
                    .map(|arr| {
                        arr.iter().filter_map(|m| {
                            Some(ModelInfo {
                                id: m["id"].as_str()?.to_string(),
                                name: m["id"].as_str()?.to_string(),
                                provider: "llamacpp".into(),
                                context_length: 4096,
                                max_output_tokens: Some(4096),
                            })
                        }).collect()
                    })
                    .unwrap_or_default();
                Ok(models)
            }
            _ => Ok(vec![
                ModelInfo {
                    id: "local-model".into(),
                    name: "Local llama.cpp Model".into(),
                    provider: "llamacpp".into(),
                    context_length: 4096,
                    max_output_tokens: Some(4096),
                }
            ]),
        }
    }

    async fn health_check(&self) -> Result<bool> {
        let resp = self.client
            .get(format!("{}/health", self.api_url))
            .send()
            .await;
        Ok(resp.is_ok())
    }
}
