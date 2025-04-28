// src/search/mod.rs
mod best_matching;
mod multiple_words;
mod single_word;

pub use best_matching::search_best_matching_emojis_for_multiple_words;
pub use multiple_words::search_emojis_for_multiple_words_input;
pub use single_word::search_emojis_for_single_word_input;
