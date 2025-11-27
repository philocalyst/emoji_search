//! Emoji Search Engine - A library for searching emojis based on keywords
//!
//! This library provides functionality to search for emojis based on text
//! input, with support for single word searches, multiple word searches, and
//! best matching searches.

use emoji::{EmojiEntry, lookup_by_glyph::lookup};
use tracing::{debug, error, trace};

pub mod error;
pub mod search;
pub mod types;
pub mod utils;

use emoji::Emoji;
use types::{EmojiData, Options};
use utils::{nlp::stemmer::stem_word, preprocess::pre_process_string};

use crate::{error::EmojiSearchError, search::{best_matching::search_for_words, single_word::search_for_word}};

#[derive(Clone)]
pub struct EmojiSearcher {
	sourced_emojis: EmojiData,
	options:        Options,
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
	pub fn new(data: &EmojiData, options: Option<Options>) -> Self {
		EmojiSearcher { sourced_emojis: data.to_owned(), options: options.unwrap_or_default() }
	}

	pub fn search_emojis(
		&self,
		input: &str,
		max_limit: Option<u32>,
	) -> Result<Vec<EmojiEntry>, EmojiSearchError> {
		let max_limit = max_limit.unwrap_or(24);
		let options = &self.options;
		let emoji_data = &self.sourced_emojis;

		debug!("Searching emojis with input: '{}', max_limit: {}", input, max_limit);

		if input.is_empty() {
			debug!("Empty input, returning empty results");
			return Ok(vec![]);
		}

		// Return the input itself if it is an emoji
		// We ignore toned variants because the glossary doesn't care for them
		if let Some(em) = lookup(input) {
			if emoji_data.emoji_set.contains(&em) {
				debug!("Input is a known emoji, returning it directly");
				return Ok(vec![em]);
			}
		} else {
			error!("{} is not a recongized emoji", input);
		}

		let processed_input = pre_process_string(input);

		// TODO: Determine if double whitespace is two words or reduced
		let input: Vec<&str> = processed_input.split_whitespace().collect();

		// Determine whether it's a single word or multiple words input
		let is_single_word_input = input.len() > 1;

		let results = if is_single_word_input {
			let input = input[0];
			trace!("Processing as single word input");
			search_for_word(&input, emoji_data, &options)
		} else {
			trace!("Processing as multiple words input");
			search_for_words(&input, emoji_data, &options)
		};

		// Truncate results to the specified limit
		let limited_results = results.into_iter().take(max_limit as usize).cloned().collect();

		Ok(limited_results)
	}

	/// Search for best matching emojis
	///
	/// This is a more forgiving search that would also match the stemmed input
	/// words by stripping off suffixes, and handles parts of speech filtering.
	///
	/// # Arguments
	/// * `input` - The search query string
	/// * `max_limit` - Maximum number of results to return (default: 24)
	/// * `options` - Custom options for the search algorithm
	///
	/// # Returns
	/// A vector of best matching emoji strings
	pub fn search_best_matching_emojis(
		&self,
		input: &str,
		max_limit: Option<u32>,
	) -> Result<Vec<&EmojiEntry>, EmojiSearchError> {
		let max_limit = max_limit.unwrap_or(24);
		let options = &self.options;
		let emoji_data = &self.sourced_emojis;

		debug!("Searching best matching emojis with input: '{}', max_limit: {}", input, max_limit);

		let input = pre_process_string(input).trim().to_string();
		if input.is_empty() {
			debug!("Empty input, returning empty results");
			return Ok([].into());
		}

		// Determine whether it's a single word or multiple words input
		let is_single_word_input = !input.contains(' ');

		let results = if is_single_word_input {
			trace!("Processing best matching for single word input");
			let mut emojis = search_for_word(&input, emoji_data, &options);

			// If no results, try with stemmed input
			if emojis.is_empty() {
				let stemmed_input = stem_word(&input);
				if stemmed_input != input {
					emojis = search_for_word(&stemmed_input, emoji_data, &options);
				}
			}

			emojis
		} else {
			trace!("Processing best matching for multiple words input");
			// First try regular multiple words search
			let emojis = search_for_word(&input, emoji_data, &options);

			// If no results, fall back to best matching search
			if emojis.is_empty() { search_for_word(&input, emoji_data, &options) } else { emojis }
		};

		// Truncate results to the specified limit
		let limited_results: Vec<&EmojiEntry> = results.into_iter().take(max_limit as usize).collect();

		Ok(limited_results)
	}
}
