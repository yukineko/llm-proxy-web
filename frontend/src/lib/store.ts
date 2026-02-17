import { create } from 'zustand';
import type { Message, ModelInfo } from '@/types';

interface ChatState {
  messages: Message[];
  selectedModel: string;
  availableModels: ModelInfo[];
  isLoading: boolean;
  
  addMessage: (message: Message) => void;
  setMessages: (messages: Message[]) => void;
  setSelectedModel: (model: string) => void;
  setAvailableModels: (models: ModelInfo[]) => void;
  setIsLoading: (loading: boolean) => void;
  clearMessages: () => void;
}

export const useChatStore = create<ChatState>((set) => ({
  messages: [],
  selectedModel: 'claude-sonnet-4-5',
  availableModels: [],
  isLoading: false,

  addMessage: (message) => 
    set((state) => ({ 
      messages: [...state.messages, { ...message, timestamp: new Date() }] 
    })),
  
  setMessages: (messages) => set({ messages }),
  setSelectedModel: (model) => set({ selectedModel: model }),
  setAvailableModels: (models) => set({ availableModels: models }),
  setIsLoading: (loading) => set({ isLoading: loading }),
  clearMessages: () => set({ messages: [] }),
}));
