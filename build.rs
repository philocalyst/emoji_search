use emojis::emoji::Emoji;
use std::{collections::HashMap, env, fs, path::Path};

use bitcode::encode;
use serde_json::from_str;

/// Map from emoji to its keywords
pub type EmojiKeywords = HashMap<Emoji, Vec<String>>;

/// Map from keyword to most relevant emoji
pub type KeywordMostRelevantEmoji = HashMap<String, Emoji>;

/// Map from keyword to emojis that match the keyword
pub type EmojiGlossary = HashMap<String, Vec<Emoji>>;

/// Map of words to their index in top 1000 words
pub type WordToTop1000WordsIdx = HashMap<String, usize>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Where we live, 2) where to put our .bc files:
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let out_dir = env::var("OUT_DIR")?;
    let data_dir = Path::new(&manifest_dir).join("data");

    // A little helper to parse JSON, encode into bitcode, write to $OUT_DIR.
    let mut do_one = |json_name: &str,
                      bc_name: &str,
                      parser: &dyn Fn(&str) -> Vec<u8>|
     -> Result<(), Box<dyn std::error::Error>> {
        let src = data_dir.join(json_name);
        println!("cargo:rerun-if-changed={}", src.display());
        let txt = fs::read_to_string(&src)?;
        let bc = parser(&txt);
        let dst = Path::new(&out_dir).join(bc_name);
        fs::write(dst, bc)?;
        Ok(())
    };

    // emoji‐keywords.json → EmojiKeywords → .bc blob
    do_one(
        "emoogle-emoji-keywords.json",
        "emoogle_emoji_keywords.bc",
        &|s| {
            let v: EmojiKeywords = from_str(s).unwrap();
            encode(&v)
        },
    )?;

    // keyword‐most‐relevant‐emoji.json → KeywordMostRelevantEmoji → .bc blob
    do_one(
        "emoogle-keyword-most-relevant-emoji.json",
        "emoogle_keyword_most_relevant_emoji.bc",
        &|s| {
            let v: KeywordMostRelevantEmoji = from_str(s).unwrap();
            encode(&v)
        },
    )?;

    // emoji‐glossary.json → EmojiGlossary → .bc blob
    do_one(
        "emoogle-emoji-glossary.json",
        "emoogle_emoji_glossary.bc",
        &|s| {
            let v: EmojiGlossary = from_str(s).unwrap();
            encode(&v)
        },
    )?;

    // top-1000-words.json → Vec<String> → .bc blob
    do_one(
        "top-1000-words-by-frequency.json",
        "top_1000_words.bc",
        &|s| {
            let v: Vec<String> = from_str(s).unwrap();
            encode(&v)
        },
    )?;

    // Finally tell rustc where to find our blobs at compile time:
    println!("cargo:rustc-env=BITCODE_OUT_DIR={}", out_dir);

    Ok(())
}
