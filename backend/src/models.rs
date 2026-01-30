use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUpload {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub search_term: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub original_input: String,
    pub masked_input: String,
    pub rag_context: Option<String>,
    pub llm_output: String,
    pub final_output: String,
    pub pii_mappings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogResponse {
    pub logs: Vec<LogEntry>,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub struct MaskingContext {
    pub request_id: Uuid,
    pub mappings: HashMap<String, String>,
    pub original_prompt: String,
    pub masked_prompt: String,
    pub rag_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}
