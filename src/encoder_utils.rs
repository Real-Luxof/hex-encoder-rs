use crate::POPULAR_PATTERNS;


// woah, at least take me out to dinner first
// vscode autofill is the funniest shit ever: ", encoder_utils.rs, before you start writing all over me with your string processing and your binary conversions and your stateful encoder state and your pattern matching and your flatmaps and your chunking and your whatever the fuck else you have in store for me, encoder_utils.rs. at least let me put on a nice dress and do my hair before you start writing all over me with your string processing and your binary conversions and your stateful encoder state and your pattern matching and your flatmaps and your chunking and your whatever the"
trait STRIP {
    fn try_strip_prefix(&self, pattern: &str) -> String;

    fn try_strip_suffix(&self, pattern: &str) -> String;
}

pub trait Chunked<T> {
    fn next_chunk_of(&mut self, size: usize) -> Option<Vec<T>>;
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

// why am i magnetically attracted to string processing holy shit
/// For use in a flatmap.
pub fn encode_line(
    line_str: &String,
    line: usize
) -> Vec<String> {

    let mut line_patterns: Vec<String> = vec![];
    let mut buffer = String::new();
    let mut in_vec = false;

    let mut last_ch = ' ';
    // this is literally just to convert a line from
    // "<[ (0, 0, 1), 5 ]>" to ["<[", "<(0, 0, 1)>", "<5>", "]>"]
    for ch in line_str.chars() {

        let mut window = String::from(last_ch);
        window.push(ch);

        // do i dislike str processing because it looks so nonsensical
        // or am i just bad at it
        if window == "<[" {
            set_encoder_state(1, get_encoder_state(1) + 1);
            line_patterns.push(window);
            buffer = String::new();
            continue;

        } else if window == "]>" {
            if get_encoder_state(1) == 0 {
                panic!("Err at line {line}: list closed without matching left square bracket.");
            }
            set_encoder_state(1, get_encoder_state(1) - 1);

            buffer.truncate(buffer.len() - 1);
            let trimmed = buffer.trim();
            if get_pat_bin_optional(trimmed).is_none() {
                line_patterns.push(trimmed.to_string());
            } else if trimmed != "" {
                line_patterns.push(String::from("<") + trimmed + ">");
            }

            buffer = String::new();
            continue;

        } else if ch == '(' {
            in_vec = true;

            let trimmed = buffer.trim().to_string();
            if trimmed != "<" && trimmed != "" {
                line_patterns.push(trimmed);
                buffer = String::new();
            }
            buffer.push(ch);
            continue;

        } else if window == ")>" && get_encoder_state(1) == 0 && in_vec {
            in_vec = false;
            buffer.push(ch);
            line_patterns.push(buffer);
            buffer = String::new();
            continue;

        } else if ch == ')' && get_encoder_state(1) > 0 && in_vec {
            in_vec = false;
            buffer.push(ch);
            //line_patterns.push(String::from("<") + &buffer + ">");
            //buffer = String::new();
            continue;

        } else if ch == ',' && get_encoder_state(1) > 0 && !in_vec {
            if buffer.len() == 0 {
                panic!("Err at line {line}: unexpected comma.");
            }

            let trimmed = buffer.trim();
            if get_pat_bin_optional(trimmed).is_none() {
                line_patterns.push(trimmed.to_string());
            } else if trimmed != "" {
                line_patterns.push(String::from("<") + trimmed + ">");
            }
            buffer = String::new();
            continue;
        }

        last_ch = ch;

        buffer.push(ch);
    }

    if line_patterns.len() == 0 {
        line_patterns.push(line_str.clone());
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
    let mut op = vec![
        pad_0_upto(get_pat_bin("Introspection"), 8)
    ];
    op.append(&mut embed_num(num));
    op.push(pad_0_upto(get_pat_bin("Retrospection"), 8));
    op.push(pad_0_upto(get_pat_bin("Flock's Disintegration"), 8));
    return op;
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
        format!("{:064b}", num.to_bits())
        //pad_0_upto(num.to_bits() as usize, 16)
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
