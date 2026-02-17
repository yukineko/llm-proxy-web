'use client';

import { useEffect } from 'react';
import { ChevronDown } from 'lucide-react';
import { useChatStore } from '@/lib/store';
import { api } from '@/lib/api';

export default function ModelSelector() {
  const { selectedModel, availableModels, setSelectedModel, setAvailableModels } = useChatStore();

  useEffect(() => {
    api.listModels().then(setAvailableModels);
  }, [setAvailableModels]);

  return (
    <div className="relative">
      <select
        value={selectedModel}
        onChange={(e) => setSelectedModel(e.target.value)}
        className="appearance-none px-4 py-2 pr-8 border rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-blue-500 cursor-pointer"
      >
        {availableModels.map((model) => (
          <option key={model.id} value={model.id}>
            {model.name} ({model.provider})
          </option>
        ))}
      </select>
      <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 pointer-events-none text-gray-500" />
    </div>
  );
}
