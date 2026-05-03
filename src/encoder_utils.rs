use core::f64;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::LazyLock;

use crate::patterns::{POPULAR_PATTERNS, is_pattern, is_special_handler};
use crate::used_types::{EncodingError, STRIP};

const ENCODING_VERSION: LazyLock<String> = LazyLock::new(|| String::from("00000000"));
const EMBEDDED_IOTA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:<\[(?:.|\n)*\]>|<-?\d+(\.\d+)?>|<\(\s*-?\d+(\.\d+)?\s*,\s*-?\d+(\.\d+)?\s*,\s*-?\d+(\.\d+)?\s*\)>)$").unwrap());
const EMBEDDED_LIST_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<\[(?:.|\n)*\]>").unwrap());
const IOTA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:\[(?:.|\n)*\]|\(\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*\)|-?\d+(?:\.\d+)?)").unwrap());
const LIST_IOTA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\[(?:.|\n)*\]$").unwrap());
const VEC_IOTA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\(\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*\)$").unwrap());
const NUM_IOTA_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?\d+(?:\.\d+)?$").unwrap());
const BK_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Bookkeeper's Gambit: [v-]+$").unwrap());
const NUMREF_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^Numerical Reflection: -?\d+(?:\.\d+)?$").unwrap());
const END_OPCODE: &str = "000";
const EMBED_NUM_OPCODE_I8: &str = "001";
const EMBED_NUM_OPCODE_F32: &str = "010";
const EMBED_NUM_OPCODE_DOUBLE: &str = "011";
const BK_OPCODE_4BIT: &str = "100";
const BK_OPCODE_16BIT: &str = "101";
const EMBED_VEC_OPCODE: &str = "110";
const EMBED_VEC_I8_OPCODE: &str = "00";
const EMBED_VEC_F32_OPCODE: &str = "01";
const EMBED_VEC_DOUBLE_OPCODE: &str = "10";
const LIST_OPCODE: &str = "111";

pub fn tokenize_file(
    file: String,
) -> Result<Vec<String>, EncodingError> {
    let mut tokens: Vec<String> = vec![];

    let replaced_file = file
        .replace("Consideration: ", "Consideration\n")
        .replace("Introspection", "{")
        .replace("Retrospection", "}");

    // once again, why do i have to bind the regex
    let mut preprocessed_file = {
        let mut last_end = 0;
        let mut parts = vec![];
        EMBEDDED_LIST_REGEX.find_iter(&replaced_file)
            .for_each(|m| {
                parts.append(&mut replaced_file[last_end..m.start()].split("\n").collect());
                parts.push(replaced_file[m.start()..m.end()+1].trim());
                last_end = m.end();
            });
        parts.append(&mut replaced_file[last_end..replaced_file.len()].split("\n").collect());
        parts.into_iter()
    };

    let mut nested = 0;
    // hey girl, are you PCRE2? 'cause there's nothing regular aboutcha!
    while let Some(word_str) = preprocessed_file.next() {
        let word = word_str.to_string();

        if is_pattern(&word) || is_special_handler(&word) || is_embedded_iota(&word) {
            tokens.push(word);
            continue;
        }


        match word_str {
            "{}" => {
                tokens.push("Introspection".into());
                tokens.push("Retrospection".into());
            },
            "{" => {
                nested += 1;
                tokens.push("Introspection".into());
            },
            "}" => {
                if nested == 0 {
                    return Err(EncodingError {
                        msg: "Unbalanced intro-retro - too many Retrospections".to_string()
                    });
                }
                nested -= 1;
                tokens.push("Retrospection".into());
            },
            "<[" => {
                return Err(EncodingError {
                    msg: String::from(
                        "Unbalanced embedded list iotas - too many embedded list iota openers (]>)."
                    )
                });
            },
            "]>" => {
                return Err(EncodingError {
                    msg: String::from(
                        "Unbalanced embedded list iotas - too many embedded list iota closers (]>)."
                    )
                });
            }
            "" => {}
            _ => {
                if word_str.starts_with("<[") {
                    tokens.push(word);
                } else {
                    return Err(EncodingError {
                        msg: format!("Invalid line: \"{word}\"")
                    });
                }
            }
        }
    }

    if nested > 0 {
        return Err(EncodingError {
            msg: "Unbalanced intro-retro - too many Introspections".to_string()
        });
    }

    return Ok(tokens);
}

pub fn find_unique_patterns(
    tokens: &Vec<String>
) -> HashSet<String> {
    let mut unique_patterns: Vec<String> = tokens.into_iter()
        .filter(|p| is_pattern(p))
        .map(|p| p.clone())
        .collect();

    if tokens.into_iter().find(|p| p.starts_with("Numerical Reflection: ")).is_some() {
        unique_patterns.push("Introspection".into());
        unique_patterns.push("Retrospection".into());
        unique_patterns.push("Flock's Disintegration".into());
    }

    unique_patterns.into_iter().collect()
}

/// The first element of the tuple is the binary output op-by-op, and the second is the local mappings.
pub fn tokens_to_binary(
    tokens: &Vec<String>,
    unique_patterns: &HashSet<String>,
    chunk_size: u32
) -> Result<(Vec<String>, Vec<String>), EncodingError> {
    let addresses = (2_isize.pow(chunk_size) - 2).max(0) as usize;

    let mut local_mappings: Vec<String> = unique_patterns.iter()
        .filter(|p| get_pat_bin(p) >= addresses)
        .map(|p| p.clone())
        .collect();

    let local_mappings_start_at = {
        let threshold = addresses - local_mappings.len();
        let mut extras = 0;

        unique_patterns.iter()
            .filter(|p| get_pat_bin(p) >= threshold)
            .for_each(|p| {
                extras += 1;
                local_mappings.push(p.clone());
            });
        threshold + extras
    };

    // to not need to tell the decoder what address we assigned to the local mappings
    local_mappings.sort_by(|p1, p2| {
        let p1bin = get_pat_bin(p1);
        let p2bin = get_pat_bin(p2);
        // it's easier on the decoder if we sort in descending order rather than ascending
        if p1bin == p2bin {
            Ordering::Equal
        } else if p1bin < p2bin {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    });

    let mut binary: Vec<Vec<String>> = vec![vec![
        ENCODING_VERSION.clone(),
        pad_0_upto(chunk_size as usize, 4)
    ]];
    binary.push({
        let mut push = vec![pad_0_upto(local_mappings.len(), 8)];
        push.append(
            &mut local_mappings.iter()
                .map(|p| pad_0_upto(get_pat_bin(p), 8))
                .collect()
        );
        push
    });

    for token in tokens {
        if let Some(pat_bin) = get_pat_bin_optional(token) {
            // just trust me
            binary.push(vec![pad_0_upto(pat_bin, chunk_size)]);

        } else if token.starts_with("<") && token.ends_with(">") {
            binary.push(iota_to_binary(token, chunk_size)?);

        } else if token.starts_with("Bookkeeper's Gambit: ") {
            binary.push(make_bk_op(token, chunk_size)?);

        } else if token.starts_with("Numerical Reflection: ") {
            binary.push(make_numref(token, chunk_size)?);

        } else {
            return Err(EncodingError {
                msg: format!("Invalid: {token}")
            });
        }
    }
    binary.push(vec![
        pad_0_upto(0, chunk_size),
        END_OPCODE.to_string()
    ]);

    Ok((
        binary.into_iter()
            .flat_map(|v|
                if v.len() > 1 { v }
                else {
                    if let Some(i) = local_mappings.iter().position(|a| a == &v[0]) {
                        vec![pad_0_upto(i + local_mappings_start_at, chunk_size)]
                    } else {
                        v
                    }
                }
            )
            .collect(),
        local_mappings
    ))
}

fn iota_to_binary(
    token: &String,
    chunk_size: u32
) -> Result<Vec<String>, EncodingError> {
    let t = token.try_strip_prefix("<").try_strip_suffix(">");
    if LIST_IOTA_REGEX.is_match(&t) {
        list_to_binary(
            t.try_strip_prefix("[").try_strip_suffix("]"),
            chunk_size
        )
    } else if let Some(m) = VEC_IOTA_REGEX.captures(&t) {
        Ok(embed_vec(
            (
                m.get(1).unwrap().as_str().parse().unwrap(),
                m.get(2).unwrap().as_str().parse().unwrap(),
                m.get(3).unwrap().as_str().parse().unwrap()
            ),
            chunk_size
        ))
    } else if NUM_IOTA_REGEX.is_match(&t) {
        Ok(embed_num(t.parse().unwrap(), chunk_size))
    } else {
        Err(EncodingError {
            msg: format!("\"{t}\" is not a valid embedded iota.")
        })
    }
}

fn list_to_binary(
    tokens: String,
    chunk_size: u32
) -> Result<Vec<String>, EncodingError> {
    let mut binary: Vec<String> = vec![
        pad_0_upto(0, chunk_size),
        LIST_OPCODE.to_string()
    ];

    let mut nest = 0;
    let mut part: String = String::new();
    for token in (tokens
        .try_strip_suffix(",")
        + "," /* scuffed lmao */)
        .chars()
        .flat_map(|c| {
            if c == '[' {
                nest += 1;
                if nest > 1 { vec![] }
                else {
                    let fin = part.clone();
                    part = String::from(c);
                    vec![fin]
                }
            } else if c == ']' {
                nest -= if nest == 0 { 0 } else { 1 };
                part.push(c);
                if nest > 0 { vec![] }
                else {
                    let fin = part.clone();
                    part = String::new();
                    vec![fin]
                }
            } else if c == ',' && nest == 0 {
                let fin = part.clone();
                part = String::new();
                vec![fin]
            } else {
                part.push(c);
                vec![]
            }
        }) {
        let t = token.trim().to_string();

        if t == "" {
            return Err(EncodingError {
                msg: String::from("One or more list iotas have too many commas.")
            });

        } else if let Some(p) = get_pat_bin_optional(&t) {
            binary.push(pad_0_upto(p, chunk_size));
            
        } else if IOTA_REGEX.is_match(&t) {
            binary.append(&mut iota_to_binary(&t, chunk_size)?)

        } else if t.starts_with("Bookkeeper's Gambit: ") {
            binary.append(&mut make_bk_op(&t, chunk_size)?);

        } else if t.starts_with("Numerical Reflection: ") {
            binary.append(&mut make_numref(&t, chunk_size)?);

        } else {
            return Err(EncodingError {
                msg: format!("Invalid list iota element: \"{t}\"")
            });

        }
    }

    binary.push(pad_0_upto(0, chunk_size));
    binary.push(END_OPCODE.to_string());
    Ok(binary)
}

fn pad_0_upto(
    num: usize,
    chunk_size: u32
) -> String {
    let mut bin = format!("{:b}", num);
    let bits = bin.len();
    if bits > chunk_size as usize {
        // intentional, so that local mappings can deal with this later
        return bin;
    }

    for _ in 0..(chunk_size - bits as u32) { bin.insert(0, '0'); }
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

fn make_bk_op(
    token: &String,
    chunk_size: u32
) -> Result<Vec<String>, EncodingError> {
    if !BK_REGEX.is_match(&token) {
        return Err(EncodingError {
            msg: format!("Invalid Bookkeeper's Gambit: \"{token}\"")
        });
    }
    let mut bookkeeper: Vec<char> = token[21..].chars().collect();
    let mut len: usize = bookkeeper.len();

    let bookkeeper_opcode;
    if len < 4 {
        len = 4;
        bookkeeper_opcode = BK_OPCODE_4BIT;
    } else if len < 16 {
        len = 16;
        bookkeeper_opcode = BK_OPCODE_16BIT;
    } else {
        // Matt, is that you?
        return Err(EncodingError { msg:
            format!("Exceeded maximum Bookkeeper's Gambit length (maximum = 16, found size = {len}).")
        });
    }

    while bookkeeper.len() < len { bookkeeper.insert(0, '-'); }
    Ok(vec![
        pad_0_upto(0, chunk_size),
        bookkeeper_opcode.into(),
        bookkeeper
            .iter()
            .map(|s| {
                match s {
                    'v' => "1",
                    '-' => "0",
                    _ => panic!("Catastrophic failure. How did we get here?") // the regex has verified it for me but still
                }
            })
            .collect()
    ])
}

fn make_numref(
    token: &String,
    chunk_size: u32
) -> Result<Vec<String>, EncodingError> {
    let mut binary: Vec<String> = vec![];
    if !NUMREF_REGEX.is_match(&token) {
        return Err(EncodingError {
            msg: format!("Invalid Numerical Reflection: \"{token}\"")
        });
    }
    binary.push(pad_0_upto(get_pat_bin("Introspection"), chunk_size));
    binary.append(&mut embed_num(token[22..].parse().unwrap(), chunk_size));
    binary.push(pad_0_upto(get_pat_bin("Retrospection"), chunk_size));
    binary.push(pad_0_upto(get_pat_bin("Flock's Disintegration"), chunk_size));
    Ok(binary)
}

fn embed_num(
    num: f64,
    chunk_size: u32
) -> Vec<String> {
    let opcode;
    let num_bin;
    if is_u8(num) {
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

fn is_u8(
    num: f64
) -> bool {
    (num % 1.0).abs() <= f64::MIN_POSITIVE && num <= 255.0
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
        pad_0_upto(0, chunk_size),
        EMBED_VEC_OPCODE.into()
    ];
    for refnum in [desired.0, desired.1, desired.2].iter() {
        let num = *refnum;
        if is_u8(num) {
            ret.push(EMBED_VEC_I8_OPCODE.into());
            ret.push(format!("{:08b}", num as i8));
        } else if is_f32(num) {
            ret.push(EMBED_VEC_F32_OPCODE.into());
            ret.push(format!("{:32b}", (num as f32).to_bits()));
        } else {
            ret.push(EMBED_VEC_DOUBLE_OPCODE.into());
            ret.push(format!("{:64b}", num.to_bits()));
        }
    }

    return ret;
}

fn is_embedded_iota(
    pattern: &String
) -> bool {
    EMBEDDED_IOTA_REGEX.is_match(pattern)
}
