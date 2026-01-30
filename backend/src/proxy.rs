use anyhow::Result;
use reqwest::Client;
use crate::models::{ChatRequest, ChatResponse};

pub struct LiteLLMProxy {
    client: Client,
    base_url: String,
}

impl LiteLLMProxy {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            anyhow::bail!("LiteLLM request failed: {} - {}", status, error_text);
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response)
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
