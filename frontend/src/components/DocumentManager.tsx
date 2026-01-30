'use client';

import { useState } from 'react';
import { Plus, FileText } from 'lucide-react';
import { api } from '@/lib/api';

export default function DocumentManager() {
  const [title, setTitle] = useState('');
  const [content, setContent] = useState('');
  const [category, setCategory] = useState('');
  const [isUploading, setIsUploading] = useState(false);

  const handleUpload = async () => {
    if (!title.trim() || !content.trim()) return;

    setIsUploading(true);
    try {
      await api.addDocument({
        title,
        content,
        category: category || undefined,
      });
      
      setTitle('');
      setContent('');
      setCategory('');
      alert('ドキュメントを追加しました');
    } catch (error) {
      console.error('Upload error:', error);
      alert('エラーが発生しました');
    } finally {
      setIsUploading(false);
    }
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center gap-2 mb-6">
          <FileText className="w-6 h-6 text-blue-500" />
          <h2 className="text-2xl font-bold">RAGドキュメント管理</h2>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-2">タイトル</label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="w-full p-3 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="ドキュメントのタイトル"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">カテゴリ（任意）</label>
            <input
              type="text"
              value={category}
              onChange={(e) => setCategory(e.target.value)}
              className="w-full p-3 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="例: 営業資料, 技術文書"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">内容</label>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              className="w-full p-3 border rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
              rows={10}
              placeholder="ドキュメントの内容を入力..."
            />
          </div>

          <button
            onClick={handleUpload}
            disabled={!title.trim() || !content.trim() || isUploading}
            className="w-full px-4 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors flex items-center justify-center gap-2"
          >
            {isUploading ? (
              <>処理中...</>
            ) : (
              <>
                <Plus className="w-5 h-5" />
                ドキュメントを追加
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
