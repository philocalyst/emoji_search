use std::{collections::HashMap, sync::Arc};

use emoji::{Emoji, EmojiEntry, lookup_by_glyph::lookup};
use sonic_rs::{Value, ValueRef};
use tracing::{error, info};

use crate::error::{EmojiSearchError, Result};

/// Map from emoji to its keywords
pub type EmojiKeywords = HashMap<EmojiEntry, Vec<String>>;

/// Map from keyword to most relevant emoji
pub type KeywordMostRelevantEmoji = HashMap<String, EmojiEntry>;

/// Map from keyword to emojis that match the keyword
pub type EmojiGlossary = HashMap<String, Vec<EmojiEntry>>;

/// Map of words to their index in top 1000 words
pub type WordToTop1000WordsIdx = HashMap<String, usize>;

/// Options for customizing emoji search
#[derive(Default, Clone)]
pub struct Options {
	/// Custom emoji keywords to extend built-in keywords
	pub custom_emoji_keywords: Option<EmojiKeywords>,

	/// Custom mappings from keywords to preferred emojis
	pub custom_keyword_most_relevant_emoji: Option<KeywordMostRelevantEmoji>,

	/// Recently searched inputs for improved search suggestions
	pub recently_searched_inputs: Option<Vec<String>>,
}

/// Core data structure containing all emoji data
#[derive(Clone)]
pub struct EmojiData {
	/// Map from emoji to its keywords
	/// e.g. {"➕": ["plus", "add", "sum", "and", "increase", "positive", "math"]}
	pub emoji_keywords: Arc<EmojiKeywords>,

	/// Map from keyword to most relevant emoji
	/// e.g. {"a": "🅰️"}
	pub keyword_most_relevant_emoji: Arc<KeywordMostRelevantEmoji>,

	/// Map from keyword to emojis that match it
	/// e.g. {"0": ["0️⃣", "✊"]}
	pub emoji_glossary: Arc<EmojiGlossary>,

	/// Set of all available emojis
	pub emoji_set: Arc<Vec<EmojiEntry>>,

	/// Map of words to their frequency rank in top 1000 words
	pub word_to_top_1000_words_idx: Arc<WordToTop1000WordsIdx>,
}

impl EmojiData {
	/// Create a new empty EmojiData structure
	pub fn new() -> Self {
		let emoji_keywords = Arc::new(HashMap::new());
		let keyword_most_relevant_emoji = Arc::new(HashMap::new());
		let emoji_glossary = Arc::new(HashMap::new());
		let emoji_set = Arc::new(Vec::new());
		let word_to_top_1000_words_idx = Arc::new(HashMap::new());

		Self {
			emoji_keywords,
			keyword_most_relevant_emoji,
			emoji_glossary,
			emoji_set,
			word_to_top_1000_words_idx,
		}
	}
}

/// Load emoji data from embedded JSON files
pub fn load_emoji_data() -> Result<EmojiData> {
	info!("Loading emoji data from embedded resources");

	// First, parse the JSON into a temporary HashMap with String keys
	let emoji_json_data: HashMap<String, Vec<String>> =
		match sonic_rs::from_str::<HashMap<String, Vec<String>>>(include_str!(
			"../data/emoogle-emoji-keywords.json"
		)) {
			Ok(data) => {
				info!("Loaded emoji keywords JSON: {} entries", data.len());
				data
			}
			Err(e) => {
				error!("Failed to parse emoji keywords: {}", e);
				return Err(EmojiSearchError::Json(e));
			}
		};

	// Then convert the HashMap with String keys to one with &'static Emoji keys
	let mut emoji_keywords: EmojiKeywords = HashMap::new();
	for (emoji_str, keywords) in emoji_json_data {
		// The keys in the JSON are emoji characters
		if let Some(emoji) = lookup(&emoji_str) {
			emoji_keywords.insert(emoji.to_owned(), keywords);
		}
	}

	let keyword_most_relevant_emoji_str =
		include_str!("../data/emoogle-keyword-most-relevant-emoji.json");
	let emoji_glossary_str = include_str!("../data/emoogle-emoji-glossary.json");

	let keyword_most_relevant_emoji_value: Value =
		sonic_rs::from_str(keyword_most_relevant_emoji_str)?;
	let emoji_glossary_value: Value = sonic_rs::from_str(emoji_glossary_str)?;

	let mut keyword_most_relevant_emoji: KeywordMostRelevantEmoji = HashMap::new();
	if let ValueRef::Object(map) = keyword_most_relevant_emoji_value.as_ref() {
		for (key, val) in map {
			if let ValueRef::String(emoji_glyph) = val.as_ref() {
				if let Some(emoji) = lookup(&emoji_glyph) {
					keyword_most_relevant_emoji.insert(key.to_string(), emoji.to_owned());
				} else {
					eprintln!(
						"Warning: Emoji glyph '{}' not found in GLYPH_LOOKUP_MAP for keyword '{}'",
						emoji_glyph, key
					);
				}
			}
		}
	}

	let mut emoji_glossary: EmojiGlossary = HashMap::new();
	if let ValueRef::Object(map) = emoji_glossary_value.as_ref() {
		for (key, val) in map {
			if let ValueRef::Array(emoji_glyphs) = val.as_ref() {
				let mut emojis_for_keyword = Vec::new();
				for glyph_val in emoji_glyphs {
					if let ValueRef::String(emoji_glyph) = glyph_val.as_ref() {
						if let Some(emoji) = lookup(&emoji_glyph) {
							emojis_for_keyword.push(emoji.to_owned());
						} else {
							eprintln!(
								"Warning: Emoji glyph '{}' not found in GLYPH_LOOKUP_MAP for keyword '{}'",
								emoji_glyph, key
							);
						}
					}
				}
				emoji_glossary.insert(key.to_string(), emojis_for_keyword);
			}
		}
	}

	let top_1000_words: Vec<String> =
		sonic_rs::from_str(include_str!("../data/top-1000-words-by-frequency.json"))?;

	// Create emoji set from keys of emoji_keywords, skipping over variants
	let emoji_set: Vec<EmojiEntry> = emoji::lookup_by_glyph::iter_emoji().cloned().collect();

	// Create map from words to their index in top 1000 words
	let word_to_top_1000_words_idx: HashMap<String, usize> = top_1000_words
        .into_iter() // Consume the Vec for efficiency
        .enumerate()
        .map(|(idx, word)| (word, idx)) // Map word -> idx
        .collect();

	info!("Emoji data loaded successfully");

	Ok(EmojiData {
		emoji_keywords:              Arc::new(emoji_keywords),
		keyword_most_relevant_emoji: Arc::new(keyword_most_relevant_emoji),
		emoji_glossary:              Arc::new(emoji_glossary),
		emoji_set:                   Arc::new(emoji_set),
		word_to_top_1000_words_idx:  Arc::new(word_to_top_1000_words_idx),
	})
}
