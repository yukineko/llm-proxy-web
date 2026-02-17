'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import {
  Upload, FileText, RefreshCw, Trash2, Loader2, Settings, AlertCircle,
  FolderPlus, FilePlus, Folder, ChevronRight, X, Check, History,
} from 'lucide-react';
import { api } from '@/lib/api';
import type { DirEntry, IndexStatus, FileVersionHistory } from '@/types';

const INTERVAL_OPTIONS = [
  { label: '15分', value: 15 },
  { label: '30分', value: 30 },
  { label: '1時間', value: 60 },
  { label: '2時間', value: 120 },
  { label: '4時間', value: 240 },
  { label: '8時間', value: 480 },
  { label: '24時間', value: 1440 },
];

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatDate(dateStr: string | null | undefined): string {
  if (!dateStr) return '-';
  const d = new Date(dateStr);
  return d.toLocaleString('ja-JP');
}

function formatBadge(format: string | undefined): string {
  if (!format) return '';
  const map: Record<string, string> = {
    PlainText: 'TXT',
    Pdf: 'PDF',
    Docx: 'DOCX',
    Xlsx: 'XLSX',
    Pptx: 'PPTX',
  };
  return map[format] || format;
}

function badgeColor(format: string | undefined): string {
  if (!format) return '';
  const map: Record<string, string> = {
    PlainText: 'bg-gray-100 text-gray-700',
    Pdf: 'bg-red-100 text-red-700',
    Docx: 'bg-blue-100 text-blue-700',
    Xlsx: 'bg-green-100 text-green-700',
    Pptx: 'bg-orange-100 text-orange-700',
  };
  return map[format] || 'bg-gray-100 text-gray-700';
}

export default function DocumentManager() {
  const [entries, setEntries] = useState<DirEntry[]>([]);
  const [currentPath, setCurrentPath] = useState('');
  const [status, setStatus] = useState<IndexStatus | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const [isUploading, setIsUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [unavailable, setUnavailable] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const pollRef = useRef<NodeJS.Timeout | null>(null);

  // New dir state
  const [showNewDirInput, setShowNewDirInput] = useState(false);
  const [newDirName, setNewDirName] = useState('');

  // New file modal state
  const [showNewFileModal, setShowNewFileModal] = useState(false);
  const [newFileName, setNewFileName] = useState('');
  const [newFileContent, setNewFileContent] = useState('');

  // Version history modal state
  const [showVersionModal, setShowVersionModal] = useState(false);
  const [versionHistory, setVersionHistory] = useState<FileVersionHistory | null>(null);
  const [versionTarget, setVersionTarget] = useState('');
  const [isRollingBack, setIsRollingBack] = useState(false);

  const fetchEntries = useCallback(async (path?: string) => {
    try {
      const p = path !== undefined ? path : currentPath;
      const data = await api.listDirEntries(p || undefined);
      setEntries(data);
      setUnavailable(false);
    } catch (e: any) {
      if (e?.response?.status === 503) {
        setUnavailable(true);
      }
    }
  }, [currentPath]);

  const fetchStatus = useCallback(async () => {
    try {
      const data = await api.getIndexStatus();
      setStatus(data);
      setUnavailable(false);
    } catch (e: any) {
      if (e?.response?.status === 503) {
        setUnavailable(true);
      }
    }
  }, []);

  // Initial fetch
  useEffect(() => {
    fetchEntries('');
    fetchStatus();
  }, [fetchStatus]);

  // Re-fetch entries when path changes
  useEffect(() => {
    fetchEntries();
  }, [currentPath, fetchEntries]);

  // Polling: fast when indexing, slow otherwise
  useEffect(() => {
    if (pollRef.current) clearInterval(pollRef.current);
    const interval = status?.is_indexing ? 3000 : 30000;
    pollRef.current = setInterval(() => {
      fetchStatus();
      if (status?.is_indexing) fetchEntries();
    }, interval);
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, [status?.is_indexing, fetchStatus, fetchEntries]);

  const navigateTo = (path: string) => {
    setCurrentPath(path);
    setShowNewDirInput(false);
    setNewDirName('');
  };

  const handleUpload = async (selectedFiles: File[]) => {
    if (selectedFiles.length === 0) return;
    setIsUploading(true);
    setError(null);
    try {
      await api.uploadFiles(selectedFiles, currentPath || undefined);
      await fetchEntries();
    } catch (e: any) {
      setError(e?.response?.data || 'アップロードに失敗しました');
    } finally {
      setIsUploading(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    const droppedFiles = Array.from(e.dataTransfer.files);
    handleUpload(droppedFiles);
  };

  const handleFileInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files ? Array.from(e.target.files) : [];
    handleUpload(selected);
    if (fileInputRef.current) fileInputRef.current.value = '';
  };

  const handleDelete = async (entry: DirEntry) => {
    const fullPath = currentPath ? `${currentPath}/${entry.name}` : entry.name;
    const msg = entry.is_dir
      ? `フォルダ「${entry.name}」とその中身をすべて削除しますか？`
      : `${entry.name} を削除しますか？`;
    if (!confirm(msg)) return;
    try {
      await api.deleteEntry(fullPath);
      await fetchEntries();
    } catch {
      setError('削除に失敗しました');
    }
  };

  const handleCreateDir = async () => {
    if (!newDirName.trim()) return;
    setError(null);
    const fullPath = currentPath ? `${currentPath}/${newDirName.trim()}` : newDirName.trim();
    try {
      await api.createDir(fullPath);
      setShowNewDirInput(false);
      setNewDirName('');
      await fetchEntries();
    } catch (e: any) {
      setError(e?.response?.data || 'フォルダの作成に失敗しました');
    }
  };

  const handleCreateFile = async () => {
    if (!newFileName.trim()) return;
    setError(null);
    const fullPath = currentPath ? `${currentPath}/${newFileName.trim()}` : newFileName.trim();
    try {
      await api.createFile(fullPath, newFileContent);
      setShowNewFileModal(false);
      setNewFileName('');
      setNewFileContent('');
      await fetchEntries();
    } catch (e: any) {
      setError(e?.response?.data || 'ファイルの作成に失敗しました');
    }
  };

  const handleTriggerIndex = async () => {
    setError(null);
    try {
      await api.triggerIndex();
      await fetchStatus();
    } catch (e: any) {
      if (e?.response?.status === 409) {
        setError('インデックス作成中です');
      } else {
        setError('インデックス開始に失敗しました');
      }
    }
  };

  const openVersionHistory = async (entry: DirEntry) => {
    const fullPath = currentPath ? `${currentPath}/${entry.name}` : entry.name;
    try {
      const history = await api.getFileVersions(fullPath);
      setVersionHistory(history);
      setVersionTarget(fullPath);
      setShowVersionModal(true);
    } catch {
      setError('バージョン履歴の取得に失敗しました');
    }
  };

  const handleRollback = async (version: number) => {
    if (!confirm(`バージョン ${version} に巻き戻しますか？\n現在の内容は自動的にバージョンとして保存されます。`)) return;
    setIsRollingBack(true);
    try {
      const result = await api.rollbackFile(versionTarget, version, true);
      setShowVersionModal(false);
      setVersionHistory(null);
      await fetchEntries();
      if (result.reindex_triggered) {
        await fetchStatus();
      }
    } catch {
      setError('巻き戻しに失敗しました');
    } finally {
      setIsRollingBack(false);
    }
  };

  const handleIntervalChange = async (minutes: number) => {
    try {
      await api.updateIndexConfig({ auto_index_interval_minutes: minutes });
      await fetchStatus();
    } catch {
      setError('設定の更新に失敗しました');
    }
  };

  // Breadcrumb segments
  const pathSegments = currentPath ? currentPath.split('/') : [];

  if (unavailable) {
    return (
      <div className="max-w-4xl mx-auto p-6">
        <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6 text-center">
          <AlertCircle className="w-8 h-8 text-yellow-500 mx-auto mb-3" />
          <h3 className="text-lg font-semibold text-yellow-800">RAGエンジンが利用できません</h3>
          <p className="text-yellow-700 mt-2">RAGエンジンの初期化に失敗しました。サーバーログを確認してください。</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6 space-y-6 h-full overflow-y-auto">
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-3 flex items-center gap-2 text-red-700">
          <AlertCircle className="w-4 h-4 flex-shrink-0" />
          <span className="text-sm">{error}</span>
          <button onClick={() => setError(null)} className="ml-auto text-red-500 hover:text-red-700">&times;</button>
        </div>
      )}

      {/* ファイルアップロード */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center gap-2 mb-4">
          <Upload className="w-5 h-5 text-blue-500" />
          <h2 className="text-lg font-bold">ファイルアップロード</h2>
          {currentPath && (
            <span className="text-sm text-gray-500 ml-2">({currentPath}/)</span>
          )}
        </div>

        <div
          onDragOver={(e) => { e.preventDefault(); setIsDragOver(true); }}
          onDragEnter={(e) => { e.preventDefault(); setIsDragOver(true); }}
          onDragLeave={() => setIsDragOver(false)}
          onDrop={handleDrop}
          className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors cursor-pointer ${
            isDragOver ? 'border-blue-500 bg-blue-50' : 'border-gray-300 hover:border-gray-400'
          }`}
          onClick={() => fileInputRef.current?.click()}
        >
          {isUploading ? (
            <div className="flex items-center justify-center gap-2 text-gray-500">
              <Loader2 className="w-6 h-6 animate-spin" />
              <span>アップロード中...</span>
            </div>
          ) : (
            <>
              <Upload className="w-10 h-10 text-gray-400 mx-auto mb-2" />
              <p className="text-gray-600">ファイルをドラッグ＆ドロップ、またはクリックして選択</p>
              <p className="text-sm text-gray-400 mt-1">PDF, DOCX, XLSX, PPTX, TXT, MD など</p>
            </>
          )}
          <input
            ref={fileInputRef}
            type="file"
            multiple
            className="hidden"
            accept=".pdf,.docx,.xlsx,.pptx,.txt,.md,.rs,.py,.js,.ts,.json,.yaml,.yml,.toml"
            onChange={handleFileInput}
          />
        </div>
      </div>

      {/* ファイル一覧 */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <FileText className="w-5 h-5 text-blue-500" />
            <h2 className="text-lg font-bold">ファイル一覧</h2>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => { setShowNewDirInput(true); setShowNewFileModal(false); }}
              className="px-3 py-1.5 text-sm bg-gray-100 hover:bg-gray-200 rounded-lg flex items-center gap-1 transition-colors"
              title="新規フォルダ"
            >
              <FolderPlus className="w-4 h-4" />
              <span>新規フォルダ</span>
            </button>
            <button
              onClick={() => { setShowNewFileModal(true); setShowNewDirInput(false); }}
              className="px-3 py-1.5 text-sm bg-gray-100 hover:bg-gray-200 rounded-lg flex items-center gap-1 transition-colors"
              title="新規ファイル"
            >
              <FilePlus className="w-4 h-4" />
              <span>新規ファイル</span>
            </button>
          </div>
        </div>

        {/* Breadcrumb */}
        <div className="flex items-center gap-1 text-sm mb-3 text-gray-600">
          <button
            onClick={() => navigateTo('')}
            className={`hover:text-blue-600 transition-colors ${currentPath === '' ? 'font-semibold text-gray-900' : ''}`}
          >
            uploads
          </button>
          {pathSegments.map((seg, i) => {
            const segPath = pathSegments.slice(0, i + 1).join('/');
            const isLast = i === pathSegments.length - 1;
            return (
              <span key={segPath} className="flex items-center gap-1">
                <ChevronRight className="w-3 h-3 text-gray-400" />
                <button
                  onClick={() => navigateTo(segPath)}
                  className={`hover:text-blue-600 transition-colors ${isLast ? 'font-semibold text-gray-900' : ''}`}
                >
                  {seg}
                </button>
              </span>
            );
          })}
        </div>

        {/* New dir inline input */}
        {showNewDirInput && (
          <div className="flex items-center gap-2 mb-3 p-2 bg-blue-50 rounded-lg">
            <Folder className="w-4 h-4 text-blue-500" />
            <input
              type="text"
              value={newDirName}
              onChange={(e) => setNewDirName(e.target.value)}
              onKeyDown={(e) => { if (e.key === 'Enter') handleCreateDir(); if (e.key === 'Escape') { setShowNewDirInput(false); setNewDirName(''); } }}
              placeholder="フォルダ名を入力..."
              className="flex-1 px-2 py-1 text-sm border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              autoFocus
            />
            <button onClick={handleCreateDir} className="text-green-600 hover:text-green-800"><Check className="w-4 h-4" /></button>
            <button onClick={() => { setShowNewDirInput(false); setNewDirName(''); }} className="text-gray-400 hover:text-gray-600"><X className="w-4 h-4" /></button>
          </div>
        )}

        {entries.length === 0 && !showNewDirInput ? (
          <p className="text-gray-400 text-center py-4">ファイルがありません</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b text-left text-gray-500">
                  <th className="pb-2">名前</th>
                  <th className="pb-2">形式</th>
                  <th className="pb-2">サイズ</th>
                  <th className="pb-2">更新日時</th>
                  <th className="pb-2">履歴</th>
                  <th className="pb-2 w-10"></th>
                </tr>
              </thead>
              <tbody>
                {entries.map((entry) => (
                  <tr key={entry.name} className="border-b last:border-0 hover:bg-gray-50">
                    <td className="py-2">
                      {entry.is_dir ? (
                        <button
                          onClick={() => navigateTo(currentPath ? `${currentPath}/${entry.name}` : entry.name)}
                          className="flex items-center gap-1.5 font-medium text-blue-600 hover:text-blue-800 transition-colors"
                        >
                          <Folder className="w-4 h-4" />
                          {entry.name}
                        </button>
                      ) : (
                        <span className="flex items-center gap-1.5 font-medium">
                          <FileText className="w-4 h-4 text-gray-400" />
                          {entry.name}
                        </span>
                      )}
                    </td>
                    <td className="py-2">
                      {entry.is_dir ? (
                        <span className="px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-700">DIR</span>
                      ) : entry.format ? (
                        <span className={`px-2 py-0.5 rounded text-xs font-medium ${badgeColor(entry.format)}`}>
                          {formatBadge(entry.format)}
                        </span>
                      ) : (
                        <span className="text-gray-400 text-xs">-</span>
                      )}
                    </td>
                    <td className="py-2 text-gray-500">
                      {entry.is_dir ? '-' : entry.size != null ? formatFileSize(entry.size) : '-'}
                    </td>
                    <td className="py-2 text-gray-500">{formatDate(entry.modified_at)}</td>
                    <td className="py-2">
                      {!entry.is_dir ? (
                        <button
                          onClick={() => openVersionHistory(entry)}
                          className="text-gray-400 hover:text-amber-600 transition-colors flex items-center gap-1"
                          title="バージョン履歴"
                        >
                          <History className="w-3.5 h-3.5" />
                          <span className="text-xs">{entry.version_count ? `${entry.version_count}件` : '-'}</span>
                        </button>
                      ) : (
                        <span className="text-gray-300 text-xs">-</span>
                      )}
                    </td>
                    <td className="py-2">
                      <button
                        onClick={() => handleDelete(entry)}
                        className="text-red-400 hover:text-red-600 transition-colors"
                        title="削除"
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* 新規ファイルモーダル */}
      {showNewFileModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowNewFileModal(false)}>
          <div className="bg-white rounded-lg shadow-xl p-6 w-full max-w-lg mx-4" onClick={(e) => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-bold flex items-center gap-2">
                <FilePlus className="w-5 h-5 text-blue-500" />
                新規ファイル作成
              </h3>
              <button onClick={() => setShowNewFileModal(false)} className="text-gray-400 hover:text-gray-600">
                <X className="w-5 h-5" />
              </button>
            </div>
            {currentPath && (
              <p className="text-sm text-gray-500 mb-3">保存先: {currentPath}/</p>
            )}
            <div className="space-y-3">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">ファイル名</label>
                <input
                  type="text"
                  value={newFileName}
                  onChange={(e) => setNewFileName(e.target.value)}
                  placeholder="例: memo.txt"
                  className="w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                  autoFocus
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">内容</label>
                <textarea
                  value={newFileContent}
                  onChange={(e) => setNewFileContent(e.target.value)}
                  placeholder="ファイルの内容を入力..."
                  rows={4}
                  className="w-full px-3 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm font-mono max-h-40 overflow-y-auto resize-y"
                />
              </div>
              <div className="flex justify-end gap-2">
                <button
                  onClick={() => { setShowNewFileModal(false); setNewFileName(''); setNewFileContent(''); }}
                  className="px-4 py-2 text-sm text-gray-600 hover:text-gray-800 transition-colors"
                >
                  キャンセル
                </button>
                <button
                  onClick={handleCreateFile}
                  disabled={!newFileName.trim()}
                  className="px-4 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
                >
                  作成
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* バージョン履歴モーダル */}
      {showVersionModal && versionHistory && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowVersionModal(false)}>
          <div className="bg-white rounded-lg shadow-xl p-6 w-full max-w-lg mx-4" onClick={(e) => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-bold flex items-center gap-2">
                <History className="w-5 h-5 text-amber-500" />
                バージョン履歴
              </h3>
              <button onClick={() => setShowVersionModal(false)} className="text-gray-400 hover:text-gray-600">
                <X className="w-5 h-5" />
              </button>
            </div>
            <p className="text-sm text-gray-500 mb-3">{versionTarget}</p>

            {/* 現在のバージョン */}
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-3 mb-4">
              <div className="text-sm font-medium text-blue-700">現在のバージョン</div>
              <div className="text-xs text-blue-600 mt-1">
                サイズ: {formatFileSize(versionHistory.current_size)} | 更新: {formatDate(versionHistory.current_modified_at)}
              </div>
            </div>

            {/* バージョン一覧 */}
            {versionHistory.versions.length === 0 ? (
              <p className="text-gray-400 text-center py-4">バージョン履歴がありません</p>
            ) : (
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {versionHistory.versions.slice().reverse().map((v) => (
                  <div key={v.version} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                    <div>
                      <div className="text-sm font-medium">v{v.version}</div>
                      <div className="text-xs text-gray-500">
                        {formatDate(v.created_at)} | {formatFileSize(v.size)}
                      </div>
                      <div className="text-xs text-gray-400">{v.comment}</div>
                    </div>
                    <button
                      onClick={() => handleRollback(v.version)}
                      disabled={isRollingBack}
                      className="px-3 py-1 text-xs bg-amber-100 text-amber-700 hover:bg-amber-200 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {isRollingBack ? '処理中...' : '巻き戻し'}
                    </button>
                  </div>
                ))}
              </div>
            )}

            <div className="text-xs text-gray-400 mt-3">
              最大10バージョンまで保存されます。巻き戻し後は自動的に再インデックスされます。
            </div>
          </div>
        </div>
      )}

      {/* インデックス管理 */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center gap-2 mb-4">
          <Settings className="w-5 h-5 text-blue-500" />
          <h2 className="text-lg font-bold">インデックス管理</h2>
        </div>

        {status && (
          <div className="space-y-4">
            {/* ステータスカード */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="bg-gray-50 rounded-lg p-3">
                <div className="text-xs text-gray-500">ステータス</div>
                <div className="font-semibold mt-1 flex items-center gap-1">
                  {status.is_indexing ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin text-blue-500" />
                      <span className="text-blue-600">作成中</span>
                    </>
                  ) : (
                    <span className="text-green-600">待機中</span>
                  )}
                </div>
              </div>
              <div className="bg-gray-50 rounded-lg p-3">
                <div className="text-xs text-gray-500">最終実行</div>
                <div className="font-semibold mt-1 text-sm">{status.last_indexed_at ? formatDate(status.last_indexed_at) : '未実行'}</div>
              </div>
              <div className="bg-gray-50 rounded-lg p-3">
                <div className="text-xs text-gray-500">ファイル数</div>
                <div className="font-semibold mt-1">{status.total_files}</div>
              </div>
              <div className="bg-gray-50 rounded-lg p-3">
                <div className="text-xs text-gray-500">チャンク数</div>
                <div className="font-semibold mt-1">{status.total_chunks}</div>
              </div>
            </div>

            {/* インデックスエラー */}
            {status.last_error && (
              <div className="bg-red-50 border border-red-300 rounded-lg p-4">
                <div className="flex items-center gap-2 mb-1">
                  <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0" />
                  <span className="text-sm font-bold text-red-800">インデックス処理エラー</span>
                </div>
                <p className="text-sm text-red-700 ml-7 break-all">{status.last_error}</p>
              </div>
            )}

            {/* 失敗ファイル */}
            {status.failed_files.length > 0 && (
              <div className="bg-red-50 border border-red-200 rounded-lg p-3">
                <div className="text-sm font-medium text-red-700 mb-1">失敗したファイル:</div>
                <ul className="text-sm text-red-600 list-disc list-inside">
                  {status.failed_files.map((f, i) => (
                    <li key={i}>{f}</li>
                  ))}
                </ul>
              </div>
            )}

            {/* アクション */}
            <div className="flex items-center gap-4 flex-wrap">
              <button
                onClick={handleTriggerIndex}
                disabled={status.is_indexing}
                className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
              >
                {status.is_indexing ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <RefreshCw className="w-4 h-4" />
                )}
                {status.is_indexing ? 'インデックス作成中...' : '今すぐインデックス作成'}
              </button>

              <div className="flex items-center gap-2">
                <label className="text-sm text-gray-600">自動実行間隔:</label>
                <select
                  value={status.auto_index_interval_minutes}
                  onChange={(e) => handleIntervalChange(Number(e.target.value))}
                  className="border rounded-lg px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  {INTERVAL_OPTIONS.map((opt) => (
                    <option key={opt.value} value={opt.value}>{opt.label}</option>
                  ))}
                </select>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
