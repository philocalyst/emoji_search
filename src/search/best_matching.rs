// src/search/best_matching.rs
use crate::constants::{EmojiData, Options};
use crate::utils::nlp::parts_of_speech::filter_parts_of_speech;
use crate::utils::nlp::stemmer::stem_word;
use crate::utils::preprocess::pre_process_string;
use emojis::emoji::Emoji;
use std::cmp::Ordering;
use std::collections::HashSet;
use tracing::debug;

/// Attributes for ranking emojis in best matching search
#[derive(Debug, Clone)]
struct Attributes {
    num_exact_word_matches: usize,
    num_exact_stemmed_word_matches: usize,
    num_prefix_word_matches: usize,
    num_prefix_stemmed_word_matches: usize,
}

/// Search for best matching emojis for input with multiple words
///
/// This is a more forgiving search that also matches stemmed words
/// by stripping suffixes, and handles parts of speech filtering.
pub async fn match_emoji_to_words(
    input_words: &str,
    emoji_data: &EmojiData,
    options: &Options,
) -> Vec<Emoji> {
    debug!("Searching best matching emojis for: {}", input_words);

    // Create owned copies of the option values to avoid borrowing issues
    let custom_emoji_keywords = options.custom_emoji_keywords.clone().unwrap_or_default();

    // Pre-process and split input into words
    let input_words_array: Vec<String> = input_words.split(' ').map(|s| s.to_string()).collect();

    // Filter parts of speech to focus on content words
    let filtered_input_words = filter_parts_of_speech(&input_words_array);

    // Stem filtered words
    let stemmed_input_words: Vec<String> = filtered_input_words
        .iter()
        .map(|word| stem_word(word))
        .collect();

    let mut emojis_attributes: Vec<(&Emoji, Attributes)> = Vec::new();

    // Use rayon to process emojis in parallel
    use rayon::prelude::*;

    let emoji_data_ref = &emoji_data;
    let custom_emoji_keywords_ref = &custom_emoji_keywords;
    let filtered_input_words_ref = &filtered_input_words;
    let stemmed_input_words_ref = &stemmed_input_words;

    let parallel_results: Vec<_> = emoji_data_ref
        .emoji_keywords
        .par_iter()
        .filter_map(|(emoji, keywords)| {
            let all_keywords = if let Some(custom_kw) = custom_emoji_keywords_ref.get(emoji) {
                let mut combined = keywords.clone();
                combined.extend(custom_kw.clone());
                combined
            } else {
                keywords.clone()
            };

            let emoji_best_attributes = get_emoji_best_attributes(
                filtered_input_words_ref,
                stemmed_input_words_ref,
                &all_keywords,
            );

            emoji_best_attributes.map(|attrs| (emoji, attrs))
        })
        .collect();

    emojis_attributes.extend(parallel_results);

    // Sort emojis by attributes
    emojis_attributes.sort_by(|(_, a), (_, b)| compare_attributes(a, b));

    // Extract sorted emojis
    // Extract sorted emojis
    let results: Vec<Emoji> = emojis_attributes
        .into_iter()
        .map(|(emoji, _attributes)| emoji)
        .cloned()
        .collect();

    debug!("Found {} best matching emojis", results.len());
    results
}

/// Get the best attributes for the emoji based on its keywords matching against the input words
fn get_emoji_best_attributes(
    input_words_array: &[String],
    stemmed_input_words_array: &[String],
    keywords: &[String],
) -> Option<Attributes> {
    // Pre-process all keywords
    let processed_keywords: Vec<String> = keywords.iter().map(|k| pre_process_string(k)).collect();

    // Create a set of all words from keywords
    let jointed_keywords: String = processed_keywords.join(" ");
    let jointed_keywords_array: Vec<String> =
        jointed_keywords.split(' ').map(|s| s.to_string()).collect();
    let jointed_keywords_set: HashSet<String> = jointed_keywords_array.iter().cloned().collect();

    // Get match counts
    let attributes = get_num_matches(
        input_words_array,
        stemmed_input_words_array,
        &jointed_keywords_array,
        &jointed_keywords_set,
    );

    // Return attributes if there's any match
    if attributes.num_exact_word_matches > 0
        || attributes.num_exact_stemmed_word_matches > 0
        || attributes.num_prefix_word_matches > 0
        || attributes.num_prefix_stemmed_word_matches > 0
    {
        Some(attributes)
    } else {
        None
    }
}

/// Calculate the number of different types of matches between input words and keywords
fn get_num_matches(
    input_words_array: &[String],
    stemmed_input_words_array: &[String],
    keywords_array: &[String],
    keywords_set: &HashSet<String>,
) -> Attributes {
    let mut num_exact_word_matches = 0;
    let mut num_exact_stemmed_word_matches = 0;
    let mut num_prefix_word_matches = 0;
    let mut num_prefix_stemmed_word_matches = 0;

    // Check each input word against all keywords
    for (i, input_word) in input_words_array.iter().enumerate() {
        let stemmed_input_word = &stemmed_input_words_array[i];

        // Check for exact match with original word
        if keywords_set.contains(input_word) {
            num_exact_word_matches += 1;
        }
        // Check for exact match with stemmed word
        else if input_word != stemmed_input_word && keywords_set.contains(stemmed_input_word) {
            num_exact_stemmed_word_matches += 1;
        }
        // If no exact match, check for prefix matches
        else {
            let mut prefix_match_stemmed_word = false;

            for keyword in keywords_array {
                if keyword.starts_with(stemmed_input_word) {
                    prefix_match_stemmed_word = true;

                    // If keyword also starts with the original word, count that instead
                    if keyword.starts_with(input_word) {
                        num_prefix_word_matches += 1;
                        prefix_match_stemmed_word = false;
                        break;
                    }
                }
            }

            // Count stemmed word prefix match if there was no original word prefix match
            if prefix_match_stemmed_word {
                num_prefix_stemmed_word_matches += 1;
            }
        }
    }

    Attributes {
        num_exact_word_matches,
        num_exact_stemmed_word_matches,
        num_prefix_word_matches,
        num_prefix_stemmed_word_matches,
    }
}

/// Compare attributes for ranking best matching emojis
fn compare_attributes(a: &Attributes, b: &Attributes) -> Ordering {
    // Compare total exact matches (original + stemmed)
    let a_num_exact_matches = a.num_exact_word_matches + a.num_exact_stemmed_word_matches;
    let b_num_exact_matches = b.num_exact_word_matches + b.num_exact_stemmed_word_matches;

    if a_num_exact_matches != b_num_exact_matches {
        return b_num_exact_matches.cmp(&a_num_exact_matches);
    }

    // Compare exact matches of original words
    if a.num_exact_word_matches != b.num_exact_word_matches {
        return b.num_exact_word_matches.cmp(&a.num_exact_word_matches);
    }

    // Compare prefix matches of original words
    if a.num_prefix_word_matches != b.num_prefix_word_matches {
        return b.num_prefix_word_matches.cmp(&a.num_prefix_word_matches);
    }

    // Compare prefix matches of stemmed words
    if a.num_prefix_stemmed_word_matches != b.num_prefix_stemmed_word_matches {
        return b
            .num_prefix_stemmed_word_matches
            .cmp(&a.num_prefix_stemmed_word_matches);
    }

    Ordering::Equal
}
