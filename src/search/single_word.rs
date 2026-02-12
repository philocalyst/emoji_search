use std::{cmp::Ordering, collections::HashMap};

use emoji::Emoji;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::{debug, trace};

use crate::{types::{EmojiData, Options}, utils::preprocess::pre_process_string};

/// Attributes for ranking emojis in single word search
#[derive(Debug, Clone, PartialEq, Eq)]
struct Attributes {
	is_exact_match:                            bool,
	is_custom_most_relevant_emoji:             bool,
	is_most_relevant_emoji:                    bool,
	is_emoji_name:                             bool,
	is_single_word:                            bool,
	match_word:                                String,
	prefix_match_recently_searched_inputs_idx: Option<u32>,
	prefix_match_top_1000_words_idx:           Option<u32>,
}

/// Search emojis for a single word input, e.g. "dog"
pub fn search_for_word<'a>(
	input_word: &str,
	emoji_data: &'a EmojiData,
	options: &Options,
) -> Vec<&'a Emoji> {
	debug!("Searching emojis for single word input: {}", input_word);

	// Empty collections for fallback
	let empty_keywords_map: HashMap<Emoji, Vec<String>> = HashMap::new(); // Adjust types to match your EmojiKeywords
	let empty_relevant_map: HashMap<String, Emoji> = HashMap::new(); // Adjust to match KeywordMostRelevantEmoji
	let empty_vec: Vec<String> = Vec::new();

	// Borrow against the fallbacks
	let custom_emoji_keywords = options.custom_emoji_keywords.as_ref().unwrap_or(&empty_keywords_map);
	let custom_keyword_most_relevant_emoji =
		options.custom_keyword_most_relevant_emoji.as_ref().unwrap_or(&empty_relevant_map);
	let recently_searched_inputs = options.recently_searched_inputs.as_ref().unwrap_or(&empty_vec);

	// Create map from words to recently searched indices
	let word_to_recently_searched_inputs_idx: Option<HashMap<String, usize>> =
		if !recently_searched_inputs.is_empty() {
			Some(
				recently_searched_inputs
                    .iter()
                    .enumerate()
                    .map(|(idx, input)| (input.clone(), idx)) // Only clone strings here, unavoidable
                    .collect(),
			)
		} else {
			None
		};

	// Parallel iteration with Rayon
	let emojis_attributes: Vec<(&'a Emoji, Attributes)> = emoji_data
		.emoji_keywords
		.par_iter()
		.filter_map(|(emoji, keywords)| {
			// Combine keywords (clone only if custom exists—minimal)
			let all_keywords = if let Some(custom_kw) = custom_emoji_keywords.get(emoji) {
				let mut combined = keywords.clone(); // Clone vec only when needed
				combined.extend_from_slice(custom_kw);
				combined
			} else {
				keywords.clone()
			};

			// Refs to shared data
			let keyword_most_relevant_emoji = &*emoji_data.keyword_most_relevant_emoji;
			let word_to_top_1000_words_idx = &*emoji_data.word_to_top_1000_words_idx;
			let recent_idx_ref = word_to_recently_searched_inputs_idx.as_ref();

			get_emoji_best_attributes(
				input_word,
				emoji,
				&all_keywords,
				custom_keyword_most_relevant_emoji,
				keyword_most_relevant_emoji,
				recent_idx_ref,
				word_to_top_1000_words_idx,
			)
			.map(|attrs| (emoji, attrs))
		})
		.collect();

	// Sort emojis by attributes
	let mut emojis_attributes = emojis_attributes; // Make mutable for sorting
	emojis_attributes.sort_by(|(_, a), (_, b)| compare_attributes(a, b));

	// Extract sorted emojis
	let results: Vec<&'a Emoji> =
		emojis_attributes.into_iter().map(|(emoji, _attributes)| emoji).collect();

	debug!("Found {} matching emojis for single word input", results.len());
	results
}

/// Get the best attributes for an emoji based on its keywords matching the
/// input word
fn get_emoji_best_attributes(
	input_word: &str,
	emoji: &Emoji,
	keywords: &[String],
	custom_keyword_most_relevant_emoji: &HashMap<String, Emoji>,
	keyword_most_relevant_emoji: &HashMap<String, Emoji>,
	word_to_recently_searched_inputs_idx: Option<&HashMap<String, usize>>,
	word_to_top_1000_words_idx: &HashMap<String, usize>,
) -> Option<Attributes> {
	trace!("Getting best attributes for emoji {:?} with input {}", emoji, input_word);

	let mut emoji_best_attributes: Option<Attributes> = None;

	// Process each keyword to find best match
	for (i, keyword) in keywords.iter().enumerate() {
		let keyword = pre_process_string(keyword);

		let is_emoji_name = i == 0; // First keyword is the emoji name
		let is_single_word = !keyword.contains(' ');

		if is_single_word {
			let is_exact_match = compute_is_exact_match(input_word, &keyword);

			// Skip if there is no keyword match
			if is_exact_match.is_none() {
				continue;
			}

			let is_exact_match = is_exact_match.unwrap();
			let is_most_relevant_emoji = keyword_most_relevant_emoji.get(&keyword) == Some(&emoji);
			let is_custom_most_relevant_emoji =
				custom_keyword_most_relevant_emoji.get(&keyword) == Some(&emoji);

			let prefix_match_recently_searched_inputs_idx = if !is_exact_match {
				word_to_recently_searched_inputs_idx
					.and_then(|map| map.get(&keyword).map(|&idx| idx as u32))
			} else {
				None
			};

			let prefix_match_top_1000_words_idx = if !is_exact_match {
				word_to_top_1000_words_idx.get(&keyword).map(|&idx| idx as u32)
			} else {
				None
			};

			let attributes = Attributes {
				is_exact_match,
				is_custom_most_relevant_emoji,
				is_most_relevant_emoji,
				is_emoji_name,
				is_single_word,
				match_word: keyword.clone(),
				prefix_match_recently_searched_inputs_idx,
				prefix_match_top_1000_words_idx,
			};

			// Update best attributes if current attributes is better
			if emoji_best_attributes.is_none()
				|| compare_attributes(&attributes, emoji_best_attributes.as_ref().unwrap())
					== Ordering::Less
			{
				emoji_best_attributes = Some(attributes);
			}
		} else {
			// For multi-word keywords, check each word
			let words: Vec<String> = keyword.split(' ').map(|w| w.to_string()).collect();

			for word in words {
				let is_exact_match = compute_is_exact_match(input_word, &word);

				// Skip if there is no keyword match
				if is_exact_match.is_none() {
					continue;
				}

				let is_exact_match = is_exact_match.unwrap();
				let is_most_relevant_emoji = keyword_most_relevant_emoji.get(&word) == Some(&emoji);
				let is_custom_most_relevant_emoji =
					custom_keyword_most_relevant_emoji.get(&word) == Some(&emoji);

				let prefix_match_recently_searched_inputs_idx = if !is_exact_match {
					word_to_recently_searched_inputs_idx.and_then(|map| map.get(&word).map(|&idx| idx as u32))
				} else {
					None
				};

				let prefix_match_top_1000_words_idx = if !is_exact_match {
					word_to_top_1000_words_idx.get(&word).map(|&idx| idx as u32)
				} else {
					None
				};

				let attributes = Attributes {
					is_exact_match,
					is_custom_most_relevant_emoji,
					is_most_relevant_emoji,
					is_emoji_name,
					is_single_word: false,
					match_word: word.clone(),
					prefix_match_recently_searched_inputs_idx,
					prefix_match_top_1000_words_idx,
				};

				// Update best attributes if current attributes is better
				if emoji_best_attributes.is_none()
					|| compare_attributes(&attributes, emoji_best_attributes.as_ref().unwrap())
						== Ordering::Less
				{
					emoji_best_attributes = Some(attributes);
				}
			}
		}
	}

	emoji_best_attributes
}

/// Check if input_word matches keyword exactly or as a prefix
fn compute_is_exact_match(input_word: &str, keyword: &str) -> Option<bool> {
	if input_word == keyword {
		Some(true)
	} else if keyword.starts_with(input_word) {
		Some(false)
	} else {
		None
	}
}

/// Compare attributes for ranking
///
/// This is the core ranking function for single word search results.
/// It uses a tie-breaking algorithm with multiple criteria in order of
/// importance.
fn compare_attributes(a: &Attributes, b: &Attributes) -> Ordering {
	// 1. Exact match ranks higher than prefix match
	match (a.is_exact_match, b.is_exact_match) {
		(true, false) => return Ordering::Less,
		(false, true) => return Ordering::Greater,
		_ => {}
	}

	if a.is_exact_match {
		// Exact match ranking criteria:

		// 2. Custom most relevant emoji ranks higher
		match (a.is_custom_most_relevant_emoji, b.is_custom_most_relevant_emoji) {
			(true, false) => return Ordering::Less,
			(false, true) => return Ordering::Greater,
			_ => {}
		}

		// 3. Most relevant emoji ranks higher
		match (a.is_most_relevant_emoji, b.is_most_relevant_emoji) {
			(true, false) => return Ordering::Less,
			(false, true) => return Ordering::Greater,
			_ => {}
		}

		// 4. Keyword in emoji name ranks higher
		match (a.is_emoji_name, b.is_emoji_name) {
			(true, false) => return Ordering::Less,
			(false, true) => return Ordering::Greater,
			_ => {}
		}

		// 5. Single word keyword ranks higher
		match (a.is_single_word, b.is_single_word) {
			(true, false) => return Ordering::Less,
			(false, true) => return Ordering::Greater,
			_ => {}
		}

		Ordering::Equal
	} else {
		// Prefix match ranking criteria:

		// 2. Recently searched input ranks higher
		match (
			&a.prefix_match_recently_searched_inputs_idx,
			&b.prefix_match_recently_searched_inputs_idx,
		) {
			(Some(a_idx), Some(b_idx)) => {
				if a_idx != b_idx {
					return a_idx.cmp(b_idx);
				}
			}
			(Some(_), None) => return Ordering::Less,
			(None, Some(_)) => return Ordering::Greater,
			_ => {}
		}

		// 3. Single word keyword ranks higher
		match (a.is_single_word, b.is_single_word) {
			(true, false) => return Ordering::Less,
			(false, true) => return Ordering::Greater,
			_ => {}
		}

		// 4. Top 1000 word ranks higher
		match (&a.prefix_match_top_1000_words_idx, &b.prefix_match_top_1000_words_idx) {
			(Some(a_idx), Some(b_idx)) => {
				if a_idx != b_idx {
					return a_idx.cmp(b_idx);
				}
			}
			(Some(_), None) => return Ordering::Less,
			(None, Some(_)) => return Ordering::Greater,
			_ => {}
		}

		// 5. Alphabetical order
		let cmp = a.match_word.cmp(&b.match_word);
		if cmp != Ordering::Equal {
			return cmp;
		}

		// 6. Custom most relevant emoji ranks higher
		match (a.is_custom_most_relevant_emoji, b.is_custom_most_relevant_emoji) {
			(true, false) => return Ordering::Less,
			(false, true) => return Ordering::Greater,
			_ => {}
		}

		// 7. Most relevant emoji ranks higher
		match (a.is_most_relevant_emoji, b.is_most_relevant_emoji) {
			(true, false) => return Ordering::Less,
			(false, true) => return Ordering::Greater,
			_ => {}
		}

		Ordering::Equal
	}
}
