mod encoder_utils;
mod file_utils;
mod used_types;
mod patterns;

use std::env;
use std::fs;
use std::io;
use std::io::Read;

use crate::encoder_utils::{encode_line, end_op};
use crate::file_utils::{get_file, translate_to_dance, translate_to_octal};
use crate::patterns::{init_patterns, patterns_bits};

const OUTPUT_FORMAT_OPTIONS: [&str; 3] = ["bin", "octal", "dance"];

/// encodes a file into an 8-bit instruction set and returns the binary of each opcode.
fn encode(
    path: &String,
    chunk_size: u32
) -> Option<Vec<String>> {
    let file: Vec<String> = get_file(path);

    let binary_opt: Option<Vec<String>> = file
        .iter()
        .enumerate()
        .map(|(i, s)| encode_line(s, i, chunk_size))
        .collect::<Option<Vec<Vec<String>>>>() // horrid type
        .map(|o| o.into_iter().flatten().collect());

    if binary_opt.is_none() { return Option::None; }
    let mut binary = binary_opt.unwrap();

    binary.append(&mut end_op(chunk_size));
    return Option::Some(binary);
}

/// returns the input file and the output file from the command line.
fn get_arguments(
    args: &Vec<String>
) -> [String; 3] {

    if args.contains(&String::from("-h")) {
        print_usage();
        panic!("Printing help message.");
    }

    let i_option = args.iter().position(|s| s == "-i");
    let o_option = args.iter().position(|s| s == "-o");
    let f_option = args.iter().position(|s| s == "-f");
    let i;
    let o;
    let f;

    if i_option.is_none() || f_option.is_none() {
        print_usage();
        panic!("Missing mandatory options.");
    }

    i = i_option.unwrap();
    o = o_option.unwrap_or(0);
    f = f_option.unwrap();

    if o_option.is_some() {
        if i.abs_diff(o) == 1
            || o.abs_diff(f) == 1
            || (o + 1) >= args.len()
        {
            print_usage();
            panic!("Options must be followed by their arguments, not other options.");
        }
    }
    if i.abs_diff(f) == 1
        || (i + 1) >= args.len()
        || (f + 1) >= args.len()
    {
        print_usage();
        panic!("Options must be followed by their arguments, not other options.");
    }

    return [
        args[i + 1].clone(),
        if o_option.is_some() { args[o + 1].clone() } else { String::new().clone() },
        args[f + 1].clone()
    ];
}

fn print_usage() {
    // ANSI codes, my old friend!
    println!("\x1b[33mUsage example: -i \"path/to/my/file.hexpattern\" -o \"path/to/my/output.txt\" -f octal\x1b[0m");
    println!("Options:");
    println!("\x1b[1m-h\x1b[0m");
    println!("    \x1b[1mOPTIONAL.\x1b[0m");
    println!("    If present, stops everything and gives you this message.");
    println!("\x1b[31m-i <input file path>\x1b[0m");
    println!("    \x1b[1mMANDATORY.\x1b[0m");
    println!("    <input file path>: path to the file to be encoded.");
    println!("\x1b[32m-o <output file path>\x1b[0m");
    println!("    \x1b[1mOPTIONAL.\x1b[0m");
    println!("    <output file path>: path to the file to write the encoded output to.");
    println!("\x1b[34m-f <output format>\x1b[0m");
    println!("    \x1b[1mMANDATORY.\x1b[0m");
    println!("    <output format>: The format to write the encoded output in.");
    println!("            \x1b[1mbin\x1b[0m: outputs a string of 1s and 0s.");
    println!("            \x1b[1moctal\x1b[0m: outputs a string of octal digits (for use with the autohotkey script).");
    println!("            \x1b[1mdance\x1b[0m: outputs lines of dance moves (for use with the dance decoder).");
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

fn main() {
    enable_ansi();
    init_patterns();

    let args: Vec<String> = env::args().collect();

    let mut input = String::new();
    let mut output = String::new();
    let mut format = String::new();

    if args.len() > 1 {
        let io = get_arguments(&args);
        input = io[0].clone();
        output = io[1].clone();
        format = io[2].clone();
    } else {
        println!("Path to input file >");
        io::stdin().read_line(&mut input).expect("Could not read input path.");
        println!("Path to output file (you may leave this empty) >");
        io::stdin().read_line(&mut output).expect("Could not read output path.");
        println!("Output format (\"bin\", \"octal\", \"dance\") >");
        io::stdin().read_line(&mut format).expect("Could not read output format.");

        // mfw \r\n
        input.truncate(input.len() - 2);
        output.truncate(output.len() - 2);
        format.truncate(format.len() - 2);
    }

    if !fs::exists(&input).unwrap_or(false) {
        panic!("Input path, \"{input}\", is inaccessible.");

    } else if !OUTPUT_FORMAT_OPTIONS.contains(&format.as_str()) {
        panic!("Cannot output to format \"{format}\". Must be one of: \"bin\", \"octal\", \"dance\".");
    }

    let mut encoded: Vec<String> = vec![];
    let mut write: String = String::new();
    // https://cdn.discordapp.com/attachments/1467549530894635171/1484382444760203324/image.png?ex=69be0661&is=69bcb4e1&hm=7f501e5bbe4e464a5795363e317dfe6130896347589fafe65cec4048c0cbb8d2&
    for chunk_size in (1..patterns_bits() as u32 + 1).rev() {
        let encoded_opt = encode(&input, chunk_size);
        if encoded_opt.is_some() {
            encoded = encoded_opt.unwrap();
        }
    }

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
    if output == "" {
        println!("Press Enter key to continue...");
        io::stdin().bytes().next();
        /*let mut _input = String::new();
        io::stdin().read_line(&mut _input).unwrap();*/
        return;
    }

    let res = fs::write(output, write);
    if res.is_err() {
        println!("Error writing to output: {}", res.unwrap_err());
    } else {
        println!("Successfully written to output!");
    }

    println!("Press Enter key to continue...");
    /*let mut _input = String::new();
    io::stdin().read_line(&mut _input).unwrap();*/
    io::stdin().bytes().next();
}
