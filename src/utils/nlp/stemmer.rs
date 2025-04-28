// src/utils/nlp/stemmer.rs
use once_cell::sync::Lazy;
use tracing::trace;

/// Custom rules for stemming with format (suffix, stemmed_suffix, slice_position)
/// The rules modify the stemming algorithm to work better with emoji search
static CUSTOM_RULES: Lazy<Vec<(&'static str, &'static str, Option<usize>)>> = Lazy::new(|| {
    vec![
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
    ]
});

/// Stem a word to its root form using a simplified algorithm with custom rules
///
/// This implementation provides functionality comparable to the Porter stemmer
/// but with custom rules to better support emoji search.
pub fn stem_word(word: &str) -> String {
    trace!("Stemming word: {}", word);

    // Apply basic stemming
    let mut stemmed = word.to_string();

    // Remove common suffixes
    if stemmed.ends_with("ing") {
        stemmed = stemmed[0..stemmed.len() - 3].to_string();
    } else if stemmed.ends_with("ed") && stemmed.len() > 3 {
        stemmed = stemmed[0..stemmed.len() - 2].to_string();
    } else if stemmed.ends_with("s") && !stemmed.ends_with("ss") && stemmed.len() > 2 {
        stemmed = stemmed[0..stemmed.len() - 1].to_string();
    } else if stemmed.ends_with("ly") && stemmed.len() > 3 {
        stemmed = stemmed[0..stemmed.len() - 2].to_string();
    }

    // Apply custom rules
    for &(word_suffix, stemmed_suffix, slice_end) in CUSTOM_RULES.iter() {
        if word.ends_with(word_suffix) && (stemmed.ends_with(stemmed_suffix) || word == stemmed) {
            if let Some(end) = slice_end {
                if word.len() > end {
                    let result = word[0..word.len() - end].to_string();
                    trace!("Stemmed result (custom rule): {} -> {}", word, result);
                    return result;
                }
            } else {
                trace!("Stemmed result (custom rule): {} -> {}", word, word);
                return word.to_string();
            }
        }
    }

    trace!("Stemmed result: {} -> {}", word, stemmed);
    stemmed
}
