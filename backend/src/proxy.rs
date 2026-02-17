use anyhow::Result;
use reqwest::Client;
use crate::models::{ChatRequest, ChatResponse};

pub struct LiteLLMProxy {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl LiteLLMProxy {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
        }
    }

    pub async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let mut req = self.client.post(&url).json(&request);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("LiteLLM request failed: {} - {}", status, error_text);
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health/liveliness", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
