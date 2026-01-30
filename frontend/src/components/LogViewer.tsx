'use client';

import { useState, useEffect } from 'react';
import { Search } from 'lucide-react';
import { format } from 'date-fns';
import { api } from '@/lib/api';
import type { LogEntry } from '@/types';

export default function LogViewer() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [total, setTotal] = useState(0);
  const [searchTerm, setSearchTerm] = useState('');
  const [page, setPage] = useState(0);
  const [isLoading, setIsLoading] = useState(false);

  const pageSize = 20;

  const fetchLogs = async () => {
    setIsLoading(true);
    try {
      const response = await api.queryLogs({
        search_term: searchTerm || undefined,
        limit: pageSize,
        offset: page * pageSize,
      });
      setLogs(response.logs);
      setTotal(response.total);
    } catch (error) {
      console.error('Failed to fetch logs:', error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchLogs();
  }, [page, searchTerm]);

  return (
    <div className="max-w-7xl mx-auto p-6">
      <div className="bg-white rounded-lg shadow">
        <div className="p-6 border-b">
          <h2 className="text-2xl font-bold mb-4">プロンプトログ</h2>
          
          <div className="flex gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => {
                  setSearchTerm(e.target.value);
                  setPage(0);
                }}
                placeholder="ログを検索..."
                className="w-full pl-10 pr-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">時刻</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">入力</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">出力</th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">PII</th>
              </tr>
            </thead>
            <tbody className="divide-y">
              {logs.map((log) => (
                <tr key={log.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap text-sm">
                    {format(new Date(log.timestamp), 'yyyy-MM-dd HH:mm:ss')}
                  </td>
                  <td className="px-6 py-4 text-sm max-w-md truncate">
                    {log.original_input}
                  </td>
                  <td className="px-6 py-4 text-sm max-w-md truncate">
                    {log.final_output}
                  </td>
                  <td className="px-6 py-4 text-sm">
                    <span className="px-2 py-1 bg-yellow-100 text-yellow-800 rounded text-xs">
                      {Object.keys(log.pii_mappings).length}件
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="px-6 py-4 border-t flex items-center justify-between">
          <div className="text-sm text-gray-500">
            全{total}件中 {page * pageSize + 1}-{Math.min((page + 1) * pageSize, total)}件を表示
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => setPage(Math.max(0, page - 1))}
              disabled={page === 0}
              className="px-4 py-2 border rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
            >
              前へ
            </button>
            <button
              onClick={() => setPage(page + 1)}
              disabled={(page + 1) * pageSize >= total}
              className="px-4 py-2 border rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
            >
              次へ
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
