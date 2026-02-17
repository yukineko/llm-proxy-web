use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::{State, Query, Multipart, Path},
    Json,
    http::StatusCode,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;

use llm_proxy::models::{
    ChatRequest, ChatResponse, ModelInfo, DocumentUpload,
    LogQuery, LogResponse, LogEntry,
    IndexStatusResponse, IndexConfigUpdate, UploadResponse,
    DirEntry, CreateDirRequest, CreateFileRequest, ListFilesQuery,
    FileVersionHistory, RollbackRequest, RollbackResponse,
};
use llm_proxy::filters::pii_detector::PIIDetector;
use llm_proxy::filters::output_sanitizer::OutputSanitizer;
use llm_proxy::rag::RAGEngine;
use llm_proxy::rag::index_manager::IndexManager;
use llm_proxy::proxy::LiteLLMProxy;
use llm_proxy::logger::Logger;
use llm_proxy::indexer::walker::SupportedFormat;
use llm_proxy::rag::versioning;

struct AppState {
    pii_detector: Mutex<PIIDetector>,
    rag_engine: Option<RAGEngine>,
    index_manager: Option<Arc<IndexManager>>,
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
    let upload_dir = std::env::var("UPLOAD_DIR")
        .unwrap_or_else(|_| "./uploads".to_string());

    tracing::info!("Connecting to database: {}", database_url);
    tracing::info!("Connecting to Qdrant: {}", qdrant_url);
    tracing::info!("Connecting to LiteLLM: {}", litellm_url);
    tracing::info!("Upload directory: {}", upload_dir);

    // アップロードディレクトリ作成
    let upload_path = PathBuf::from(&upload_dir);
    std::fs::create_dir_all(&upload_path)?;

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

    // IndexManager初期化
    let index_manager = if let Some(ref engine) = rag_engine {
        let manager = Arc::new(IndexManager::new(
            upload_path,
            engine.embeddings.clone(),
            engine.vector_store.clone(),
            60,
        ));
        IndexManager::start_scheduler(manager.clone());
        tracing::info!("Index manager initialized with 60-minute auto-index");
        Some(manager)
    } else {
        tracing::warn!("Index manager not available (RAG engine not initialized)");
        None
    };

    let litellm_api_key = std::env::var("LITELLM_API_KEY").ok();
    let litellm_proxy = LiteLLMProxy::new(litellm_url, litellm_api_key);

    let state = Arc::new(AppState {
        pii_detector: Mutex::new(PIIDetector::new()),
        rag_engine,
        index_manager,
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
        .route("/api/v1/rag/upload", post(rag_upload_handler))
        .route("/api/v1/rag/files", get(rag_list_files_handler))
        .route("/api/v1/rag/files/{filename}", delete(rag_delete_file_handler))
        .route("/api/v1/rag/mkdir", post(rag_mkdir_handler))
        .route("/api/v1/rag/files/create", post(rag_create_file_handler))
        .route("/api/v1/rag/files/{path}/versions", get(rag_file_versions_handler))
        .route("/api/v1/rag/files/{path}/rollback", post(rag_file_rollback_handler))
        .route("/api/v1/rag/index", post(rag_trigger_index_handler))
        .route("/api/v1/rag/status", get(rag_status_handler))
        .route("/api/v1/rag/config", put(rag_config_handler))
        .route("/api/health", get(health_check))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Backend server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

// ===== Chat Handlers =====

async fn chat_completion_handler(
    State(state): State<Arc<AppState>>,
    Json(mut request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let request_id = Uuid::new_v4();

    let user_message = request.messages.iter()
        .filter(|m| m.role == "user")
        .last()
        .ok_or((StatusCode::BAD_REQUEST, "No user message found".to_string()))?;

    let original_content = user_message.content.clone();

    // ① RAG検索（生テキストで検索 → 精度を維持）
    let rag_context = if let Some(ref rag_engine) = state.rag_engine {
        rag_engine
            .retrieve_context(&original_content, 3)
            .await
            .map_err(|e| {
                tracing::error!("RAG error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("RAG error: {}", e))
            })?
    } else {
        String::new()
    };

    // ② Input Filter: PII置換（入力テキスト + RAGコンテキスト両方をマスク）
    let text_to_mask = if !rag_context.is_empty() {
        format!("{}{}", rag_context, original_content)
    } else {
        original_content.clone()
    };

    let (masked_content, mappings) = {
        let mut detector = state.pii_detector.lock().await;
        detector.detect_and_mask(&text_to_mask)
    };

    tracing::info!("Masked {} PII entities for request {}", mappings.len(), request_id);

    // マスク済みテキストでLLMに送信
    if let Some(last_msg) = request.messages.iter_mut()
        .filter(|m| m.role == "user")
        .last() {
        last_msg.content = masked_content.clone();
    }

    // ③ LLM呼び出し
    let llm_response = state.litellm_proxy
        .chat_completion(request)
        .await
        .map_err(|e| {
            tracing::error!("LiteLLM error: {}", e);
            (StatusCode::BAD_GATEWAY, format!("LiteLLM error: {}", e))
        })?;

    // ④ Output Filter: PII復元（架空名→実名）
    let mut final_response = llm_response.clone();
    if let Some(choice) = final_response.choices.first_mut() {
        let detector = state.pii_detector.lock().await;
        choice.message.content = detector.unmask(&choice.message.content, &mappings);
    }

    // ⑤ Output Filter: 危険コマンド除去
    if let Some(choice) = final_response.choices.first_mut() {
        let (sanitized, removed) = OutputSanitizer::sanitize(&choice.message.content);
        if !removed.is_empty() {
            tracing::warn!("Removed {} dangerous patterns from response {}: {:?}",
                removed.len(), request_id, removed);
        }
        choice.message.content = sanitized;
    }

    // ⑥ ログ保存
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
            id: "claude-opus-4-6".to_string(),
            name: "Claude Opus 4.6".to_string(),
            provider: "Anthropic".to_string(),
            description: "最も高性能なClaudeモデル".to_string(),
        },
        ModelInfo {
            id: "claude-sonnet-4-5".to_string(),
            name: "Claude Sonnet 4.5".to_string(),
            provider: "Anthropic".to_string(),
            description: "高性能バランス型".to_string(),
        },
        ModelInfo {
            id: "claude-haiku-4-5".to_string(),
            name: "Claude Haiku 4.5".to_string(),
            provider: "Anthropic".to_string(),
            description: "高速・低コスト".to_string(),
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

// ===== RAG Management Handlers =====

async fn rag_upload_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListFilesQuery>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    let relative = query.path.as_deref().unwrap_or("");
    let upload_dir = manager.safe_resolve(relative)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e))
    })? {
        let file_name = field.file_name()
            .ok_or((StatusCode::BAD_REQUEST, "Missing file name".to_string()))?
            .to_string();

        // Validate extension
        let ext = std::path::Path::new(&file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if SupportedFormat::from_extension(ext).is_none() {
            return Err((StatusCode::BAD_REQUEST, format!("Unsupported file type: .{}", ext)));
        }

        let data = field.bytes().await.map_err(|e| {
            (StatusCode::BAD_REQUEST, format!("Failed to read file data: {}", e))
        })?;

        let dest = upload_dir.join(&file_name);

        // Auto-version existing file before overwrite
        if dest.exists() && dest.is_file() {
            if let Err(e) = versioning::save_version(&dest, "Auto-saved before upload overwrite") {
                tracing::warn!("Failed to save version before overwrite: {}", e);
            }
        }

        std::fs::write(&dest, &data).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save file: {}", e))
        })?;

        uploaded_files.push(file_name);
    }

    let total_files = manager.list_files().len();

    Ok(Json(UploadResponse {
        uploaded_files,
        total_files_in_dir: total_files,
    }))
}

async fn rag_list_files_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<Vec<DirEntry>>, (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    let relative = query.path.as_deref().unwrap_or("");
    let entries = manager.list_dir_entries(relative)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(entries))
}

async fn rag_delete_file_handler(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    let target = manager.safe_resolve(&filename)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    if !target.exists() {
        return Err((StatusCode::NOT_FOUND, format!("Not found: {}", filename)));
    }

    if target.is_dir() {
        std::fs::remove_dir_all(&target).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete directory: {}", e))
        })?;
    } else {
        // Clean up versions before deleting the file
        if let Err(e) = versioning::delete_versions(&target) {
            tracing::warn!("Failed to clean up versions: {}", e);
        }
        std::fs::remove_file(&target).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete file: {}", e))
        })?;
    }

    Ok(Json(serde_json::json!({
        "status": "deleted",
        "path": filename
    })))
}

async fn rag_mkdir_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateDirRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    let target = manager.safe_resolve_new(&req.path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if target.exists() {
        return Err((StatusCode::CONFLICT, format!("Already exists: {}", req.path)));
    }

    std::fs::create_dir_all(&target).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create directory: {}", e))
    })?;

    Ok(Json(serde_json::json!({
        "status": "created",
        "path": req.path
    })))
}

async fn rag_create_file_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateFileRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    let target = manager.safe_resolve_new(&req.path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if target.exists() {
        return Err((StatusCode::CONFLICT, format!("Already exists: {}", req.path)));
    }

    std::fs::write(&target, &req.content).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create file: {}", e))
    })?;

    Ok(Json(serde_json::json!({
        "status": "created",
        "path": req.path
    })))
}

async fn rag_file_versions_handler(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Result<Json<FileVersionHistory>, (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    let file_path = manager.safe_resolve(&path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if !file_path.is_file() {
        return Err((StatusCode::BAD_REQUEST, "Not a file".to_string()));
    }

    let history = versioning::get_version_history(&file_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get versions: {}", e)))?;

    Ok(Json(history))
}

async fn rag_file_rollback_handler(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Json(req): Json<RollbackRequest>,
) -> Result<Json<RollbackResponse>, (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    let file_path = manager.safe_resolve(&path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if !file_path.is_file() {
        return Err((StatusCode::BAD_REQUEST, "Not a file".to_string()));
    }

    versioning::rollback_to_version(&file_path, req.version)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Rollback failed: {}", e)))?;

    let mut reindex_triggered = false;
    if req.reindex {
        if !manager.is_indexing().await {
            let manager_clone = manager.clone();
            tokio::spawn(async move {
                if let Err(e) = manager_clone.run_index().await {
                    tracing::error!("Re-index after rollback failed: {}", e);
                }
            });
            reindex_triggered = true;
        }
    }

    Ok(Json(RollbackResponse {
        status: "rolled_back".to_string(),
        rolled_back_to: req.version,
        reindex_triggered,
    }))
}

async fn rag_trigger_index_handler(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    if manager.is_indexing().await {
        return Err((StatusCode::CONFLICT, "Indexing already in progress".to_string()));
    }

    let manager_clone = manager.clone();
    tokio::spawn(async move {
        if let Err(e) = manager_clone.run_index().await {
            tracing::error!("Manual indexing failed: {}", e);
        }
    });

    Ok((StatusCode::ACCEPTED, Json(serde_json::json!({
        "status": "indexing_started"
    }))))
}

async fn rag_status_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<IndexStatusResponse>, (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    let status = manager.get_status().await;

    Ok(Json(IndexStatusResponse {
        is_indexing: status.is_indexing,
        last_indexed_at: status.last_indexed_at,
        total_files: status.total_files,
        total_chunks: status.total_chunks,
        failed_files: status.failed_files,
        auto_index_interval_minutes: status.auto_index_interval_minutes,
        upload_dir: manager.upload_dir().to_string_lossy().to_string(),
        last_error: status.last_error,
    }))
}

async fn rag_config_handler(
    State(state): State<Arc<AppState>>,
    Json(config): Json<IndexConfigUpdate>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let manager = state.index_manager.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "RAG engine not available".to_string()))?;

    manager.set_interval(config.auto_index_interval_minutes).await;

    Ok(Json(serde_json::json!({
        "status": "updated",
        "auto_index_interval_minutes": config.auto_index_interval_minutes
    })))
}

// ===== Health Check =====

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
