use std::{collections::HashSet, sync::LazyLock};

use tracing::trace;

/// Subject pronouns, object pronouns, possessive adjectives, etc.
static PRONOUNS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
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
static PREPOSITIONS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
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
static CONJUNCTIONS: LazyLock<HashSet<&'static str>> =
	LazyLock::new(|| ["for", "and", "nor", "but", "or", "yet", "so"].into_iter().collect());

/// Articles
static ARTICLES: LazyLock<HashSet<&'static str>> =
	LazyLock::new(|| ["a", "an", "the"].into_iter().collect());

/// Words that determine quantity
static PREDETERMINERS: LazyLock<HashSet<&'static str>> =
	LazyLock::new(|| ["all", "both"].into_iter().collect());

/// Exceptions for predeterminers based on previous word
static PREDETERMINERS_EXCEPTIONS_PREVIOUS_WORDS: LazyLock<HashSet<&'static str>> =
	LazyLock::new(|| ["calling"].into_iter().collect());

/// Other common words to filter out
static OTHERS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
	[
		"is", "are", "was", "were", "if", "will", "would", "be", "being", "one", "have", "has", "had",
		"can", "more", "then", "do", "don't", "first", "even", "there", "only", "also", "such", "each",
		"because", "however", "very", "must", "due",
	]
	.into_iter()
	.collect()
});

/// Filter out words that are pronouns, prepositions, conjunctions, articles or
/// some others.
///
/// This helps focus search on meaningful content words rather than function
/// words.
pub fn filter_parts_of_speech(words: &[&str]) -> Vec<String> {
	trace!("Filtering parts of speech from: {:?}", words);

	let filtered = words
		.iter()
		.enumerate()
		.filter(|(idx, word)| {
			let previous_word = if *idx > 0 { Some(&words[idx - 1]) } else { None };

			// Skip if it's a pronoun, preposition, etc.
			!(PRONOUNS.contains(*word)
				|| PREPOSITIONS.contains(*word)
				|| CONJUNCTIONS.contains(*word)
				|| ARTICLES.contains(*word)
				|| (PREDETERMINERS.contains(*word)
					&& !(previous_word.is_some()
						&& PREDETERMINERS_EXCEPTIONS_PREVIOUS_WORDS.contains(previous_word.unwrap())))
				|| OTHERS.contains(*word))
		})
		.map(|(_, word)| word.clone().to_owned())
		.collect();

	trace!("Filtered result: {:?}", filtered);
	filtered
}
