mod models;
mod filters;
mod rag;
mod proxy;
mod logger;

use axum::{
    Router,
    routing::{get, post},
    extract::{State, Query},
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;

use models::{
    ChatRequest, ChatResponse, ModelInfo, DocumentUpload,
    LogQuery, LogResponse, LogEntry,
};
use filters::pii_detector::PIIDetector;
use rag::RAGEngine;
use proxy::LiteLLMProxy;
use logger::Logger;

struct AppState {
    pii_detector: Mutex<PIIDetector>,
    rag_engine: Option<RAGEngine>,
    litellm_proxy: LiteLLMProxy,
    logger: Logger,
}

#[tokio::main]
async fn main() -> Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt::init();

    // 環境変数読み込み
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://llmproxy:password@localhost/llm_proxy".to_string());
    let qdrant_url = std::env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    let litellm_url = std::env::var("LITELLM_URL")
        .unwrap_or_else(|_| "http://localhost:4000".to_string());

    tracing::info!("Connecting to database: {}", database_url);
    tracing::info!("Connecting to Qdrant: {}", qdrant_url);
    tracing::info!("Connecting to LiteLLM: {}", litellm_url);

    // コンポーネント初期化
    let logger = Logger::new(&database_url).await?;
    logger.init_schema().await?;

    let rag_engine = match RAGEngine::new(&qdrant_url, "documents").await {
        Ok(engine) => {
            tracing::info!("RAG engine initialized successfully");
            Some(engine)
        }
        Err(e) => {
            tracing::warn!("RAG engine initialization failed (continuing without RAG): {}", e);
            None
        }
    };
    let litellm_proxy = LiteLLMProxy::new(litellm_url);

    let state = Arc::new(AppState {
        pii_detector: Mutex::new(PIIDetector::new()),
        rag_engine,
        litellm_proxy,
        logger,
    });

    // CORS設定
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any);

    // ルーター設定
    let app = Router::new()
        .route("/api/v1/chat/completions", post(chat_completion_handler))
        .route("/api/v1/models", get(list_models_handler))
        .route("/api/v1/documents", post(add_document_handler))
        .route("/api/v1/logs", get(query_logs_handler))
        .route("/api/health", get(health_check))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Backend server listening on {}", listener.local_addr()?);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn chat_completion_handler(
    State(state): State<Arc<AppState>>,
    Json(mut request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let request_id = Uuid::new_v4();
    
    // 最後のユーザーメッセージを取得
    let user_message = request.messages.iter()
        .filter(|m| m.role == "user")
        .last()
        .ok_or((StatusCode::BAD_REQUEST, "No user message found".to_string()))?;

    let original_content = user_message.content.clone();

    // 1. Input Filter: PII検出とマスキング
    let (masked_content, mappings) = {
        let mut detector = state.pii_detector.lock().await;
        detector.detect_and_mask(&original_content)
    };

    tracing::info!("Masked {} PII entities for request {}", mappings.len(), request_id);

    // 2. RAG: コンテキスト検索
    let rag_context = if let Some(ref rag_engine) = state.rag_engine {
        rag_engine
            .retrieve_context(&masked_content, 3)
            .await
            .map_err(|e| {
                tracing::error!("RAG error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("RAG error: {}", e))
            })?
    } else {
        String::new()
    };

    // RAGコンテキストをプロンプトに追加
    let enhanced_content = if !rag_context.is_empty() {
        format!("{}{}", rag_context, masked_content)
    } else {
        masked_content.clone()
    };

    // メッセージを更新
    if let Some(last_msg) = request.messages.iter_mut()
        .filter(|m| m.role == "user")
        .last() {
        last_msg.content = enhanced_content.clone();
    }

    // 3. LiteLLMにリクエスト
    let llm_response = state.litellm_proxy
        .chat_completion(request)
        .await
        .map_err(|e| {
            tracing::error!("LiteLLM error: {}", e);
            (StatusCode::BAD_GATEWAY, format!("LiteLLM error: {}", e))
        })?;

    // 4. Output Filter: マスキング解除
    let mut final_response = llm_response.clone();
    if let Some(choice) = final_response.choices.first_mut() {
        let detector = state.pii_detector.lock().await;
        choice.message.content = detector.unmask(&choice.message.content, &mappings);
    }

    // 5. ログ記録
    let log_entry = LogEntry {
        id: request_id,
        timestamp: Utc::now(),
        original_input: original_content,
        masked_input: masked_content,
        rag_context: if rag_context.is_empty() { None } else { Some(rag_context) },
        llm_output: llm_response.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default(),
        final_output: final_response.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default(),
        pii_mappings: serde_json::to_value(&mappings).unwrap(),
    };

    state.logger.log_request(log_entry)
        .await
        .map_err(|e| {
            tracing::error!("Logging error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Logging error: {}", e))
        })?;

    Ok(Json(final_response))
}

async fn list_models_handler() -> Json<Vec<ModelInfo>> {
    Json(vec![
        ModelInfo {
            id: "claude-3-5-sonnet-20241022".to_string(),
            name: "Claude 3.5 Sonnet".to_string(),
            provider: "Anthropic".to_string(),
            description: "最新のClaude 3.5 Sonnet - 高性能バランス型".to_string(),
        },
        ModelInfo {
            id: "claude-3-opus-20240229".to_string(),
            name: "Claude 3 Opus".to_string(),
            provider: "Anthropic".to_string(),
            description: "最も高性能なClaudeモデル".to_string(),
        },
        ModelInfo {
            id: "claude-3-haiku-20240307".to_string(),
            name: "Claude 3 Haiku".to_string(),
            provider: "Anthropic".to_string(),
            description: "高速・低コストなClaudeモデル".to_string(),
        },
        ModelInfo {
            id: "gpt-4-turbo-preview".to_string(),
            name: "GPT-4 Turbo".to_string(),
            provider: "OpenAI".to_string(),
            description: "最新のGPT-4".to_string(),
        },
        ModelInfo {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: "OpenAI".to_string(),
            description: "OpenAIの最高性能モデル".to_string(),
        },
        ModelInfo {
            id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            provider: "OpenAI".to_string(),
            description: "高速で低コスト".to_string(),
        },
    ])
}

async fn add_document_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DocumentUpload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let id = payload.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let metadata = serde_json::json!({
        "title": payload.title,
        "category": payload.category,
    });

    if let Some(ref rag_engine) = state.rag_engine {
        rag_engine
            .add_document(&id, &payload.content, metadata)
            .await
            .map_err(|e| {
                tracing::error!("RAG document add error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("RAG error: {}", e))
            })?;
    } else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()));
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "id": id
    })))
}

async fn query_logs_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LogQuery>,
) -> Result<Json<LogResponse>, (StatusCode, String)> {
    let response = state.logger
        .query_logs(query)
        .await
        .map_err(|e| {
            tracing::error!("Query logs error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {}", e))
        })?;
    
    Ok(Json(response))
}

async fn health_check(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let litellm_healthy = state.litellm_proxy.health_check().await.unwrap_or(false);
    
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339(),
        "services": {
            "litellm": litellm_healthy
        }
    }))
}
