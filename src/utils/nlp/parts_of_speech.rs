// src/utils/nlp/parts_of_speech.rs
use once_cell::sync::Lazy;
use std::collections::HashSet;
use tracing::trace;

/// Subject pronouns, object pronouns, possessive adjectives, etc.
static PRONOUNS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "i",
        "you",
        "he",
        "she",
        "it",
        "we",
        "they",
        "me",
        "you",
        "him",
        "her",
        "it",
        "us",
        "them",
        "my",
        "your",
        "his",
        "her",
        "its",
        "our",
        "their",
        "mine",
        "yours",
        "his",
        "hers",
        "its",
        "ours",
        "theirs",
        "myself",
        "yourself",
        "himself",
        "herself",
        "itself",
        "ourselves",
        "themselves",
        "yourselves",
        "this",
        "that",
        "these",
        "those",
        "who",
        "whom",
        "which",
        "what",
    ]
    .into_iter()
    .collect()
});

/// Common prepositions
static PREPOSITIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "about",
        "across",
        "after",
        "against",
        "along",
        "among",
        "around",
        "as",
        "at",
        "before",
        "behind",
        "beneath",
        "beside",
        "between",
        "beyond",
        "by",
        "despite",
        "during",
        "except",
        "for",
        "from",
        "in",
        "inside",
        "into",
        "near",
        "of",
        "on",
        "onto",
        "out",
        "outside",
        "over",
        "since",
        "than",
        "through",
        "throughout",
        "to",
        "toward",
        "under",
        "until",
        "upon",
        "via",
        "with",
        "within",
        "without",
    ]
    .into_iter()
    .collect()
});

/// Coordinating conjunctions
static CONJUNCTIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["for", "and", "nor", "but", "or", "yet", "so"]
        .into_iter()
        .collect()
});

/// Articles
static ARTICLES: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["a", "an", "the"].into_iter().collect());

/// Words that determine quantity
static PREDETERMINERS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["all", "both"].into_iter().collect());

/// Exceptions for predeterminers based on previous word
static PREDETERMINERS_EXCEPTIONS_PREVIOUS_WORDS: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ["calling"].into_iter().collect());

/// Other common words to filter out
static OTHERS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "is", "are", "was", "were", "if", "will", "would", "be", "being", "one", "have", "has",
        "had", "can", "more", "then", "do", "don't", "first", "even", "there", "only", "also",
        "such", "each", "because", "however", "very", "must", "due",
    ]
    .into_iter()
    .collect()
});

/// Filter out words that are pronouns, prepositions, conjunctions, articles or some others.
///
/// This helps focus search on meaningful content words rather than function words.
pub fn filter_parts_of_speech(words: &[String]) -> Vec<String> {
    trace!("Filtering parts of speech from: {:?}", words);

    let filtered = words
        .iter()
        .enumerate()
        .filter(|(idx, word)| {
            let previous_word = if *idx > 0 {
                Some(&words[idx - 1])
            } else {
                None
            };

            // Skip if it's a pronoun, preposition, etc.
            !(PRONOUNS.contains(word.as_str())
                || PREPOSITIONS.contains(word.as_str())
                || CONJUNCTIONS.contains(word.as_str())
                || ARTICLES.contains(word.as_str())
                || (PREDETERMINERS.contains(word.as_str())
                    && !(previous_word.is_some()
                        && PREDETERMINERS_EXCEPTIONS_PREVIOUS_WORDS
                            .contains(previous_word.unwrap().as_str())))
                || OTHERS.contains(word.as_str()))
        })
        .map(|(_, word)| word.clone())
        .collect();

    trace!("Filtered result: {:?}", filtered);
    filtered
}
