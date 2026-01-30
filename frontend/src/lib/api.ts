import axios from 'axios';
import type { ChatRequest, ModelInfo, Document, LogQuery, LogResponse } from '@/types';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/api';

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

export const api = {
  // チャット
  async chatCompletion(request: ChatRequest) {
    const response = await apiClient.post('/v1/chat/completions', request);
    return response.data;
  },

  // モデル一覧
  async listModels(): Promise<ModelInfo[]> {
    const response = await apiClient.get('/v1/models');
    return response.data;
  },

  // ドキュメント管理
  async addDocument(doc: Document) {
    const response = await apiClient.post('/v1/documents', doc);
    return response.data;
  },

  // ログ
  async queryLogs(query: LogQuery): Promise<LogResponse> {
    const response = await apiClient.get('/v1/logs', { params: query });
    return response.data;
  },

  // ヘルスチェック
  async healthCheck() {
    const response = await apiClient.get('/health');
    return response.data;
  },
};
