use std::{fs, sync::Mutex};

pub static POPULAR_PATTERNS: Mutex<Vec<String>> = Mutex::new(vec![]);

// with permission from petra i scraped the forums
// i found the most popular patterns
// fun fact: somehow, Introspection is used 2067 times but Retrospection is used only 1757 times
// problem in my scraper? i dunno. One must imagine Luxof lazy.
pub fn init_patterns() {
    let file = fs::read_to_string("need/patterns.json")
        .expect("could not find patterns.json.");

    let mut patterns_array: Vec<String> = serde_json::from_str(file.as_str())
        .expect("could not parse patterns.json as an array of pattern names (strings).");

    let mut patterns = POPULAR_PATTERNS.lock().unwrap();
    patterns.clear();
    patterns.append(&mut patterns_array);
}

pub fn is_pattern(pattern: &String) -> bool {
    POPULAR_PATTERNS.lock().unwrap().contains(pattern)
}
