use core::f64;

use crate::patterns::{POPULAR_PATTERNS, big_enough_for_embeds, big_enough_for_vecs, is_pattern};
use crate::used_types::{Chunked, STRIP};


const END_OPCODE: &str = "000";
const EMBED_NUM_OPCODE_I8: &str = "001";
const EMBED_NUM_OPCODE_F32: &str = "010";
const EMBED_NUM_OPCODE_DOUBLE: &str = "011";
const BK_OPCODE_4BIT: &str = "100";
const BK_OPCODE_8BIT: &str = "101";
const BK_OPCODE_16BIT: &str = "110";
const LIST_OPCODE: &str = "111";

/// stores in order:
/// - the indentation level (list)
/// - the embedded list nest level (do not fw this it's used for line pre-processing)
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

pub fn end_op(chunk_size: u32) -> Vec<String> {
    with_null_chunk(END_OPCODE.to_string(), chunk_size)
}

pub fn with_null_chunk(
    opcode: String,
    chunk_size: u32
) -> Vec<String> {
    let mut null_chunk = String::new();
    for _ in 0..chunk_size { null_chunk.push('0'); }
    return vec![null_chunk, opcode];
}

// why am i magnetically attracted to string processing holy shit
/// For use in a flatmap.
pub fn encode_line(
    line_str: &String,
    line: usize,
    chunk_size: u32
) -> Option<Vec<String>> {

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

    return line_patterns.iter()
        .map(|p| encode_pattern(p, line, chunk_size))
        .collect::<Option<Vec<Vec<String>>>>()
        .map(|v| v.into_iter().flatten().collect());
}

/// For use in a flatmap.
pub fn encode_pattern(
    pattern: &String,
    line: usize,
    chunk_size: u32
) -> Option<Vec<String>> {

    if *pattern == String::from("") { return Some(vec![]); }

    if is_pattern(&pattern) {
        let padded = pad_0_upto_maybe(get_pat_bin(pattern), chunk_size);
        return padded.map(|o| vec![o]);

    } else if pattern.starts_with("Numerical Reflection: ") {
        if !big_enough_for_embeds(chunk_size) { return None; }

        return Some(make_numref(
            pattern[22..]
                .parse()
                .unwrap(),
            chunk_size
        ));

    } else if pattern.starts_with("Bookkeeper's Gambit: ") {
        return Some(make_bk_op(
            &pattern[21..],
            line,
            chunk_size
        ));

    } else if is_embedded_iota(pattern) || get_encoder_state(0) > 0 || pattern == "<[" || pattern == "]>" {
        if !big_enough_for_embeds(chunk_size) { return None }

        let iota = get_embedded_iota(pattern);

        let num = iota
            .parse::<f64>()
            .map(|double| make_numref(double, chunk_size))
            .ok();

        let vec;
        if remove_paren_or_blank(&iota) != "" {
            if !big_enough_for_vecs(chunk_size) { return None; }

            vec = remove_paren_or_blank(&iota)
                .replace(" ", "")
                .split(",")
                .filter_map(|s| s.parse::<f64>().ok())
                .next_chunk_of(3)
                .map(|v| embed_vec((v[0], v[1], v[2]), chunk_size));

        } else {
            vec = None;
        }

        let list = try_embed_list(&iota, line, chunk_size);

        return Some(num.unwrap_or_else( ||
            vec.unwrap_or_else( ||
            list.unwrap_or_else( ||
            panic!("Err at line {line}: Unsupported: {pattern}")
        ))));

    } else {
        panic!("Err at line {line}: Unsupported: {pattern}");
    }
}

fn pad_0_upto(
    num: usize,
    chunk_size: u32
) -> String {
    pad_0_upto_maybe(num, chunk_size).unwrap()
}

fn pad_0_upto_maybe(
    num: usize,
    chunk_size: u32
) -> Option<String> {
    let mut bin = format!("{:b}", num);
    let bits = bin.len();
    if bits > chunk_size as usize {
        return None;
    } else if bits == chunk_size as usize {
        return Some(bin);
    }

    for _ in 0..(chunk_size - bits as u32) {
        bin.insert(0, '0');
    }
    return Some(bin);
}

fn get_pat_bin(
    pattern: &str
) -> usize {
    return get_pat_bin_optional(pattern).expect(format!("\"{pattern}\" is not a pattern.").as_str());
}

fn get_pat_bin_optional(
    pattern: &str
) -> Option<usize> {
    return POPULAR_PATTERNS.lock().unwrap()
        .iter()
        .position(|s| *s == pattern)
        .and_then(|p| Some(p + 1));
}

fn make_numref(
    num: f64,
    chunk_size: u32
) -> Vec<String> {
    let mut op = vec![
        pad_0_upto(get_pat_bin("Introspection"), chunk_size)
    ];
    op.append(&mut embed_num(num, chunk_size));
    op.push(pad_0_upto(get_pat_bin("Retrospection"), chunk_size));
    op.push(pad_0_upto(get_pat_bin("Flock's Disintegration"), chunk_size));
    return op;
}

fn make_bk_op(
    desired: &str,
    line: usize,
    chunk_size: u32
) -> Vec<String> {
    let mut bookkeeper: Vec<char> = desired.chars().collect();
    let mut len: usize = desired.chars().count();

    let bookkeeper_opcode;
    if len < 4 {
        len = 4;
        bookkeeper_opcode = BK_OPCODE_4BIT;
    } else if len < 8 {
        len = 8;
        bookkeeper_opcode = BK_OPCODE_8BIT;
    } else if len < 16 {
        len = 16;
        bookkeeper_opcode = BK_OPCODE_16BIT;
    } else {
        // Matt, is that you?
        panic!("Exceeded max Bookkeeper's Gambit length (maximum = 16, found size = {len}) at line {line}");
    }

    while bookkeeper.len() < len {
        bookkeeper.insert(0, '-');
    }
    vec![
        pad_0_upto(0, chunk_size),
        bookkeeper_opcode.into(),
        bookkeeper
            .iter()
            .map(|s| {
                match s {
                    'v' => "1",
                    '-' => "0",
                    _ => panic!("Err at line {line}: Unsupported character in Bookkeeper's Gambit notation: {s}")
                }
            })
            .collect()
    ]
}

fn embed_num(
    num: f64,
    chunk_size: u32
) -> Vec<String> {
    let opcode;
    let num_bin;
    if (num % 1.0).abs() <= f64::MIN_POSITIVE && num <= 255.0 {
        opcode = EMBED_NUM_OPCODE_I8;
        num_bin = format!("{:08b}", num as i8);
    } else if is_f32(num) {
        opcode = EMBED_NUM_OPCODE_F32;
        num_bin = format!("{:032b}", (num as f32).to_bits());
    } else {
        opcode = EMBED_NUM_OPCODE_DOUBLE;
        num_bin = format!("{:064b}", num.to_bits());
    }
    vec![
        pad_0_upto(0, chunk_size),
        opcode.into(),
        num_bin
    ]
}

fn is_f32(
    num: f64
) -> bool {
    (num - (num as f32 as f64)).abs() <= f64::MIN_POSITIVE
}

fn embed_vec(
    desired: (f64, f64, f64),
    chunk_size: u32
) -> Vec<String> {
    let mut ret = vec![
        pad_0_upto(get_pat_bin("Introspection"), chunk_size)
    ];
    ret.append(&mut embed_num(desired.0, chunk_size));
    ret.append(&mut embed_num(desired.1, chunk_size));
    ret.append(&mut embed_num(desired.2, chunk_size));
    ret.append(&mut vec![
        pad_0_upto(get_pat_bin("Retrospection"), chunk_size),
        pad_0_upto(get_pat_bin("Flock's Disintegration"), chunk_size),
        pad_0_upto(get_pat_bin("Vector Exaltation"), chunk_size)
    ]);
    return ret;
}

fn try_embed_list(
    iota: &String,
    line: usize,
    chunk_size: u32
) -> Option<Vec<String>> {
    let mut ret = None;

    if iota == "[" {
        ret = Some(vec![pad_0_upto(0, chunk_size), LIST_OPCODE.to_string()]);
        set_encoder_state(0, get_encoder_state(0) + 1);

    } else if iota == "]" {
        if get_encoder_state(0) == 0 {
            panic!("Err at line {line}: list closed without matching left square bracket.");
        }
        ret = Some(vec![pad_0_upto(0, chunk_size), END_OPCODE.to_string()]);
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
