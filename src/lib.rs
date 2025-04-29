// src/lib.rs
//! Emoji Search Engine - A library for searching emojis based on keywords
//!
//! This library provides functionality to search for emojis based on text input,
//! with support for single word searches, multiple word searches, and best matching searches.

use tracing::{debug, error, trace};

pub mod constants;
pub mod error;
pub mod search;
pub mod utils;

use constants::{EmojiData, Options};
use emojis::{get, Emoji};
use error::Result;
use search::{match_emoji_to_words, match_emojis_to_word};
use utils::nlp::stemmer::stem_word;
use utils::preprocess::pre_process_string;

/// Main entry point for searching emojis
///
/// Optimized for search-as-you-type experience. The more characters/words
/// a user types, the narrower the set of emojis returned.
///
/// # Arguments
/// * `input` - The search query string
/// * `max_limit` - Maximum number of results to return (default: 24)
/// * `options` - Custom options for the search algorithm
///
/// # Returns
/// A vector of matching emoji strings
pub async fn search_emojis(
    input: &str,
    max_limit: Option<usize>,
    options: Option<Options>,
    emoji_data: &EmojiData,
) -> Result<Vec<&'static Emoji>> {
    let max_limit = max_limit.unwrap_or(24);
    let options = options.unwrap_or_default();

    debug!(
        "Searching emojis with input: '{}', max_limit: {}",
        input, max_limit
    );

    let input = pre_process_string(input).trim().to_string();
    if input.is_empty() {
        debug!("Empty input, returning empty results");
        return Ok(Vec::new());
    }

    // Return the input itself if it is an emoji
    if let Some(em) = get(input.as_str()) {
        if emoji_data.emoji_set.contains(em) {
            debug!("Input is a known emoji, returning it directly");
            return Ok(vec![em]);
        }
    } else {
        error!("{} is not a recongized emoji", input);
    }

    // Determine whether it's a single word or multiple words input
    let is_single_word_input = !input.contains(' ');

    let results = if is_single_word_input {
        trace!("Processing as single word input");
        match_emojis_to_word(&input, emoji_data, &options).await
    } else {
        trace!("Processing as multiple words input");
        match_emoji_to_words(&input, emoji_data, &options).await
    };

    // Truncate results to the specified limit
    let limited_results = results.into_iter().take(max_limit).collect();

    Ok(limited_results)
}

/// Search for best matching emojis
///
/// This is a more forgiving search that would also match the stemmed input words
/// by stripping off suffixes, and handles parts of speech filtering.
///
/// # Arguments
/// * `input` - The search query string
/// * `max_limit` - Maximum number of results to return (default: 24)
/// * `options` - Custom options for the search algorithm
///
/// # Returns
/// A vector of best matching emoji strings
pub async fn search_best_matching_emojis(
    input: &str,
    max_limit: Option<usize>,
    options: Option<Options>,
    emoji_data: &EmojiData,
) -> Result<Vec<&'static Emoji>> {
    let max_limit = max_limit.unwrap_or(24);
    let options = options.unwrap_or_default();

    debug!(
        "Searching best matching emojis with input: '{}', max_limit: {}",
        input, max_limit
    );

    let input = pre_process_string(input).trim().to_string();
    if input.is_empty() {
        debug!("Empty input, returning empty results");
        return Ok(Vec::new());
    }

    // Determine whether it's a single word or multiple words input
    let is_single_word_input = !input.contains(' ');

    let results = if is_single_word_input {
        trace!("Processing best matching for single word input");
        let mut emojis = match_emojis_to_word(&input, emoji_data, &options).await;

        // If no results, try with stemmed input
        if emojis.is_empty() {
            let stemmed_input = stem_word(&input);
            if stemmed_input != input {
                emojis = match_emojis_to_word(&stemmed_input, emoji_data, &options).await;
            }
        }

        emojis
    } else {
        trace!("Processing best matching for multiple words input");
        // First try regular multiple words search
        let emojis = match_emoji_to_words(&input, emoji_data, &options).await;

        // If no results, fall back to best matching search
        if emojis.is_empty() {
            match_emoji_to_words(&input, emoji_data, &options).await
        } else {
            emojis
        }
    };

    // Truncate results to the specified limit
    let limited_results: Vec<&'static Emoji> = results.into_iter().take(max_limit).collect();

    Ok(limited_results)
}
