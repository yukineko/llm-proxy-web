'use client';

import { useState, useRef, useEffect } from 'react';
import { Send, Loader2 } from 'lucide-react';
import { useChatStore } from '@/lib/store';
import { api } from '@/lib/api';
import MessageBubble from './MessageBubble';

export default function ChatInterface() {
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  
  const { 
    messages, 
    selectedModel, 
    isLoading, 
    addMessage, 
    setIsLoading 
  } = useChatStore();

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || isLoading) return;

    const userMessage = {
      role: 'user' as const,
      content: input,
    };

    addMessage(userMessage);
    setInput('');
    setIsLoading(true);

    try {
      const response = await api.chatCompletion({
        model: selectedModel,
        messages: [...messages, userMessage],
      });

      const assistantMessage = {
        role: 'assistant' as const,
        content: response.choices[0].message.content,
      };

      addMessage(assistantMessage);
    } catch (error) {
      console.error('Chat error:', error);
      addMessage({
        role: 'assistant',
        content: 'エラーが発生しました。もう一度お試しください。',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* メッセージエリア */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {messages.length === 0 ? (
          <div className="text-center text-gray-500 mt-20">
            <p className="text-xl mb-2">👋 こんにちは！</p>
            <p>何かお手伝いできることはありますか？</p>
            <p className="text-sm mt-4 text-gray-400">
              個人情報は自動的にマスキングされます
            </p>
          </div>
        ) : (
          messages.map((message, index) => (
            <MessageBubble key={index} message={message} />
          ))
        )}
        {isLoading && (
          <div className="flex items-center justify-center p-4">
            <Loader2 className="w-6 h-6 animate-spin text-blue-500" />
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* 入力エリア */}
      <div className="border-t p-4 bg-white">
        <div className="flex gap-2 max-w-4xl mx-auto">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="メッセージを入力..."
            className="flex-1 p-3 border rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
            rows={3}
            disabled={isLoading}
          />
          <button
            onClick={handleSend}
            disabled={!input.trim() || isLoading}
            className="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
          >
            <Send className="w-5 h-5" />
          </button>
        </div>
      </div>
    </div>
  );
}
