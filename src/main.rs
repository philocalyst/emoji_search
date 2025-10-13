use emoji_search::{
    constants::{self},
    EmojiSearcher,
};
use env_logger;
use log::info;
use std::env::args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    // (Optionally initialize tracing subscriber here)

    info!("Starting emoji search example");

    // Load the emoji data (using sample data here)
    let emoji_data = constants::load_emoji_data().unwrap();

    // Collect CLI args
    let arguments: Vec<String> = args().collect();

    // Build matcher
    let matcher = EmojiSearcher::new(emoji_data, None);

    // Perform the search
    let results = matcher
        .search_best_matching_emojis(arguments[1].as_str(), Some(10))
        .await?;

    for result in results {
        println!("{result}");
    }

    info!("Example completed successfully");
    Ok(())
}
