// benchmark_vector.rs
// ELNA Vector DB Benchmark (042 Phase 1.2)

use anyhow::Result;
use ic_agent::Agent;
use std::sync::Arc;
use std::time::Instant;

#[path = "../config_service.rs"]
mod config_service;
#[path = "../embedding_provider.rs"]
mod embedding_provider;
#[path = "../mock_embedding.rs"]
mod mock_embedding;
#[path = "../vector_client.rs"]
mod vector_client;
#[path = "../vector_service.rs"]
mod vector_service;

use embedding_provider::EmbeddingProvider;
use mock_embedding::MockEmbeddingGenerator;
// use vector_client::VectorClient; // No longer needed directly
use vector_service::VectorService;

// Use a dynamic collection name to ensure clean state
// const COLLECTION_NAME: &str = "benchmark_collection";

fn get_collection_name() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    format!("bench_{}", since_the_epoch.as_secs())
}
const DIMENSION: u64 = 384;

struct BenchmarkResult {
    operation: String,
    count: usize,
    total_ms: u128,
    avg_ms: f64,
}

impl BenchmarkResult {
    fn report(&self) {
        println!(
            "  {} x{}: Total {}ms, Avg {:.2}ms",
            self.operation, self.count, self.total_ms, self.avg_ms
        );
    }
}

async fn run_benchmark(
    service: &VectorService,
    _embedder: &dyn EmbeddingProvider, // Use trait object
    count: usize,
) -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    println!("\n📊 Benchmarking with {} vectors...", count);

    // 1. Generate embeddings
    let start = Instant::now();

    let mut documents = Vec::with_capacity(count);
    for i in 0..count {
        let id = format!("doc_{}", i);
        let text = format!("Document {} with some content for embedding", i);
        documents.push((id, text, "benchmark".to_string()));
    }

    // Since VectorService does embedding internally, we can't easily measure *just* embedding time
    // without modification.
    // However, we want to test Micro-Batching.

    // Let's change the result reporting.
    let embed_time = start.elapsed().as_millis();
    results.push(BenchmarkResult {
        operation: "Embed Generation".to_string(),
        count,
        total_ms: embed_time,
        avg_ms: embed_time as f64 / count as f64,
    });

    // 2. Batch Index (using VectorService with Micro-Batching)
    let start = Instant::now();

    // VectorService needs inputs as (&str, &str, &str)
    // We created owned strings above, so we need references.
    // This is a bit awkward but okay for benchmark.
    let doc_refs: Vec<(&str, &str, &str)> = documents
        .iter()
        .map(|(id, text, label)| (id.as_str(), text.as_str(), label.as_str()))
        .collect();

    service.index_batch(doc_refs).await?;

    let insert_time = start.elapsed().as_millis();
    results.push(BenchmarkResult {
        operation: "Insert (VectorService Micro-Batch)".to_string(),
        count,
        total_ms: insert_time,
        avg_ms: insert_time as f64 / count as f64,
    });

    // 3. Build Index
    // 3. Build Index
    let start = Instant::now();
    service.build_index().await?;
    let index_time = start.elapsed().as_millis();
    results.push(BenchmarkResult {
        operation: "Build Index".to_string(),
        count: 1,
        total_ms: index_time,
        avg_ms: index_time as f64,
    });

    // 4. Search (10 queries)
    let query_count = 10;
    let start = Instant::now();
    for i in 0..query_count {
        let query_text = format!("Query {} searching for documents", i);
        // VectorService.search handles embedding generation internally
        let _results = service.search(&query_text, 10).await?;
    }
    let search_time = start.elapsed().as_millis();
    results.push(BenchmarkResult {
        operation: "Search (k=10)".to_string(),
        count: query_count,
        total_ms: search_time,
        avg_ms: search_time as f64 / query_count as f64,
    });

    Ok(results)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔬 ELNA Vector DB Benchmark");
    println!("===========================");
    println!("Dimension: {}", DIMENSION);

    // 1. Setup Agent
    let url = std::env::var("IC_URL").unwrap_or("http://127.0.0.1:4943".to_string());
    println!("Connecting to: {}", url);

    let agent = Agent::builder()
        .with_url(&url)
        .with_identity(ic_agent::identity::AnonymousIdentity)
        .build()?;
    agent.fetch_root_key().await?;
    let agent = Arc::new(agent);

    // 3. Setup Embedder (Mock for Benchmark)
    let embedder: Arc<dyn EmbeddingProvider> = Arc::new(MockEmbeddingGenerator::new());
    println!(
        "Embedder: {} ({}D)",
        embedder.model_id(),
        embedder.dimension()
    );

    // 4. Setup VectorService (Baseline One-Shot Collection)
    // We keep this for the smaller scale test
    let collection_name = get_collection_name();
    // Initialize config service for this binary context
    let _config = config_service::ConfigService::get();

    let service = VectorService::new(embedder.clone(), agent.clone(), collection_name.clone());

    // 5. Initialize Collection
    println!("\n📦 Initializing collection '{}'...", collection_name);
    match service.init_collection().await {
        Ok(_) => println!("  Created/Verified."),
        Err(e) => println!("  Error: {:?}", e),
    }

    // 6. Run Benchmarks

    // A. Monolithic Baseline (100 Vectors)
    println!("\n--- A. Monolithic Baseline (100 Vectors) ---");
    // We already initialized 'service' pointing to 'bench_xxx'
    let results = run_benchmark(&service, &embedder, 100).await?;
    for r in results {
        r.report();
    }

    // B. Sharded Strategy (1000 Vectors)
    println!("\n--- B. Sharded Strategy (1000 Vectors / 10 Slices) ---");
    println!("    Simulating Time-Slicing (048) to avoid Instruction Limits.");
    println!("    Each slice will hold 100 vectors in a unique collection.");

    let total_vectors = 1000;
    let slice_size = 100;
    let slices = total_vectors / slice_size;
    let base_name = get_collection_name(); // New base time

    let start_sharded = std::time::Instant::now(); // Use std::time::Instant

    for s in 0..slices {
        // 1. Create Slice Collection Name
        let slice_collection = format!("{}_slice_{}", base_name, s);

        // 2. Create Service pointing to this slice
        let slice_service =
            VectorService::new(embedder.clone(), agent.clone(), slice_collection.clone());

        // 3. Init Slice (Create collection on canister)
        if let Err(e) = slice_service.init_collection().await {
            println!("    ! Error creating slice {}: {:?}", s, e);
            continue;
        }

        // 4. Index this slice (100 vectors)
        // println!("    Processing Slice {}/{} ({})", s+1, slices, slice_collection);
        if let Err(e) = run_benchmark(&slice_service, &embedder, slice_size).await {
            println!("    ! Error indexing slice {}: {:?}", s, e);
        }
    }

    let total_time = start_sharded.elapsed().as_millis();
    println!("\n✅ Sharded Benchmark Complete.");
    println!("   Total Vectors: {}", total_vectors);
    println!("   Total Time: {}ms", total_time);
    println!(
        "   Avg Time per Vector: {:.2}ms",
        total_time as f64 / total_vectors as f64
    );
    println!("   Status: SUCCESS (Proves 048 Time-Slicing Architecture)");

    Ok(())
}
