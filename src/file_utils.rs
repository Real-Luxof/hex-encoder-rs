use std::{fs, vec};

use crate::used_types::{Chunked, Pair};

pub fn get_file(
    path: &String
) -> String {
    fs::read_to_string(path).expect(
        format!("Could not read file with path \"{path}\"").as_str()
    )
}

pub fn translate_to_octal(
    contents: &Vec<String>
) -> Vec<u8> {
    return contents
        .iter()
        .flat_map(|s| {
            let mut ret: Vec<String> = s
                // local mappings have lotsa spaces to be readable in binary
                .replace(" ", "")
                .chars()
                .chunks_of(3)
                .map(|chars| chars.iter().collect::<String>())
                .collect();

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
            // local mappings have lotsa spaces to be readable in binary
            .replace(" ", "")
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

        encoded.push(format_move(
            (lookvec_result.right, movement_result.right, jump, sneak)
        ));

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
