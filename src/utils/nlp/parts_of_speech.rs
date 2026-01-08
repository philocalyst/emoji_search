use std::collections::HashSet;

use tracing::trace;

/// Subject pronouns, object pronouns, possessive adjectives, etc.
const PRONOUNS: &[&str] = &[
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
];

/// Common prepositions
const PREPOSITIONS: &[&str] = &[
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
];

/// Coordinating conjunctions
const CONJUNCTIONS: &[&str] = &["for", "and", "nor", "but", "or", "yet", "so"];

/// Articles
const ARTICLES: &[&str] = &["a", "an", "the"];

/// Words that determine quantity
const PREDETERMINERS: &[&str] = &["all", "both"];

/// Exceptions for predeterminers based on previous word
const PREDETERMINERS_EXCEPTIONS_PREVIOUS_WORDS: &[&str] = &["calling"];

/// Other common words to filter out
const OTHERS: &[&str] = &[
	"is", "are", "was", "were", "if", "will", "would", "be", "being", "one", "have", "has", "had",
	"can", "more", "then", "do", "don't", "first", "even", "there", "only", "also", "such", "each",
	"because", "however", "very", "must", "due",
];

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
			!(PRONOUNS.contains(word)
				|| PREPOSITIONS.contains(word)
				|| CONJUNCTIONS.contains(word)
				|| ARTICLES.contains(word)
				|| (PREDETERMINERS.contains(word)
					&& !(previous_word.is_some()
						&& PREDETERMINERS_EXCEPTIONS_PREVIOUS_WORDS.contains(previous_word.unwrap())))
				|| OTHERS.contains(word))
		})
		.map(|(_, word)| (*word).to_owned())
		.collect();
	trace!("Filtered result: {:?}", filtered);
	filtered
}
