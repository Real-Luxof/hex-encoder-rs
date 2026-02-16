use std::{fs, vec};

use crate::used_types::Chunked;

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

/*pub fn translate_to_dance(
    contents: &Vec<String>
) -> Vec<String> {
    
}*/
