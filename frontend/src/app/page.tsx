'use client';

import { useState } from 'react';
import { MessageSquare, FileText, History } from 'lucide-react';
import ChatInterface from '@/components/ChatInterface';
import ModelSelector from '@/components/ModelSelector';
import DocumentManager from '@/components/DocumentManager';
import LogViewer from '@/components/LogViewer';

type Tab = 'chat' | 'documents' | 'logs';

export default function Home() {
  const [activeTab, setActiveTab] = useState<Tab>('chat');

  return (
    <div className="h-screen flex flex-col bg-gray-50">
      {/* ヘッダー */}
      <header className="bg-white border-b px-6 py-4">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900">LLM Proxy</h1>
          {activeTab === 'chat' && <ModelSelector />}
        </div>
      </header>

      {/* タブナビゲーション */}
      <nav className="bg-white border-b">
        <div className="max-w-7xl mx-auto px-6">
          <div className="flex gap-8">
            <button
              onClick={() => setActiveTab('chat')}
              className={`flex items-center gap-2 py-4 border-b-2 transition-colors ${
                activeTab === 'chat'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-600 hover:text-gray-900'
              }`}
            >
              <MessageSquare className="w-5 h-5" />
              チャット
            </button>
            <button
              onClick={() => setActiveTab('documents')}
              className={`flex items-center gap-2 py-4 border-b-2 transition-colors ${
                activeTab === 'documents'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-600 hover:text-gray-900'
              }`}
            >
              <FileText className="w-5 h-5" />
              ドキュメント
            </button>
            <button
              onClick={() => setActiveTab('logs')}
              className={`flex items-center gap-2 py-4 border-b-2 transition-colors ${
                activeTab === 'logs'
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-600 hover:text-gray-900'
              }`}
            >
              <History className="w-5 h-5" />
              ログ
            </button>
          </div>
        </div>
      </nav>

      {/* メインコンテンツ */}
      <main className="flex-1 overflow-hidden">
        {activeTab === 'chat' && <ChatInterface />}
        {activeTab === 'documents' && <DocumentManager />}
        {activeTab === 'logs' && <LogViewer />}
      </main>
    </div>
  );
}
