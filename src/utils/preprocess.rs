// src/utils/preprocess.rs
use tracing::trace;

/// Pre-process a keyword string to help with search
///
/// It performs the following operations:
/// - Remove """:;(),.!? characters
/// - Replace - with space
/// - Replace ' with '
/// - Convert to lowercase
pub fn pre_process_string(s: &str) -> String {
    trace!("Pre-processing string: {}", s);

    let mut result = s.to_lowercase();
    // Replace special characters
    result = result.replace(&['"', '"', ':', ';', '(', ')', ',', '.', '!', '?'][..], "");
    result = result.replace('-', " ");
    result = result.replace('\'', "'");

    trace!("Pre-processed result: {}", result);
    result
}
