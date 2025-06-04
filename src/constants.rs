// src/constants.rs
use crate::error::{EmojiSearchError, Result};
use emojis::common::EMOJIS;
use emojis::emoji::Emoji;
use emojis::get;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Map from emoji to its keywords
pub type EmojiKeywords = HashMap<Emoji, Vec<String>>;

/// Map from keyword to most relevant emoji
pub type KeywordMostRelevantEmoji = HashMap<String, Emoji>;

/// Map from keyword to emojis that match the keyword
pub type EmojiGlossary = HashMap<String, Vec<Emoji>>;

/// Map of words to their index in top 1000 words
pub type WordToTop1000WordsIdx = HashMap<String, usize>;

/// Options for customizing emoji search
#[derive(Clone, Default)]
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
    pub emoji_set: Arc<HashSet<Emoji>>,

    /// Map of words to their frequency rank in top 1000 words
    pub word_to_top_1000_words_idx: Arc<WordToTop1000WordsIdx>,
}

impl EmojiData {
    /// Create a new empty EmojiData structure
    pub fn new() -> Self {
        let emoji_keywords = Arc::new(HashMap::new());
        let keyword_most_relevant_emoji = Arc::new(HashMap::new());
        let emoji_glossary = Arc::new(HashMap::new());
        let emoji_set = Arc::new(HashSet::new());
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
        match serde_json::from_str::<HashMap<String, Vec<String>>>(include_str!(
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
        // Assuming the keys in your JSON are emoji characters
        if let Some(emoji) = emojis::get(&emoji_str) {
            emoji_keywords.insert(emoji, keywords);
        } else {
            // If the keys are shortcodes instead, try this
            if let Some(emoji) = emojis::get_by_shortcode(&emoji_str) {
                emoji_keywords.insert(emoji.to_owned(), keywords);
            } else {
                warn!("Could not find emoji for key: {}", emoji_str);
            }
        }
    }

    let keyword_most_relevant_emoji: KeywordMostRelevantEmoji = serde_json::from_str(
        include_str!("../data/emoogle-keyword-most-relevant-emoji.json"),
    )?;

    let emoji_glossary: EmojiGlossary =
        serde_json::from_str(include_str!("../data/emoogle-emoji-glossary.json"))?;

    let top_1000_words: Vec<String> =
        serde_json::from_str(include_str!("../data/top-1000-words-by-frequency.json"))?;

    // Create emoji set from keys of emoji_keywords
    let emoji_set: HashSet<Emoji> = EMOJIS.iter().cloned().collect();

    // Create map from words to their index in top 1000 words
    let word_to_top_1000_words_idx: WordToTop1000WordsIdx = top_1000_words
        .iter() // Gives &String
        .enumerate() // Gives (usize, &String)
        .filter_map(|(idx, word)| {
            get(word)
                // If get returns Some(emoji), map it to Some((emoji_str, idx))
                .map(|emoji| (emoji.to_string(), idx))
        })
        .collect();

    info!("Emoji data loaded successfully");

    Ok(EmojiData {
        emoji_keywords: Arc::new(emoji_keywords),
        keyword_most_relevant_emoji: Arc::new(keyword_most_relevant_emoji),
        emoji_glossary: Arc::new(emoji_glossary),
        emoji_set: Arc::new(emoji_set),
        word_to_top_1000_words_idx: Arc::new(word_to_top_1000_words_idx),
    })
}
