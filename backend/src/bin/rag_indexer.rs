use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Sha256, Digest};
use std::path::PathBuf;

use llm_proxy::rag::embeddings::EmbeddingGenerator;
use llm_proxy::rag::vector_store::VectorStore;
use llm_proxy::indexer::walker::{walk_directory, SupportedFormat};
use llm_proxy::indexer::extractor::extract_text;
use llm_proxy::indexer::chunker::chunk_text;

#[derive(Parser, Debug)]
#[command(name = "rag-indexer")]
#[command(about = "Index documents into the RAG vector store")]
struct Args {
    /// Directory to recursively index
    #[arg(short, long)]
    dir: PathBuf,

    /// Qdrant server URL
    #[arg(long, env = "QDRANT_URL", default_value = "http://localhost:6334")]
    qdrant_url: String,

    /// Qdrant collection name
    #[arg(long, default_value = "documents")]
    collection: String,

    /// Maximum chunk size in characters
    #[arg(long, default_value_t = 1000)]
    chunk_size: usize,

    /// Overlap between chunks in characters
    #[arg(long, default_value_t = 200)]
    chunk_overlap: usize,
}

fn file_id(path: &PathBuf) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.to_string_lossy().as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    if !args.dir.exists() {
        anyhow::bail!("Directory does not exist: {}", args.dir.display());
    }

    println!("Initializing embedding model...");
    let embeddings = EmbeddingGenerator::new().await?;

    println!("Connecting to Qdrant at {}...", args.qdrant_url);
    let vector_store = VectorStore::new(&args.qdrant_url, &args.collection).await?;

    println!("Scanning directory: {}", args.dir.display());
    let files = walk_directory(&args.dir);
    println!("Found {} supported files", files.len());

    if files.is_empty() {
        println!("No supported files found. Exiting.");
        return Ok(());
    }

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
            .progress_chars("#>-"),
    );

    let mut success_count = 0usize;
    let mut fail_count = 0usize;
    let mut total_chunks = 0usize;
    let mut failed_files: Vec<(PathBuf, String)> = Vec::new();

    for (path, format) in &files {
        pb.set_message(format!("{}", path.file_name().unwrap_or_default().to_string_lossy()));

        match process_file(path, *format, &embeddings, &vector_store, &args).await {
            Ok(chunk_count) => {
                success_count += 1;
                total_chunks += chunk_count;
            }
            Err(e) => {
                tracing::warn!("Failed to process {}: {}", path.display(), e);
                failed_files.push((path.clone(), format!("{}", e)));
                fail_count += 1;
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("done");

    println!("\nIndexing complete!");
    println!("  Files processed: {}/{}", success_count, files.len());
    println!("  Files failed:    {}", fail_count);
    println!("  Total chunks:    {}", total_chunks);
    println!("  Collection:      {}", args.collection);
    println!("  Qdrant URL:      {}", args.qdrant_url);

    if !failed_files.is_empty() {
        println!("\nFailed files:");
        for (path, err) in &failed_files {
            println!("  {}: {}", path.display(), err);
        }
    }

    Ok(())
}

async fn process_file(
    path: &PathBuf,
    format: SupportedFormat,
    embeddings: &EmbeddingGenerator,
    vector_store: &VectorStore,
    args: &Args,
) -> Result<usize> {
    let text = extract_text(path, format)?;

    if text.trim().is_empty() {
        return Ok(0);
    }

    let chunks = chunk_text(&text, args.chunk_size, args.chunk_overlap);
    let path_id = file_id(path);

    let batch_size = 32;
    for batch in chunks.chunks(batch_size) {
        let texts: Vec<String> = batch.iter().map(|c| c.text.clone()).collect();
        let embeddings_batch = embeddings.generate(texts)?;

        for (chunk, embedding) in batch.iter().zip(embeddings_batch.into_iter()) {
            let chunk_id = format!("{}_{}", path_id, chunk.chunk_index);
            let metadata = serde_json::json!({
                "file_path": path.to_string_lossy(),
                "chunk_index": chunk.chunk_index,
                "format": format!("{:?}", format),
            });

            vector_store.add_document(&chunk_id, &chunk.text, embedding, metadata).await?;
        }
    }

    Ok(chunks.len())
}
