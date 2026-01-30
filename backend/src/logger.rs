use sqlx::{PgPool, postgres::PgPoolOptions};
use anyhow::Result;
use crate::models::{LogEntry, LogQuery, LogResponse};

pub struct Logger {
    pool: PgPool,
}

impl Logger {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn log_request(&self, entry: LogEntry) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO prompt_logs 
            (id, timestamp, original_input, masked_input, rag_context, llm_output, final_output, pii_mappings)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            entry.id,
            entry.timestamp,
            entry.original_input,
            entry.masked_input,
            entry.rag_context,
            entry.llm_output,
            entry.final_output,
            entry.pii_mappings,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn query_logs(&self, query: LogQuery) -> Result<LogResponse> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);
        
        let mut where_clauses = vec!["1=1".to_string()];
        
        if let Some(start) = &query.start_date {
            where_clauses.push(format!("timestamp >= '{}'", start));
        }
        
        if let Some(end) = &query.end_date {
            where_clauses.push(format!("timestamp <= '{}'", end));
        }
        
        if let Some(search) = &query.search_term {
            where_clauses.push(format!(
                "(original_input ILIKE '%{}%' OR final_output ILIKE '%{}%')",
                search.replace('\'', "''"),
                search.replace('\'', "''")
            ));
        }
        
        let where_clause = where_clauses.join(" AND ");
        
        let sql = format!(
            "SELECT * FROM prompt_logs WHERE {} ORDER BY timestamp DESC LIMIT {} OFFSET {}",
            where_clause, limit, offset
        );
        
        let count_sql = format!(
            "SELECT COUNT(*) as count FROM prompt_logs WHERE {}",
            where_clause
        );
        
        let logs = sqlx::query_as::<_, LogEntry>(&sql)
            .fetch_all(&self.pool)
            .await?;
        
        let total: (i64,) = sqlx::query_as(&count_sql)
            .fetch_one(&self.pool)
            .await?;
        
        Ok(LogResponse {
            logs,
            total: total.0,
        })
    }

    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS prompt_logs (
                id UUID PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL,
                original_input TEXT NOT NULL,
                masked_input TEXT NOT NULL,
                rag_context TEXT,
                llm_output TEXT NOT NULL,
                final_output TEXT NOT NULL,
                pii_mappings JSONB NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_timestamp ON prompt_logs(timestamp DESC)
            "#
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_pii_mappings ON prompt_logs USING GIN(pii_mappings)
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
