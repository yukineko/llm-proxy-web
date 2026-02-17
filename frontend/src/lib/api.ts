import axios from 'axios';
import type { ChatRequest, ModelInfo, Document, LogQuery, LogResponse, FileInfo, IndexStatus, IndexConfigUpdate, UploadResponse, DirEntry, CreateDirRequest, CreateFileRequest, FileVersionHistory, RollbackRequest, RollbackResponse } from '@/types';

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

  // RAG管理
  async uploadFiles(files: File[], path?: string): Promise<UploadResponse> {
    const formData = new FormData();
    files.forEach(file => formData.append('files', file));
    const params = path ? { path } : {};
    const response = await apiClient.post('/v1/rag/upload', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
      params,
    });
    return response.data;
  },

  async listDirEntries(path?: string): Promise<DirEntry[]> {
    const params = path ? { path } : {};
    const response = await apiClient.get('/v1/rag/files', { params });
    return response.data;
  },

  async deleteEntry(path: string): Promise<void> {
    await apiClient.delete(`/v1/rag/files/${encodeURIComponent(path)}`);
  },

  async createDir(path: string): Promise<void> {
    await apiClient.post('/v1/rag/mkdir', { path } as CreateDirRequest);
  },

  async createFile(path: string, content: string): Promise<void> {
    await apiClient.post('/v1/rag/files/create', { path, content } as CreateFileRequest);
  },

  async triggerIndex(): Promise<void> {
    await apiClient.post('/v1/rag/index');
  },

  async getIndexStatus(): Promise<IndexStatus> {
    const response = await apiClient.get('/v1/rag/status');
    return response.data;
  },

  async updateIndexConfig(config: IndexConfigUpdate): Promise<void> {
    await apiClient.put('/v1/rag/config', config);
  },

  async getFileVersions(path: string): Promise<FileVersionHistory> {
    const response = await apiClient.get(`/v1/rag/files/${encodeURIComponent(path)}/versions`);
    return response.data;
  },

  async rollbackFile(path: string, version: number, reindex: boolean): Promise<RollbackResponse> {
    const response = await apiClient.post(
      `/v1/rag/files/${encodeURIComponent(path)}/rollback`,
      { version, reindex } as RollbackRequest
    );
    return response.data;
  },
};
