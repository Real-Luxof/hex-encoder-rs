use regex::Regex;
use std::{fs, sync::LazyLock};

const SPECIAL_HANDLER_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Bookkeeper's Gambit: [v-]+$|^Numerical Reflection: -?\d+(\.\d+)?$").unwrap());

// with permission from petra i scraped the forums and found the most popular patterns
// fun fact: somehow, Introspection is used 2067 times but Retrospection is used only 1757 times
// problem in my scraper? i dunno. One must imagine Luxof lazy.
pub const POPULAR_PATTERNS: LazyLock<Vec<String>> = LazyLock::new(
    || {
        serde_json::from_str(
            fs::read_to_string("needs/patterns.json")
                .expect("could not read patterns.json.")
                .as_str())
            .expect("could not parse patterns.json as an array of pattern names (strings).")
    }
);

pub fn is_pattern(pattern: &String) -> bool {
    POPULAR_PATTERNS.contains(pattern)
}

pub fn is_special_handler(pattern: &String) -> bool {
    SPECIAL_HANDLER_REGEX.is_match(pattern)
}

pub fn patterns_len() -> usize {
    POPULAR_PATTERNS.len()
}

pub fn patterns_bits() -> u32 {
    (patterns_len() as f64).log2().ceil() as u32
}
