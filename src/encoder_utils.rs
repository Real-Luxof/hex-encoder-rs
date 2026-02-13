use crate::POPULAR_PATTERNS;


trait STRIP {
    fn try_strip_prefix(&self, pattern: &str) -> String;

    fn try_strip_suffix(&self, pattern: &str) -> String;
}

trait Chunked<T> {
    fn next_chunk_of(&mut self, size: usize) -> Option<Vec<T>>;

    fn windows(&mut self, size: usize) -> Option<Vec<Vec<T>>>;
}

impl STRIP for String {
    fn try_strip_prefix(&self, pattern: &str) -> String {
        return self.strip_prefix(pattern).unwrap_or(self).to_string();
    }

    fn try_strip_suffix(&self, pattern: &str) -> String {
        return self.strip_suffix(pattern).unwrap_or(self).to_string();
    }
}

impl<T: Clone, I: Iterator<Item = T>> Chunked<T> for I {
    fn next_chunk_of(&mut self, size: usize) -> Option<Vec<T>> {
        let mut whole = vec![];
        
        for _ in 0..size {
            let next = self.next();
            if next.is_none() { return None; }
            else { whole.push(next.unwrap()); }
        }

        return Some(whole);
    }

    fn windows(&mut self, size: usize) -> Option<Vec<Vec<T>>> {
        let first_chunk = self.next_chunk_of(size);
        if first_chunk.is_none() { return None; }

        let mut chunks = vec![first_chunk.unwrap()];

        loop {
            let mut next_chunk = chunks.last().unwrap().to_vec();
            next_chunk.remove(0);

            let next = self.next();
            if next.is_none() {
                break;
            } else {
                next_chunk.push(next.unwrap());
            }
            chunks.push(next_chunk);
        }

        return Some(chunks);
    }
}


const END_OPCODE: &str = "00";
const NUMREF_OPCODE: &str = "01";
const BK_OPCODE: &str = "10";
const LIST_OPCODE: &str = "11";

/// stores in order:
/// - the indentation level (list)
/// - the list nest level (do not fw this it's used for line-processing)
static mut ENCODER_STATE: [usize; 2] = [0, 0];

pub fn get_encoder_state(idx: usize) -> usize {
    return unsafe { ENCODER_STATE }[idx];
}
pub fn set_encoder_state(
    idx: usize,
    value: usize
) {
    unsafe { ENCODER_STATE[idx] = value; }
}

pub fn end_op() -> Vec<String> { return with_null_byte(END_OPCODE.to_string()) }

pub fn with_null_byte(
    opcode: String
) -> Vec<String> {
    return vec![String::from("00000000"), opcode]
}

/// For use in a flatmap.
pub fn encode_line(
    line_str: &String,
    line: usize
) -> Vec<String> {

    let windows_option = line_str.chars().into_iter().windows(2);
    if windows_option.is_none() { return encode_pattern_8bit(line_str, line); }

    let mut line_patterns: Vec<String> = vec![];
    let mut buffer = String::new();

    for window_chars in windows_option.unwrap() {
        let window = window_chars[0].to_string() + &window_chars[1].to_string();

        if window == "<[" {
            set_encoder_state(1, get_encoder_state(1) + 1);
            line_patterns.push(window);
            continue;

        } else if window == "]>" {
            if get_encoder_state(1) == 0 {
                panic!("Err at line {line}: list closed without matching left square bracket.");
            }
            set_encoder_state(1, get_encoder_state(1) + 1);
            line_patterns.push(buffer);
            buffer = String::new();
            line_patterns.push(window);
            continue;

        } else if window_chars[1] == ',' && get_encoder_state(1) > 0 {
            buffer.push(window_chars[0]);
            line_patterns.push(buffer);
            buffer = String::new();
            line_patterns.push(String::from(","));
            continue;
        }

        buffer += &window;
    }

    return line_patterns.iter().flat_map(|p| encode_pattern_8bit(p, line)).collect();
}

/// For use in a flatmap.
pub fn encode_pattern_8bit(
    pattern: &String,
    line: usize
) -> Vec<String> {

    if *pattern == String::from("") { return vec![]; }
    let chunk_size = 8;

    if POPULAR_PATTERNS.contains(&pattern.as_str()) {
        return vec![pad_0_upto(get_pat_bin(pattern), chunk_size)];

    } else if pattern.starts_with("Numerical Reflection: ") {
        return make_numref_op(
            pattern
                .strip_prefix("Numerical Reflection: ")
                .unwrap()
                .parse()
                .unwrap()
        );
    } else if pattern.starts_with("Bookkeeper's Gambit: ") {
        return make_bk_op(
            pattern.strip_prefix("Bookkeeper's Gambit: ").unwrap(),
            chunk_size
        )
    } else if is_embedded_iota(pattern) || get_encoder_state(0) > 0 || pattern == "<[" || pattern == "]>" {

        let iota = get_embedded_iota(pattern);

        let num = iota
            .parse::<f64>()
            .map(|double| make_numref_op(double))
            .ok();

        let vec = remove_paren_or_blank(&iota)
            .replace(" ", "")
            .split(",")
            .filter_map(|s| s.parse::<f64>().ok())
            .next_chunk_of(3)
            .map(|v| embed_vec((v[0], v[1], v[2])));

        let list = try_embed_list(&iota, line);

        return num.unwrap_or_else( ||
            vec.unwrap_or_else( ||
            list.unwrap_or_else( ||
            panic!("Err at line {line}: Unsupported: {pattern}")
        )));

    } else {
        panic!("Err at line {line}: Unsupported: {pattern}");
    }
}

fn pad_0_upto(
    num: usize,
    at_least_size: usize
) -> String {
    let mut bin = format!("{:b}", num);
    if bin.len() >= at_least_size {
        return bin;
    }

    for _ in 0..(at_least_size - bin.len()) {
        bin.insert(0, '0');
    }
    return bin;
}

fn get_pat_bin(
    pattern: &str
) -> usize {
    return get_pat_bin_optional(pattern).expect(format!("\"{pattern}\" is not a pattern.").as_str());
}

fn get_pat_bin_optional(
    pattern: &str
) -> Option<usize> {
    return POPULAR_PATTERNS
        .iter()
        .position(|s| *s == pattern)
        .and_then(|p| Some(p + 1));
}

fn make_numref_op(
    num: f64
) -> Vec<String> {
    return vec![
        pad_0_upto(0, 8),
        NUMREF_OPCODE.to_string(),
        pad_0_upto(num as usize, 16)
    ];
}

fn make_bk_op(
    desired: &str,
    line: usize
) -> Vec<String> {
    return vec![
        pad_0_upto(0, 8),
        BK_OPCODE.to_string(),
        desired
            .chars()
            .into_iter()
            .map(|s| {
                match s {
                    'v' => "1",
                    '-' => "0",
                    _ => panic!("Err at line {line}: Unsupported character in Bookkeeper's Gambit notation: {s}")
                }
            })
            .collect()
    ];
}

fn embed_num(
    num: f64
) -> Vec<String> {
    return vec![
        pad_0_upto(0, 8),
        NUMREF_OPCODE.to_string(),
        pad_0_upto(num.to_bits() as usize, 16)
    ];
}

fn embed_vec(
    desired: (f64, f64, f64)
) -> Vec<String> {
    let mut ret = vec![];
    ret.push(pad_0_upto(get_pat_bin("Introspection"), 8));
    ret.append(&mut embed_num(desired.0));
    ret.append(&mut embed_num(desired.1));
    ret.append(&mut embed_num(desired.2));
    ret.push(pad_0_upto(get_pat_bin("Retrospection"), 8));
    ret.push(pad_0_upto(get_pat_bin("Flock's Disintegration"), 8));
    ret.push(pad_0_upto(get_pat_bin("Vector Exaltation"), 8));
    return ret;
}

fn try_embed_list(
    iota: &String,
    line: usize
) -> Option<Vec<String>> {
    let mut ret = None;

    if iota == "[" {
        ret = Some(vec![pad_0_upto(0, 8), LIST_OPCODE.to_string()]);
        set_encoder_state(0, get_encoder_state(0) + 1);

    } else if iota == "]" {
        if get_encoder_state(0) == 0 {
            panic!("Err at line {line}: list closed without matching left square bracket.");
        }
        ret = Some(vec![pad_0_upto(0, 8), END_OPCODE.to_string()]);
        set_encoder_state(0, get_encoder_state(0) - 1);
    }

    return ret;
}

fn is_embedded_iota(
    pattern: &String
) -> bool {
    return pattern.starts_with("<") && pattern.ends_with(">");
}

fn get_embedded_iota(
    pattern: &String
) -> String {
    return pattern
        .try_strip_prefix("<")
        .try_strip_suffix(">")
        .to_string();
}

fn remove_paren_or_blank(str: &String) -> String {
    str
        .strip_prefix("(")
        .and_then(|s| s
        .strip_suffix(")"))
        .map_or(String::new(), String::from)
}
