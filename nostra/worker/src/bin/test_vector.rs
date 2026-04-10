use anyhow::Result;
use ic_agent::Agent;
use std::sync::Arc;

#[path = "../vector_client.rs"]
mod vector_client;
use vector_client::VectorClient;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Setup Agent
    let url = "http://127.0.0.1:4943";
    let agent = Agent::builder()
        .with_url(url)
        .with_identity(ic_agent::identity::AnonymousIdentity)
        .build()?;
    agent.fetch_root_key().await?;
    let agent = Arc::new(agent);

    // 2. Setup Client
    let config = cortex_worker::config_service::ConfigService::get();
    let canister_id = config
        .get_canister_id("primary")
        .expect("CANISTER_ID (primary) is required for vector testing");
    let client = VectorClient::new(agent, canister_id);

    println!("Creating collection...");
    match client.create_collection("test_col", 4).await {
        Ok(_) => println!("Created collection."),
        Err(e) => println!("Collection creation status: {:?}", e), // Likely UniqueViolation
    }

    println!("Inserting vector...");
    let vec1 = vec![0.1, 0.2, 0.3, 0.4];
    match client
        .insert(
            "test_col",
            vec![vec1.clone()],
            vec!["vec1".to_string()],
            "test_label",
        )
        .await
    {
        Ok(_) => println!("Inserted vector."),
        Err(e) => println!("Insert status: {:?}", e), // Likely UniqueViolation
    }

    println!("Building index...");
    client.build_index("test_col").await?;

    println!("Searching...");
    let results = client.search("test_col", vec1, 5).await?;

    println!("Results: {:?}", results);

    if results.contains(&"vec1".to_string()) {
        println!("SUCCESS: Found vec1");
    } else {
        println!("FAILURE: Did not find vec1");
    }

    Ok(())
}
