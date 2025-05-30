// src/constants.rs
use crate::error::{EmojiSearchError, FfiResult};
use bitcode::decode;
use emojis::emoji::Emoji;
use env_logger;
use log::info;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uniffi;

/// Map from emoji to its keywords
pub type EmojiKeywords = HashMap<Emoji, Vec<String>>;

/// Map from keyword to most relevant emoji
pub type KeywordMostRelevantEmoji = HashMap<String, Emoji>;

/// Map from keyword to emojis that match the keyword
pub type EmojiGlossary = HashMap<String, Vec<Emoji>>;

/// Map of words to their index in top 1000 words
pub type WordToTop1000WordsIdx = HashMap<String, usize>;

const EMOJI_KEYWORDS_BC: &[u8] = include_bytes!(concat!(
    env!("BITCODE_OUT_DIR"),
    "/emoogle_emoji_keywords.bc"
));
const KEYWORD_MOST_RELEVANT_EMOJI_BC: &[u8] = include_bytes!(concat!(
    env!("BITCODE_OUT_DIR"),
    "/emoogle_keyword_most_relevant_emoji.bc"
));
const EMOJI_GLOSSARY_BC: &[u8] = include_bytes!(concat!(
    env!("BITCODE_OUT_DIR"),
    "/emoogle_emoji_glossary.bc"
));
const TOP_1000_WORDS_BC: &[u8] =
    include_bytes!(concat!(env!("BITCODE_OUT_DIR"), "/top_1000_words.bc"));

/// Options for customizing emoji search
#[derive(Clone, uniffi::Record, Debug, Default)]
pub struct Options {
    /// Custom emoji keywords to extend built-in keywords
    pub custom_emoji_keywords: Option<EmojiKeywords>,

    /// Custom mappings from keywords to preferred emojis
    pub custom_keyword_most_relevant_emoji: Option<KeywordMostRelevantEmoji>,

    /// Recently searched inputs for improved search suggestions
    pub recently_searched_inputs: Option<Vec<String>>,
}

/// Core data structure containing all emoji data
#[derive(Clone, Debug, uniffi::Object)]
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
#[uniffi::export]
pub fn load_emoji_data() -> FfiResult<EmojiData> {
    use bitcode::decode;
    info!("Loading emoji data from embedded resources");

    let emoji_keywords: EmojiKeywords = decode(EMOJI_KEYWORDS_BC).expect("decode emoji keywords");
    let keyword_most_relevant_emoji: KeywordMostRelevantEmoji =
        decode(KEYWORD_MOST_RELEVANT_EMOJI_BC).expect("decode keyword→emoji");
    let emoji_glossary: EmojiGlossary = decode(EMOJI_GLOSSARY_BC).expect("decode glossary");
    let top_1000_words: Vec<String> = decode(TOP_1000_WORDS_BC).expect("decode top 1000 words");

    let emoji_set = emoji_keywords.keys().cloned().collect();

    // Create map from words to their index in top 1000 words
    let word_to_top_1000_words_idx: WordToTop1000WordsIdx = top_1000_words
        .iter()
        .enumerate()
        .map(|(idx, word)| (word.clone(), idx)) // Swap order and clone the String
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
