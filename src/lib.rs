//! Emoji Search Engine - A library for searching emojis based on keywords
//!
//! This library provides functionality to search for emojis based on text input,
//! with support for single word searches, multiple word searches, and best matching searches.

use emoji::lookup_by_glyph::lookup;
use tracing::{debug, error, trace};

pub mod error;
pub mod search;
pub mod types;
pub mod utils;

use emoji::{lookup_by_glyph, Emoji};
use search::{match_emoji_to_words, match_emojis_to_word};
use types::{EmojiData, Options};
use utils::nlp::stemmer::stem_word;
use utils::preprocess::pre_process_string;

use crate::error::EmojiSearchError;

#[derive(Clone)]
pub struct EmojiSearcher {
    sourced_emojis: EmojiData,
    options: Options,
}

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
impl EmojiSearcher {
    pub fn new(data: EmojiData, options: Option<Options>) -> Self {
        EmojiSearcher {
            sourced_emojis: data,
            options: options.unwrap_or_default(),
        }
    }

    pub async fn search_emojis(
        &self,
        input: &str,
        max_limit: Option<u32>,
    ) -> Result<&[&Emoji], EmojiSearchError> {
        let max_limit = max_limit.unwrap_or(24);
        let options = &self.options;
        let emoji_data = &self.sourced_emojis;

        debug!(
            "Searching emojis with input: '{}', max_limit: {}",
            input, max_limit
        );

        let input = pre_process_string(input).trim().to_string();
        if input.is_empty() {
            debug!("Empty input, returning empty results");
            return Ok(&[]);
        }

        // Return the input itself if it is an emoji
        if let Some(em) = lookup(input.as_str()) {
            if emoji_data.emoji_set.contains(&em) {
                debug!("Input is a known emoji, returning it directly");
                return Ok(&[em]);
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
        let limited_results = results.into_iter().take(max_limit as usize).collect();

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
        &self,
        input: &str,
        max_limit: Option<u32>,
    ) -> Result<&[&Emoji], EmojiSearchError> {
        let max_limit = max_limit.unwrap_or(24);
        let options = &self.options;
        let emoji_data = &self.sourced_emojis;

        debug!(
            "Searching best matching emojis with input: '{}', max_limit: {}",
            input, max_limit
        );

        let input = pre_process_string(input).trim().to_string();
        if input.is_empty() {
            debug!("Empty input, returning empty results");
            return Ok(&[]);
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
        let limited_results: Vec<Emoji> = results.into_iter().take(max_limit as usize).collect();

        Ok(limited_results)
    }
}
