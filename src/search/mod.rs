// src/search/mod.rs
mod best_matching;
mod multiple_words;
mod single_word;

pub use best_matching::match_emoji_to_words;
pub use multiple_words::match_emojis_to_words_raw;
pub use single_word::match_emojis_to_word;
