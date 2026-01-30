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
