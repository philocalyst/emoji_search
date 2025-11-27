use std::env::args;

use emoji_search::{EmojiSearcher, types::{self}};
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	info!("Starting emoji search example");

	// Load the emoji data (using sample data here)
	let emoji_data = types::load_emoji_data().unwrap();

	// Collect CLI args
	let arguments: Vec<String> = args().collect();

	// Build matcher
	let matcher = EmojiSearcher::new(&emoji_data, None);

	// Perform the search
	let results = matcher.search_best_matching_emojis(arguments[1].as_str(), Some(10))?;

	for result in results {
		println!("{result:?}");
	}

	info!("Example completed successfully");
	Ok(())
}
