// src/constants.rs
use crate::error::{EmojiSearchError, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{error, info};

/// Map from emoji to its keywords
pub type EmojiKeywords = HashMap<String, Vec<String>>;

/// Map from keyword to most relevant emoji
pub type KeywordMostRelevantEmoji = HashMap<String, String>;

/// Map from keyword to emojis that match the keyword
pub type EmojiGlossary = HashMap<String, Vec<String>>;

/// Map of words to their index in top 1000 words
pub type WordToTop1000WordsIdx = HashMap<String, usize>;

/// Options for customizing emoji search
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Custom emoji keywords to extend built-in keywords
    pub custom_emoji_keywords: Option<EmojiKeywords>,

    /// Custom mappings from keywords to preferred emojis
    pub custom_keyword_most_relevant_emoji: Option<KeywordMostRelevantEmoji>,

    /// Recently searched inputs for improved search suggestions
    pub recently_searched_inputs: Option<Vec<String>>,
}

/// Core data structure containing all emoji data
#[derive(Clone, Debug)]
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
    pub emoji_set: Arc<HashSet<String>>,

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

    /// Create an EmojiData instance with some sample data for testing
    pub fn sample_data() -> Self {
        let mut emoji_keywords = HashMap::new();
        emoji_keywords.insert(
            "😀".to_string(),
            vec![
                "grinning face".to_string(),
                "happy".to_string(),
                "smile".to_string(),
                "joy".to_string(),
                "cheerful".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🤩".to_string(),
            vec![
                "star-struck".to_string(),
                "wow".to_string(),
                "amazing".to_string(),
                "excited".to_string(),
            ],
        );
        emoji_keywords.insert(
            "💯".to_string(),
            vec![
                "hundred points".to_string(),
                "perfect".to_string(),
                "amazing".to_string(),
                "score".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🙌".to_string(),
            vec![
                "raising hands".to_string(),
                "celebration".to_string(),
                "amazing".to_string(),
                "hooray".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🌈".to_string(),
            vec![
                "rainbow".to_string(),
                "colorful".to_string(),
                "amazing".to_string(),
                "pride".to_string(),
            ],
        );
        emoji_keywords.insert(
            "👋".to_string(),
            vec![
                "waving hand".to_string(),
                "hello".to_string(),
                "goodbye".to_string(),
                "greeting".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🫂".to_string(),
            vec![
                "people hugging".to_string(),
                "hug".to_string(),
                "hello".to_string(),
                "comfort".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🤝".to_string(),
            vec![
                "handshake".to_string(),
                "agreement".to_string(),
                "help".to_string(),
                "deal".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🙏".to_string(),
            vec![
                "folded hands".to_string(),
                "please".to_string(),
                "help".to_string(),
                "pray".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🆘".to_string(),
            vec![
                "SOS".to_string(),
                "help".to_string(),
                "emergency".to_string(),
            ],
        );
        emoji_keywords.insert(
            "📈".to_string(),
            vec![
                "chart increasing".to_string(),
                "growth".to_string(),
                "help".to_string(),
                "trending".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🌍".to_string(),
            vec![
                "globe".to_string(),
                "earth".to_string(),
                "world".to_string(),
                "planet".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🌎".to_string(),
            vec![
                "globe".to_string(),
                "earth".to_string(),
                "world".to_string(),
                "planet".to_string(),
            ],
        );
        emoji_keywords.insert(
            "🧮".to_string(),
            vec![
                "abacus".to_string(),
                "calculation".to_string(),
                "math".to_string(),
            ],
        );

        let mut keyword_most_relevant_emoji = HashMap::new();
        keyword_most_relevant_emoji.insert("amazing".to_string(), "🤩".to_string());
        keyword_most_relevant_emoji.insert("help".to_string(), "🤝".to_string());
        keyword_most_relevant_emoji.insert("hello".to_string(), "👋".to_string());
        keyword_most_relevant_emoji.insert("world".to_string(), "🌍".to_string());

        let mut word_to_top_1000_words_idx = HashMap::new();
        word_to_top_1000_words_idx.insert("help".to_string(), 50);
        word_to_top_1000_words_idx.insert("hello".to_string(), 150);

        let emoji_set: HashSet<String> = emoji_keywords.keys().cloned().collect();

        Self {
            emoji_keywords: Arc::new(emoji_keywords),
            keyword_most_relevant_emoji: Arc::new(keyword_most_relevant_emoji),
            emoji_glossary: Arc::new(HashMap::new()),
            emoji_set: Arc::new(emoji_set),
            word_to_top_1000_words_idx: Arc::new(word_to_top_1000_words_idx),
        }
    }
}

/// Load emoji data from embedded JSON files
pub fn load_emoji_data() -> Result<EmojiData> {
    info!("Loading emoji data from embedded resources");

    // Load data from embedded JSON files
    let emoji_keywords: EmojiKeywords =
        match serde_json::from_str::<HashMap<std::string::String, Vec<std::string::String>>>(
            include_str!("data/emoogle-emoji-keywords.json"),
        ) {
            Ok(data) => {
                info!("Loaded emoji keywords: {} entries", data.len());
                data
            }
            Err(e) => {
                error!("Failed to parse emoji keywords: {}", e);
                return Err(EmojiSearchError::Json(e));
            }
        };

    let keyword_most_relevant_emoji: KeywordMostRelevantEmoji = serde_json::from_str(
        include_str!("data/emoogle-keyword-most-relevant-emoji.json"),
    )?;

    let emoji_glossary: EmojiGlossary =
        serde_json::from_str(include_str!("data/emoogle-emoji-glossary.json"))?;

    let top_1000_words: Vec<String> =
        serde_json::from_str(include_str!("data/top-1000-words-by-frequency.json"))?;

    // Create emoji set from keys of emoji_keywords
    let emoji_set: HashSet<String> = emoji_keywords.keys().cloned().collect();

    // Create map from words to their index in top 1000 words
    let word_to_top_1000_words_idx: WordToTop1000WordsIdx = top_1000_words
        .iter()
        .enumerate()
        .map(|(idx, word)| (word.clone(), idx))
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
