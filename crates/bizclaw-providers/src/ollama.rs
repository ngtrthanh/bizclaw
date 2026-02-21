//! Ollama provider implementation — local/remote Ollama server.

use async_trait::async_trait;
use bizclaw_core::config::BizClawConfig;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::provider::{GenerateParams, Provider};
use bizclaw_core::types::{Message, ModelInfo, ProviderResponse, ToolDefinition};

pub struct OllamaProvider {
    api_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(config: &BizClawConfig) -> Result<Self> {
        let api_url = std::env::var("OLLAMA_HOST")
            .unwrap_or_else(|_| "http://localhost:11434".into());

        let _ = config; // Config may be used later for additional settings

        Ok(Self {
            api_url,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str { "ollama" }

    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        params: &GenerateParams,
    ) -> Result<ProviderResponse> {
        // Ollama uses OpenAI-compatible /api/chat endpoint
        let formatted_messages: Vec<serde_json::Value> = messages.iter().map(|m| {
            serde_json::json!({
                "role": m.role.to_string(),
                "content": m.content,
            })
        }).collect();

        let model = if params.model.is_empty() {
            "llama3.2"
        } else {
            &params.model
        };

        let mut body = serde_json::json!({
            "model": model,
            "messages": formatted_messages,
            "stream": false,
            "options": {
                "temperature": params.temperature,
                "top_p": params.top_p,
                "num_predict": params.max_tokens,
            }
        });

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
            .post(format!("{}/api/chat", self.api_url))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Http(format!("Ollama connection failed ({}): {}", self.api_url, e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BizClawError::Provider(format!("Ollama API error {status}: {text}")));
        }

        let json: serde_json::Value = resp.json().await
            .map_err(|e| BizClawError::Http(e.to_string()))?;

        let content = json["message"]["content"].as_str().map(String::from);

        // Parse tool calls if present
        let tool_calls = if let Some(tc_array) = json["message"]["tool_calls"].as_array() {
            tc_array.iter().filter_map(|tc| {
                let func = &tc["function"];
                Some(bizclaw_core::types::ToolCall {
                    id: uuid::Uuid::new_v4().to_string(),
                    r#type: "function".to_string(),
                    function: bizclaw_core::types::FunctionCall {
                        name: func["name"].as_str()?.to_string(),
                        arguments: func["arguments"].to_string(),
                    },
                })
            }).collect()
        } else {
            vec![]
        };

        // Parse token usage
        let usage = Some(bizclaw_core::types::Usage {
            prompt_tokens: json["prompt_eval_count"].as_u64().unwrap_or(0) as u32,
            completion_tokens: json["eval_count"].as_u64().unwrap_or(0) as u32,
            total_tokens: (json["prompt_eval_count"].as_u64().unwrap_or(0)
                + json["eval_count"].as_u64().unwrap_or(0)) as u32,
        });

        Ok(ProviderResponse {
            content,
            tool_calls,
            finish_reason: Some("stop".into()),
            usage,
        })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Call Ollama's /api/tags endpoint to list installed models
        let resp = self.client
            .get(format!("{}/api/tags", self.api_url))
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await.unwrap_or_default();
                let models = json["models"].as_array()
                    .map(|arr| {
                        arr.iter().filter_map(|m| {
                            Some(ModelInfo {
                                id: m["name"].as_str()?.to_string(),
                                name: m["name"].as_str()?.to_string(),
                                provider: "ollama".into(),
                                context_length: 4096,
                                max_output_tokens: Some(4096),
                            })
                        }).collect()
                    })
                    .unwrap_or_default();
                Ok(models)
            }
            _ => {
                // Ollama not running — return empty
                Ok(vec![])
            }
        }
    }

    async fn health_check(&self) -> Result<bool> {
        let resp = self.client
            .get(format!("{}/api/tags", self.api_url))
            .send()
            .await;
        Ok(resp.is_ok())
    }
}
