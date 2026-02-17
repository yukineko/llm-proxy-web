export interface Message {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp?: Date;
}

export interface ChatRequest {
  model: string;
  messages: Message[];
  temperature?: number;
  max_tokens?: number;
  stream?: boolean;
}

export interface ChatResponse {
  id: string;
  object: string;
  created: number;
  model: string;
  choices: Choice[];
}

export interface Choice {
  index: number;
  message: Message;
  finish_reason: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  description: string;
}

export interface Document {
  id?: string;
  title: string;
  content: string;
  category?: string;
  created_at?: Date;
}

export interface LogEntry {
  id: string;
  timestamp: Date;
  original_input: string;
  masked_input: string;
  rag_context?: string;
  llm_output: string;
  final_output: string;
  pii_mappings: Record<string, string>;
}

export interface LogQuery {
  start_date?: string;
  end_date?: string;
  search_term?: string;
  limit?: number;
  offset?: number;
}

export interface LogResponse {
  logs: LogEntry[];
  total: number;
}

// RAG Management types

export interface FileInfo {
  name: string;
  size: number;
  format: string;
  modified_at: string;
}

export interface IndexStatus {
  is_indexing: boolean;
  last_indexed_at: string | null;
  total_files: number;
  total_chunks: number;
  failed_files: string[];
  auto_index_interval_minutes: number;
  upload_dir: string;
  last_error: string | null;
}

export interface IndexConfigUpdate {
  auto_index_interval_minutes: number;
}

export interface UploadResponse {
  uploaded_files: string[];
  total_files_in_dir: number;
}

// Directory browsing types

export interface DirEntry {
  name: string;
  is_dir: boolean;
  size?: number;
  format?: string;
  modified_at?: string;
  version_count?: number;
}

export interface CreateDirRequest {
  path: string;
}

export interface CreateFileRequest {
  path: string;
  content: string;
}

// Version management types

export interface VersionEntry {
  version: number;
  created_at: string;
  size: number;
  comment: string;
}

export interface FileVersionHistory {
  file_path: string;
  current_size: number;
  current_modified_at: string;
  versions: VersionEntry[];
}

export interface RollbackRequest {
  version: number;
  reindex: boolean;
}

export interface RollbackResponse {
  status: string;
  rolled_back_to: number;
  reindex_triggered: boolean;
}
