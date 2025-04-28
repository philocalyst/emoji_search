// src/main.rs
use emoji_search::{
    constants::{EmojiData, Options},
    search_best_matching_emojis, search_emojis,
};
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the tracing subscriber
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting emoji search example");

    // Use sample data since we don't have the actual files
    let emoji_data = EmojiData::sample_data();

    // Basic usage
    let results = search_emojis("amazing", None, None, &emoji_data).await?;
    info!("Basic search for 'amazing': {:?}", results);

    // With max limit
    let max_limit = Some(2);
    let results = search_emojis("amazing", max_limit, None, &emoji_data).await?;
    info!("Search for 'amazing' with max limit 2: {:?}", results);

    // Personalize with custom emoji keywords
    let mut custom_emoji_keywords = HashMap::new();
    custom_emoji_keywords.insert("🏆".to_string(), vec!["amazing".to_string()]);

    let options = Options {
        custom_emoji_keywords: Some(custom_emoji_keywords),
        ..Default::default()
    };

    let results = search_emojis("amazing", None, Some(options), &emoji_data).await?;
    info!(
        "Search for 'amazing' with custom emoji keywords: {:?}",
        results
    );

    // Personalize with user preferred keyword to emoji
    let mut custom_keyword_most_relevant_emoji = HashMap::new();
    custom_keyword_most_relevant_emoji.insert("amazing".to_string(), "💯".to_string());

    let options = Options {
        custom_keyword_most_relevant_emoji: Some(custom_keyword_most_relevant_emoji),
        ..Default::default()
    };

    let results = search_emojis("amazing", None, Some(options), &emoji_data).await?;
    info!(
        "Search for 'amazing' with custom most relevant emoji: {:?}",
        results
    );

    // Personalize with user recently searched inputs
    let recently_searched_inputs = vec!["hello".to_string()];

    let options = Options {
        recently_searched_inputs: Some(recently_searched_inputs),
        ..Default::default()
    };

    let results = search_emojis("h", Some(4), Some(options), &emoji_data).await?;
    info!(
        "Search for 'h' with recently searched inputs: {:?}",
        results
    );

    // Search for best match
    let results = search_best_matching_emojis("hello world", Some(4), None, &emoji_data).await?;
    info!("Best matching search for 'hello world': {:?}", results);

    info!("Example completed successfully");
    Ok(())
}
