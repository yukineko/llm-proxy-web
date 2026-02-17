-- init.sql - Database schema initialization

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Prompt logs table
CREATE TABLE IF NOT EXISTS prompt_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    original_input TEXT NOT NULL,
    masked_input TEXT NOT NULL,
    rag_context TEXT,
    llm_output TEXT NOT NULL,
    final_output TEXT NOT NULL,
    pii_mappings JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_prompt_logs_timestamp ON prompt_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_prompt_logs_pii_mappings ON prompt_logs USING GIN(pii_mappings);
CREATE INDEX IF NOT EXISTS idx_prompt_logs_search ON prompt_logs USING gin(to_tsvector('english', original_input || ' ' || final_output));

-- Documents table (for RAG metadata)
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    category TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_documents_category ON documents(category);
CREATE INDEX IF NOT EXISTS idx_documents_created_at ON documents(created_at DESC);

COMMENT ON TABLE prompt_logs IS 'Logs all LLM interactions with PII masking information';
COMMENT ON TABLE documents IS 'RAG document metadata storage';
