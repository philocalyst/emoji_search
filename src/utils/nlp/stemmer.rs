use tracing::trace;

/// Custom rules for stemming with format (suffix, stemmed_suffix,
/// slice_position) The rules modify the stemming algorithm to work better with
/// emoji search
const CUSTOM_RULES: &[(&str, &str, Option<usize>)] = &[
	("y", "i", None),          // "happy" -> "happi" -> "happy"
	("Y", "i", None),          // "DIY" -> "DIi" -> "DIY"
	("ying", "i", Some(3)),    // "crying" -> "cri" -> "cry"
	("yings", "i", Some(4)),   // "carryings" -> "carri" -> "carry"
	("ing", "e", Some(3)),     // "smiling" -> "smile" -> "smil"
	("ings", "e", Some(4)),    // "codings" -> "code" -> "cod"
	("ingly", "e", Some(5)),   // "blazingly" -> "blaze" -> "blaz"
	("ility", "l", Some(4)),   // "disability" -> "disabl" -> "disabi"
	("ilities", "l", Some(6)), // "capabilities" -> "capabl" -> "capabi"
	("ys", "i", Some(1)),      // "candys" -> "candi" -> "candy"
	("est", "est", Some(3)),   // "coolest" -> "coolest" -> "cool"
];

/// Stem a word to its root form using a simplified algorithm with custom rules
///
/// This implementation provides functionality comparable to the Porter stemmer
/// but with custom rules to better support emoji search.
pub fn stem_word(word: &str) -> String {
	trace!("Stemming word: {}", word);

	// Apply basic stemming
	let strip_suffix = |suffix, min_len, extra_cond: bool| {
		word.strip_suffix(suffix).filter(|_| word.len() > min_len && extra_cond)
	};

	let stemmed = strip_suffix("ing", 0, true)
		.or_else(|| strip_suffix("ed", 3, true))
		.or_else(|| strip_suffix("s", 2, !word.ends_with("ss")))
		.or_else(|| strip_suffix("ly", 3, true))
		.unwrap_or(word);

	for &(word_suffix, stemmed_suffix, slice_end) in CUSTOM_RULES.iter() {
		if word.ends_with(word_suffix) && (stemmed.ends_with(stemmed_suffix) || word == stemmed) {
			// Resolve cut length. If Some(n) is too long for the word, skip this rule.
			let cut = match slice_end {
				Some(n) if word.len() > n => n,
				None => 0,
				_ => continue,
			};

			let result = word[..word.len() - cut].to_string();
			trace!("Stemmed result (custom rule): {} -> {}", word, result);
			return result;
		}
	}

	trace!("Stemmed result: {} -> {}", word, stemmed);
	stemmed.to_string()
}
