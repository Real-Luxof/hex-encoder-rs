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

pub fn patterns_len() -> usize {
    POPULAR_PATTERNS.lock().unwrap().len()
}

pub fn patterns_bits() -> usize {
    (patterns_len() as f64).log2().ceil() as usize
}

/// yes, if you remove intro, retro, or flock disint this always returns false.
pub fn big_enough_for_embeds(bits: u32) -> bool {
    big_enough_for(
        bits,
        vec!["Introspection", "Retrospection", "Flock's Disintegration"]
    )
}

/// yes, if you remove intro, retro, flock disint, or vec exalt this always returns false.
pub fn big_enough_for_vecs(bits: u32) -> bool {
    big_enough_for(
        bits,
        vec![
            "Introspection", "Retrospection", "Flock's Disintegration", "Vector Exaltation"
        ]
    )
}

/// if the patterns are not in the pattern list this always returns false.
pub fn big_enough_for(
    bits: u32,
    unfulfilled: Vec<&str>
) -> bool {
    let range = 2_i32.pow(bits);
    let patterns = POPULAR_PATTERNS.lock().unwrap();

    let required = unfulfilled.len();
    let mut fulfilled = 0;

    for unfulfilled_pattern in unfulfilled {

        let mut idx = -1;
        for pattern in patterns.iter() {
            idx += 1;
            if pattern.as_str() != unfulfilled_pattern { continue; }
            if idx > range { return false; }
            fulfilled += 1;
            break;
        }
    }
    return fulfilled == required;
}
