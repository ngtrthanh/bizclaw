//! # BizClaw Providers
//!
//! LLM provider implementations: OpenAI, Anthropic, Ollama, LlamaCpp, Brain, Gemini, DeepSeek, Groq.

pub mod anthropic;
pub mod brain;
pub mod custom;
pub mod deepseek;
pub mod gemini;
pub mod groq;
pub mod llamacpp;
pub mod ollama;
pub mod openai;

use bizclaw_core::config::BizClawConfig;
use bizclaw_core::error::Result;
use bizclaw_core::traits::Provider;

/// Create a provider from configuration.
pub fn create_provider(config: &BizClawConfig) -> Result<Box<dyn Provider>> {
    match config.default_provider.as_str() {
        "openai" | "openrouter" => Ok(Box::new(openai::OpenAiProvider::new(config)?)),
        "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::new(config)?)),
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(config)?)),
        "llamacpp" | "llama.cpp" => Ok(Box::new(llamacpp::LlamaCppProvider::new(config)?)),
        "brain" => Ok(Box::new(brain::BrainProvider::new(config)?)),
        "gemini" | "google" => Ok(Box::new(gemini::GeminiProvider::new(config)?)),
        "deepseek" => Ok(Box::new(deepseek::DeepSeekProvider::new(config)?)),
        "groq" => Ok(Box::new(groq::GroqProvider::new(config)?)),
        other if other.starts_with("custom:") => {
            Ok(Box::new(custom::CustomProvider::new(config, other)?))
        }
        other => Err(bizclaw_core::error::BizClawError::ProviderNotFound(
            other.into(),
        )),
    }
}

/// List all available provider names.
pub fn available_providers() -> Vec<&'static str> {
    vec![
        "openai",
        "anthropic",
        "ollama",
        "llamacpp",
        "brain",
        "gemini",
        "deepseek",
        "groq",
        "openrouter",
        "custom",
    ]
}
