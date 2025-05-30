use emoji_search::{
    constants::{self},
    search_best_matching_emojis, search_emojis,
};
use env_logger;
use log::info;
use serde_cbor;
use std::io::Write;
use std::{env::args, ops::Deref};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // (Optionally initialize tracing subscriber here)

    info!("Starting emoji search example");

    // Load the emoji data (using sample data here)
    let emoji_data = constants::load_emoji_data().unwrap();

    // Collect CLI args
    let arguments: Vec<String> = args().collect();

    // Perform the search
    let results =
        search_best_matching_emojis(arguments[1].as_str(), Some(400), None, &emoji_data).await?;
    info!("Best matching search for query: {:?}", results);

    // Serialize results to CBOR
    let cbor_bytes = serde_cbor::to_vec(&emoji_data.emoji_set.deref())?;
    // Emit raw CBOR to stdout
    std::io::stdout().write_all(&cbor_bytes)?;

    info!("Example completed successfully");
    Ok(())
}
