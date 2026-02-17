use std::{fs, vec};

use crate::used_types::{Chunked, Pair};

fn get_file_lines(
    path: &String
) -> Vec<String> {
    let file = fs::read_to_string(path).expect(
        format!("Could not read file with path \"{path}\"").as_str()
    );

    let mut buffer: Vec<String> = vec![String::new()];
    let mut prev_ch = ' ';
    let mut is_comment = false;
    let mut is_multiline_comment = false;

    // GOD I LOVE STRING PROCESSING!!!!!!
    for ch in file.chars() {
        if ch == '\n' {
            buffer.push(String::new());
            is_comment = false;

        } else if prev_ch == '/' && ch == '/' {
            buffer.last_mut().unwrap().pop();
            is_comment = true;
        
        } else if prev_ch == '/' && ch == '*' && !is_comment {
            buffer.last_mut().unwrap().pop();
            is_multiline_comment = true;

        } else if prev_ch == '*' && ch == '/' && is_multiline_comment {
            is_multiline_comment = false;

        } else if ch == ' ' && buffer.last_mut().unwrap() == "Consideration:" {
            buffer.push(String::new());

        } else if !(is_comment || is_multiline_comment) {
            buffer.last_mut().unwrap().push(ch);
        }

        prev_ch = ch;
    }

    return buffer;
}

fn remove_lines_whitespace(
    lines: Vec<String>
) -> Vec<String> {
    return lines
        .iter()
        .map(|s| s.trim_ascii().to_string())
        .collect::<Vec<String>>();
}

fn flatmap_necessaries(
    patterns: Vec<String>
) -> Vec<String> {
    return patterns
        .iter()
        .flat_map(
            |s| match s.as_str() {
                "{}" => vec!["Introspection", "Retrospection"],
                "{" => vec!["Introspection"],
                "}" => vec!["Retrospection"],
                "Consideration:" => vec!["Consideration"],
                _ => vec![s.as_str()]
            }
        ).map(str::to_string)
        .collect();
}

pub fn get_file(
    path: &String
) -> Vec<String> {
    return flatmap_necessaries(remove_lines_whitespace(get_file_lines(path)));
}

pub fn translate_to_bin(
    contents: &Vec<String>
) -> Vec<Vec<bool>> {
    return contents
        .iter()
        .map(|s| s
            .chars()
            .map(|c| c == '1')
            .collect()
        )
        .collect();
}

pub fn translate_to_octal(
    contents: &Vec<String>
) -> Vec<u8> {
    return contents
        .iter()
        .flat_map(|s| {
            let mut ret: Vec<String> = s
                .chars()
                .chunks_of(3)
                .map(|chars| chars
                    .iter()
                    .collect::<String>()
                ).collect();

            let the_rest = s[s.len() - s.len() % 3..s.len()].to_string();
            if the_rest != "" {
                ret.push(the_rest);
            }

            ret
        })
        .map(|s| {
            u8::from_str_radix(s.as_str(), 2).unwrap()
        })
        .collect();
}


pub fn translate_to_dance(
    contents: &Vec<String>
) -> Vec<String> {
    let mut binary = contents
        .iter()
        .flat_map(|s| s
            .chars()
            .collect::<Vec<char>>()
        )
        .collect::<Vec<char>>();

    let mut encoded: Vec<String> = vec![];
    //encoded.push(String::from("| LOOK DIR  | MOVE DIR  |  ACT  |  ACT  |"));
    //encoded.push(String::from("|-----------|-----------|-------|-------|"));
    //encoded.push(String::from("-----------------------------------------"));
    encoded.push(String::from("\x1b[4m| LOOK DIR  | MOVE DIR  |  ACT  |  ACT  |\x1b[0m"));

    let mut len: usize = binary.len();
    while len > 0 {

        let mut withdrawn: String = String::new();

        for i in 0..10.min(len) {
            withdrawn.push(binary[i]);
        }
        withdrawn = pad(&withdrawn, '0', 10);

        let mut used_size: usize = 0;
        let lookvec_result = encode_lookvec_to_dir(&withdrawn);
        withdrawn = withdrawn.split_off(lookvec_result.left);
        used_size += lookvec_result.left;

        let movement_result = encode_movement_to_dir(&withdrawn);
        withdrawn = withdrawn.split_off(movement_result.left);
        used_size += movement_result.left;

        // what the hell..
        let jump = &withdrawn[0..1] == "1";
        let sneak = &withdrawn[1..2] == "1";
        used_size += 2;

        encoded.push(
            String::from("\x1b[4m")
            + &format_move(
                (lookvec_result.right, movement_result.right, jump, sneak)
            )
            + &String::from("\x1b[0m")
        );

        len -= used_size.min(len);
        binary = binary.split_off(used_size.min(len));
    }

    return encoded;
}

fn pad(
    to_pad: &String,
    with: char,
    up_to: usize
) -> String {
    let mut padded = to_pad.chars().collect::<Vec<char>>();
    while padded.len() < up_to {
        padded.push(with);
    }
    return padded.iter().collect::<String>();
}

fn encode_lookvec_to_dir(
    bits: &String
) -> Pair<usize, String> {
    match &bits[0..4] {
        // hey kid, you ever wanted to see someone use 3.4 or 3.222... bits?
        "0000" => Pair { left: 4_usize, right: "Up" },
        "0001" => Pair { left: 4_usize, right: "Down" },
        _ => Pair {
            left: 3_usize,
            right: encode_to_horizontal_dir(&bits[0..3]).unwrap_or_else(
                || panic!("Give me some 1s and 0s, dumbass.")
            )
        }
    }.convert()
}

fn encode_movement_to_dir(
    bits: &String
) -> Pair<usize, String> {
    match &bits[0..4] {
        "0000" => Pair { left: 4_usize, right: "" },
        _ => Pair {
            left: 3_usize,
            right: encode_to_horizontal_dir(&bits[0..3]).unwrap_or_else(
                || panic!("Give me some 1s and 0s, dumbass.")
            )
        }
    }.convert()
}

fn encode_to_horizontal_dir(
    three_bits: &str
) -> Option<&str> {
    match three_bits {
        "000" => Some("North"),
        "001" => Some("West"),
        "010" => Some("South"),
        "011" => Some("East"),
        "100" => Some("Northwest"),
        "101" => Some("Northeast"),
        "110" => Some("Southwest"),
        "111" => Some("Southeast"),
        _ => None
    }
}

fn format_move(
    components: (String, String, bool, bool)
) -> String {
    format!(
        //"| {} | {} | {} | {} |\n-----------------------------------------",
        "| {} | {} | {} | {} |",
        center(components.0, 9),
        center(components.1, 9),
        center(if components.2 { "JUMP" } else { "" }.into(), 5),
        center(if components.3 { "SNEAK" } else { "" }.into(), 5)
    )
}

fn center(
    what: String,
    in_size: usize
) -> String {
    let mut centered = what.chars().collect::<Vec<char>>();
    let mut other = false;
    while centered.len() < in_size {
        centered.insert(if other { 0 } else { centered.len() }, ' ');
        other = !other;
    }
    return centered.iter().collect::<String>();
}
