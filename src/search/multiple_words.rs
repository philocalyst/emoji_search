// src/search/multiple_words.rs
use crate::constants::{EmojiData, Options};
use crate::utils::preprocess::pre_process_string;
use emojis::Emoji;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use tracing::{debug, trace};

/// Attributes for ranking emojis in multiple words search
#[derive(Debug, Clone)]
struct Attributes {
    is_multiple_words_keyword_match: bool,
    is_multiple_words_keyword_in_order_match: bool,
    is_multiple_words_keyword_in_order_match_exact_match: bool,
    is_custom_most_relevant_emoji: bool,
    num_exact_matches: usize,
    num_prefix_matches: usize,
    num_words_in_multiple_words_keyword: usize,
}

/// Search emojis for an input with multiple words, e.g. "smiling face"
pub async fn match_emojis_to_words_raw(
    input_words: &str,
    emoji_data: &EmojiData,
    options: &Options,
) -> Vec<&'static Emoji> {
    debug!("Searching emojis for multiple words input: {}", input_words);

    // Create owned copies of the option values to avoid borrowing issues
    let custom_emoji_keywords = options.custom_emoji_keywords.clone().unwrap_or_default();
    let custom_keyword_most_relevant_emoji = options
        .custom_keyword_most_relevant_emoji
        .clone()
        .unwrap_or_default();

    let input_words_array: Vec<String> = input_words.split(' ').map(|s| s.to_string()).collect();

    let mut emojis_attributes: Vec<(&Emoji, Attributes)> = Vec::new();

    // Use tokio tasks to process emojis in parallel
    let mut handles = Vec::new();

    for (emoji, keywords) in emoji_data.emoji_keywords.iter() {
        let emoji = emoji.clone();
        let keywords = keywords.clone();
        let custom_keywords = custom_emoji_keywords.get(&emoji).cloned();
        let custom_keyword_most_relevant_emoji = custom_keyword_most_relevant_emoji.clone();
        let input_words = input_words.to_string();
        let input_words_array = input_words_array.clone();

        let handle = tokio::spawn(async move {
            let all_keywords = if let Some(custom_kw) = custom_keywords {
                let mut combined = keywords;
                combined.extend(custom_kw);
                combined
            } else {
                keywords
            };

            let emoji_best_attributes = get_emoji_best_attributes(
                &input_words,
                &input_words_array,
                emoji,
                &all_keywords,
                &custom_keyword_most_relevant_emoji,
            );

            emoji_best_attributes.map(|attrs| (emoji, attrs))
        });

        handles.push(handle);
    }

    // Collect results from all tasks
    for handle in handles {
        if let Ok(Some((emoji, attributes))) = handle.await {
            emojis_attributes.push((emoji, attributes));
        }
    }

    // Sort emojis by attributes
    emojis_attributes.sort_by(|(_, a), (_, b)| compare_attributes(a, b));

    // Extract sorted emojis
    let results: Vec<&'static Emoji> = emojis_attributes
        .into_iter()
        .map(|(emoji, _attributes)| emoji)
        .collect();

    debug!(
        "Found {} matching emojis for multiple words input",
        results.len()
    );
    results
}

/// Get best attributes for emoji based on its keywords matching against input words
fn get_emoji_best_attributes(
    input_words: &str,
    input_words_array: &[String],
    emoji: &Emoji,
    keywords: &[String],
    custom_keyword_most_relevant_emoji: &HashMap<String, &'static Emoji>,
) -> Option<Attributes> {
    trace!(
        "Getting best attributes for emoji {} with multiple words input {}",
        emoji,
        input_words
    );

    let mut emoji_best_attributes: Option<Attributes> = None;

    // Pre-process keywords
    let processed_keywords: Vec<String> = keywords.iter().map(|k| pre_process_string(k)).collect();

    // First, check for multiple words keyword matches
    let multiple_words_keywords: Vec<String> = processed_keywords
        .iter()
        .filter(|k| k.contains(' '))
        .cloned()
        .collect();

    for keyword in multiple_words_keywords {
        // Check for exact in-order match
        if keyword == input_words {
            let is_custom_most_relevant_emoji =
                custom_keyword_most_relevant_emoji.get(&keyword) == Some(&emoji);

            let attributes = Attributes {
                is_multiple_words_keyword_match: true,
                is_multiple_words_keyword_in_order_match: true,
                is_multiple_words_keyword_in_order_match_exact_match: true,
                is_custom_most_relevant_emoji,
                num_exact_matches: 0,  // Not used in this context
                num_prefix_matches: 0, // Not used in this context
                num_words_in_multiple_words_keyword: 0, // Not used in this context
            };

            if emoji_best_attributes.is_none()
                || compare_attributes(&attributes, emoji_best_attributes.as_ref().unwrap())
                    == Ordering::Less
            {
                emoji_best_attributes = Some(attributes);
            }
        }
        // Check for partial in-order match
        else if keyword.starts_with(input_words) || keyword.contains(&format!(" {}", input_words))
        {
            let keyword_words_array: Vec<String> =
                keyword.split(' ').map(|s| s.to_string()).collect();

            let is_custom_most_relevant_emoji =
                custom_keyword_most_relevant_emoji.get(&keyword) == Some(&emoji);

            let attributes = Attributes {
                is_multiple_words_keyword_match: true,
                is_multiple_words_keyword_in_order_match: true,
                is_multiple_words_keyword_in_order_match_exact_match: false,
                is_custom_most_relevant_emoji,
                num_exact_matches: 0,  // Not used in this context
                num_prefix_matches: 0, // Not used in this context
                num_words_in_multiple_words_keyword: keyword_words_array.len(),
            };

            if emoji_best_attributes.is_none()
                || compare_attributes(&attributes, emoji_best_attributes.as_ref().unwrap())
                    == Ordering::Less
            {
                emoji_best_attributes = Some(attributes);
            }
        }
        // Check for out-of-order match
        else {
            let keyword_words_array: Vec<String> =
                keyword.split(' ').map(|s| s.to_string()).collect();

            // Skip if keyword has fewer words than input
            if keyword_words_array.len() < input_words_array.len() {
                continue;
            }

            let (num_exact_matches, num_prefix_matches) =
                get_num_matches(input_words_array, &keyword_words_array);

            // Skip if no matches found
            if num_exact_matches == 0 && num_prefix_matches == 0 {
                continue;
            }

            let attributes = Attributes {
                is_multiple_words_keyword_match: true,
                is_multiple_words_keyword_in_order_match: false,
                is_multiple_words_keyword_in_order_match_exact_match: false, // Not used in out-of-order match
                is_custom_most_relevant_emoji: false, // Not used in this context
                num_exact_matches,
                num_prefix_matches,
                num_words_in_multiple_words_keyword: keyword_words_array.len(),
            };

            if emoji_best_attributes.is_none()
                || compare_attributes(&attributes, emoji_best_attributes.as_ref().unwrap())
                    == Ordering::Less
            {
                emoji_best_attributes = Some(attributes);
            }
        }
    }

    // If no multiple words keyword match, check jointed keywords
    if emoji_best_attributes.is_none() {
        let jointed_keywords_set: HashSet<String> = processed_keywords
            .iter()
            .flat_map(|k| k.split(' ').map(|s| s.to_string()))
            .collect();

        let jointed_keywords_array: Vec<String> = jointed_keywords_set.into_iter().collect();

        let (num_exact_matches, num_prefix_matches) =
            get_num_matches(input_words_array, &jointed_keywords_array);

        if num_exact_matches > 0 || num_prefix_matches > 0 {
            let attributes = Attributes {
                is_multiple_words_keyword_match: false,
                is_multiple_words_keyword_in_order_match: false, // Not used in jointed match
                is_multiple_words_keyword_in_order_match_exact_match: false, // Not used in jointed match
                is_custom_most_relevant_emoji: false, // Not used in this context
                num_exact_matches,
                num_prefix_matches,
                num_words_in_multiple_words_keyword: 0, // Not used in jointed match
            };

            emoji_best_attributes = Some(attributes);
        }
    }

    emoji_best_attributes
}

/// Calculate the number of exact and prefix matches between input words and keywords
fn get_num_matches(input_words_array: &[String], keywords_array: &[String]) -> (usize, usize) {
    let mut num_exact_matches = 0;
    let mut num_prefix_matches = 0;

    // Check each input word against all keywords
    for input_word in input_words_array {
        let mut best_match_type: Option<MatchType> = None;

        for keyword in keywords_array {
            if keyword == input_word {
                best_match_type = Some(MatchType::Exact);
                break; // Found exact match - best possible outcome
            } else if keyword.starts_with(input_word) {
                best_match_type = Some(MatchType::Prefix);
                // Continue checking for potential exact match
            }
        }

        // If no match for this word, return zero for both counts
        if best_match_type.is_none() {
            return (0, 0);
        }

        match best_match_type.unwrap() {
            MatchType::Exact => num_exact_matches += 1,
            MatchType::Prefix => num_prefix_matches += 1,
        }
    }

    (num_exact_matches, num_prefix_matches)
}

enum MatchType {
    Exact,
    Prefix,
}

/// Compare attributes for ranking
///
/// This implements the multiple words search ranking algorithm with a tie-breaking strategy.
fn compare_attributes(a: &Attributes, b: &Attributes) -> Ordering {
    // 1. Multiple words keyword match is ranked higher than jointed keywords match
    match (
        a.is_multiple_words_keyword_match,
        b.is_multiple_words_keyword_match,
    ) {
        (true, false) => return Ordering::Less,
        (false, true) => return Ordering::Greater,
        _ => {}
    }

    if a.is_multiple_words_keyword_match {
        // Multiple words keyword match additional ranking criteria

        // 2. In-order match ranks higher than out-of-order match
        match (
            a.is_multiple_words_keyword_in_order_match,
            b.is_multiple_words_keyword_in_order_match,
        ) {
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            _ => {}
        }

        if a.is_multiple_words_keyword_in_order_match {
            // In-order match additional ranking criteria

            // 3. Exact match ranks higher than prefix match
            match (
                a.is_multiple_words_keyword_in_order_match_exact_match,
                b.is_multiple_words_keyword_in_order_match_exact_match,
            ) {
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                _ => {}
            }

            // 4. Custom most relevant emoji ranks higher
            match (
                a.is_custom_most_relevant_emoji,
                b.is_custom_most_relevant_emoji,
            ) {
                (true, false) => return Ordering::Less,
                (false, true) => return Ordering::Greater,
                _ => {}
            }
        } else {
            // Out-of-order match additional ranking criteria

            // 3. More exact matches rank higher
            if a.num_exact_matches != b.num_exact_matches {
                return b.num_exact_matches.cmp(&a.num_exact_matches);
            }

            // 4. More prefix matches rank higher
            if a.num_prefix_matches != b.num_prefix_matches {
                return b.num_prefix_matches.cmp(&a.num_prefix_matches);
            }
        }

        // 5. Fewer words in keyword ranks higher
        if a.num_words_in_multiple_words_keyword != b.num_words_in_multiple_words_keyword {
            return a
                .num_words_in_multiple_words_keyword
                .cmp(&b.num_words_in_multiple_words_keyword);
        }

        Ordering::Equal
    } else {
        // Jointed keywords match additional ranking criteria

        // 2. More exact matches rank higher
        if a.num_exact_matches != b.num_exact_matches {
            return b.num_exact_matches.cmp(&a.num_exact_matches);
        }

        // 3. More prefix matches rank higher
        if a.num_prefix_matches != b.num_prefix_matches {
            return b.num_prefix_matches.cmp(&a.num_prefix_matches);
        }

        Ordering::Equal
    }
}
