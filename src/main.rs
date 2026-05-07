mod encoder_utils;
mod file_utils;
mod used_types;
mod patterns;

use device_query::DeviceState;
use device_query::Keycode;
use enigo::Button;
use enigo::Direction;
use enigo::Enigo;
use enigo::Key;
use enigo::Keyboard;
use enigo::Mouse;
use enigo::Settings;
use std::env;
use std::fs;

use crate::encoder_utils::find_unique_patterns;
use crate::encoder_utils::preprocess_file;
use crate::encoder_utils::tokens_to_binary;
use crate::encoder_utils::{tokenize_file};
use crate::file_utils::{get_file, translate_to_dance, translate_to_octal};
use crate::patterns::{patterns_bits};
use crate::used_types::EncodingError;
use crate::used_types::Promptable;

const OUTPUT_FORMAT_OPTIONS: [&str; 3] = ["bin", "octal", "dance"];

fn get_tokens(
    path: &String
) -> Result<Vec<String>, EncodingError> {
    tokenize_file(preprocess_file(get_file(path))?)
}

/// encodes a file into an 8-bit instruction set and returns the binary of each opcode.
fn encode(
    path: &String
) -> Vec<String> {
    let tokens: Vec<String> = match get_tokens(path) {
        Ok(val) => val,
        Err(e) => panic!("{e}")
    };
    let unique_patterns = find_unique_patterns(&tokens);

    let mut strongest_contender: Vec<String> = vec![];
    let mut strongest_contender_points = usize::MAX;

    // https://cdn.discordapp.com/attachments/1467549530894635171/1484382444760203324/image.png?ex=69be0661&is=69bcb4e1&hm=7f501e5bbe4e464a5795363e317dfe6130896347589fafe65cec4048c0cbb8d2&
    for i in (unique_patterns.len() as f64).log2().floor() as u32..patterns_bits() {
        match tokens_to_binary(
            &tokens,
            &unique_patterns,
            i
        ) {
            Ok(opt) => {
                match opt {
                    Some(binary_and_lm) => {
                        let contender = binary_and_lm.0;
                        let points = contender.join("").replace(" ", "").len();

                        if points < strongest_contender_points {
                            strongest_contender = contender;
                            strongest_contender_points = points;
                        }
                    },
                    None => continue
                };
            },
            Err(err) => panic!("{err}")
        };
    }

    return strongest_contender;
}

/// returns the input file and the output file from the command line.
fn get_arguments(
    args: Vec<String>,
    fulfill: Vec<&str>
) -> Vec<Option<String>> {

    if args.contains(&String::from("-h")) {
        print_usage();
        panic!("Printing help message.");
    }

    let mut given: Vec<Option<String>> = vec![];
    let mut fulfilled: Vec<&str> = vec![];
    for arg in &fulfill {
        given.push(
            args.iter().position(|s| s == arg)
                .and_then(|i| args.get(i + 1))
                .and_then(|s| {
                    if (&fulfill).contains(&s.as_str()) {
                        panic!("Options must be followed by their arguments, not other options.");
                    }
                    Some(s)
                } )
                .and_then(|s| { fulfilled.push(arg); Some(s.clone()) })
        );
    }
    given
}

fn print_usage() {
    // ANSI codes, my old friend!
    println!("\x1b[33mUsage example: -i \"path/to/my/file.hexpattern\" -o \"path/to/my/output.txt\" -f octal\x1b[0m");
    println!("Options:");
    println!("\x1b[1m-h\x1b[0m");
    println!("    If present, stops everything and gives you this message.");
    println!("\x1b[31m-i <input file path>\x1b[0m");
    println!("    <input file path>: path to the file to be encoded.");
    println!("\x1b[32m-o <output file path>\x1b[0m");
    println!("    <output file path>: path to the file to write the encoded output to.");
    println!("\x1b[34m-f <output format>\x1b[0m");
    println!("    <output format>: The format to write the encoded output in.");
    println!("            \x1b[1mbin\x1b[0m: outputs a string of 1s and 0s.");
    println!("            \x1b[1moctal\x1b[0m: outputs a string of octal digits (for use with the macro).");
    println!("            \x1b[1mdance\x1b[0m: outputs lines of dance moves (for use with the dance decoder).");
    println!("    To disable the prompt on unfulfilled argument, you can provide an empty string (\"\") to this option.");
    println!("\x1b[35m-p <Y/n>\x1b[0m");
    println!("    <Y/n>: Y or N. Case-insensitive. Yes or no to pasting in the octals for your decoder when you next press F6.");
    println!("    Can only be used if output format is octal.");
}

#[cfg(windows)]
fn enable_ansi() {
    match enable_ansi_support::enable_ansi_support() {
        Ok(()) => {}
        Err(_) => {
            println!("Could not enable ANSI codes on your terminal. Colors on errs may not exist.");
        }
    }
}

// mac is sensible and has colorful terminals right
#[cfg(not(windows))]
fn enable_ansi() {}

fn wait_for_keys(
    device_state: &DeviceState,
    keys: Vec<Keycode>
) -> Keycode {
    loop {
        for pressed_key in device_state.query_keymap() {
            if keys.contains(&pressed_key) {
                return pressed_key;
            }
        }
    }
}

fn main() {
    enable_ansi();

    let args: Vec<String> = env::args().collect();

    let inputs = get_arguments(args, vec!["-i", "-o", "-f", "-p"]);
    let input = inputs[0].or_else_ask("Path to input file >");
    let output = inputs[1].or_else_ask("Path to output file (may be left empty) >");
    let format = {
        let f = inputs[2].or_else_ask(
            "Output format (\"bin\", \"octal\", or \"dance\") >"
        );
        if f != "bin" && f != "octal" && f != "dance" {
            panic!("Format must be \"bin\", \"octal\", or \"dance\".");
        }
        f
    };
    let paste: bool = if format == "octal" {
        let p = inputs[3].or_else_ask(
            "Would you like the program to paste the output for your decoder (with right clicks!) when you next press F6? (Y/n) >"
        ).to_ascii_lowercase();
        if p == "y" {
            true
        } else if p == "n" {
            false
        } else {
            panic!("The input to this option must be Y or N (case-insensitive).");
        }
    } else { false };

    if !fs::exists(&input).unwrap_or(false) {
        panic!("Input path, \"{input}\", is inaccessible.");

    } else if !OUTPUT_FORMAT_OPTIONS.contains(&format.as_str()) {
        panic!("Cannot output to format \"{format}\". Must be one of: \"bin\", \"octal\", \"dance\".");
    }

    let encoded: Vec<String> = encode(&input);
    let mut write: String = String::new();

    if format == "bin" {
        encoded.iter().for_each(|s| println!("{s}"));
        write = encoded.join("\n");

    } else if format == "octal" {
        let display = translate_to_octal(&encoded);

        display.iter().for_each(|s| print!("{s}"));
        println!();

        write = display
            .iter()
            .map(|n| n.to_string())
            .collect();

    } else if format == "dance" {
        let display = translate_to_dance(&encoded);

        for line in display {
            write.push_str(&line);
            println!("\x1b[4m{line}\x1b[0m");
            write.push_str("\n-----------------------------------------\n");
        }

    }
    println!();

    let device_state = DeviceState::new();
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    if paste {
        println!("Waiting for you to press F6...");
        wait_for_keys(&device_state, vec![Keycode::F6]);

        for key in write.chars() {
            enigo.key(
                Key::from(
                    match key {
                        '0' => Key::Num1,
                        '1' => Key::Num2,
                        '2' => Key::Num3,
                        '3' => Key::Num4,
                        '4' => Key::Num5,
                        '5' => Key::Num6,
                        '6' => Key::Num7,
                        '7' => Key::Num8,
                        _ => panic!("You're not supposed to be here.")
                    }
                ),
                Direction::Click
            ).unwrap();
            enigo.button(Button::Right, Direction::Click).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(150));
        }
    }

    if output == "" {
        println!("Press Enter key to continue...");
        wait_for_keys(&device_state, vec![Keycode::Enter]);
        return;
    }

    let res = fs::write(output, write);
    if res.is_err() {
        println!("Error writing to output: {}", res.unwrap_err());
    } else {
        println!("Successfully written to output!");
    }

    println!("Press Enter key to continue...");
    wait_for_keys(&device_state, vec![Keycode::Enter]);
}
