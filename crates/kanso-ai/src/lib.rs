//! AI for Kanso: choosing where a question goes, streaming the answer, and
//! the prompt it is asked with.
//!
//! The provider clients speak directly to the provider the person picked.
//! OpenAI, OpenRouter, Ollama, LM Studio and any other OpenAI-compatible
//! server share one client; Anthropic has its own. Nothing here reads or
//! writes notes, and nothing is logged.

pub mod client;
pub mod prompt;
pub mod sse;

use serde::{Deserialize, Serialize};

pub use client::{AiError, ChatProvider, ClientSettings, Event, client, probe};
pub use prompt::{DEFAULT_BUDGET, Excerpt, Prompt, build as build_prompt};

/// Where questions go.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Provider {
    /// Ollama, or another model on this device or network, through an
    /// OpenAI-compatible server.
    #[default]
    LocalModel,
    OpenAi,
    Anthropic,
    OpenRouter,
    /// Any other OpenAI-compatible server, at a URL the person types.
    Custom,
}

impl Provider {
    pub const ALL: [Provider; 5] = [
        Provider::LocalModel,
        Provider::OpenAi,
        Provider::Anthropic,
        Provider::OpenRouter,
        Provider::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Provider::LocalModel => "Local model",
            Provider::OpenAi => "OpenAI",
            Provider::Anthropic => "Anthropic",
            Provider::OpenRouter => "OpenRouter",
            Provider::Custom => "Custom URL",
        }
    }

    /// The name under `Provider` in a config file.
    pub fn key(self) -> &'static str {
        match self {
            Provider::LocalModel => "local",
            Provider::OpenAi => "openai",
            Provider::Anthropic => "anthropic",
            Provider::OpenRouter => "openrouter",
            Provider::Custom => "custom",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            Provider::LocalModel => "http://localhost:11434/v1",
            Provider::OpenAi => "https://api.openai.com/v1",
            Provider::Anthropic => "https://api.anthropic.com",
            Provider::OpenRouter => "https://openrouter.ai/api/v1",
            Provider::Custom => "http://localhost:1234/v1",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Provider::LocalModel => "llama3.2",
            Provider::OpenAi => "gpt-4o-mini",
            Provider::Anthropic => "claude-haiku-5-5",
            Provider::OpenRouter => "openai/gpt-4o-mini",
            Provider::Custom => "",
        }
    }

    /// Whether the provider needs an API key, which goes in the Keychain.
    pub fn needs_key(self) -> bool {
        matches!(
            self,
            Provider::OpenAi | Provider::Anthropic | Provider::OpenRouter
        )
    }

    /// The server host requests go to, for the line that says where they go.
    pub fn host(self, base_url: &str) -> String {
        let rest = base_url
            .trim()
            .split_once("://")
            .map_or(base_url.trim(), |(_, rest)| rest);
        let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
        if host.is_empty() {
            self.default_base_url()
                .trim()
                .split_once("://")
                .map_or("", |(_, rest)| rest)
                .split('/')
                .next()
                .unwrap_or("")
                .to_owned()
        } else {
            host.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Provider;

    #[test]
    fn providers_name_their_defaults() {
        assert_eq!(Provider::Anthropic.default_model(), "claude-haiku-5-5");
        assert!(Provider::OpenAi.needs_key());
        assert!(!Provider::LocalModel.needs_key());
        assert!(!Provider::Custom.needs_key());
        assert_eq!(
            Provider::OpenAi.host("https://api.openai.com/v1"),
            "api.openai.com"
        );
        assert_eq!(
            Provider::LocalModel.host("http://localhost:11434/v1"),
            "localhost:11434"
        );
        assert_eq!(Provider::OpenAi.host(""), "api.openai.com");
    }
}
