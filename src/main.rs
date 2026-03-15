mod encoder_utils;
mod file_utils;
mod used_types;
mod patterns;

use std::env;
use std::fs;
use std::io;

use crate::encoder_utils::{encode_line, end_op};
use crate::file_utils::{get_file, translate_to_dance, translate_to_octal};
use crate::patterns::init_patterns;

/// encodes a file into an 8-bit instruction set and returns the binary of each opcode.
fn encode(
    path: &String
) -> Vec<String> {
    let file: Vec<String> = get_file(path);
    let mut line: usize = 0;
    let mut binary: Vec<String> = file
        .iter()
        .flat_map(|s| {
            line += 1;
            return encode_line(s, line);
        })
        .collect();

    binary.append(&mut end_op());
    return binary;
}

/// returns the input file and the output file from the command line.
fn get_arguments(
    args: &Vec<String>
) -> [String; 3] {

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
    println!("\x1b[31m-i <input file path>\x1b[0m");
    println!("    \x1b[1mMANDATORY.\x1b[0m");
    println!("    <input file path>: path to the file to be encoded.");
    println!("\x1b[32m-o <output file path>\x1b[0m");
    println!("    \x1b[1mOPTIONAL.\x1b[0m");
    println!("    <output file path>: path to the file to write the encoded output to.");
    println!("\x1b[34m-f <output format>\x1b[0m");
    println!("    \x1b[1mMANDATORY.\x1b[0m");
    println!("    <output format>: The format to write the encoded output in.");
    println!("    \x1b[1mCHOICES\x1b[0m");
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

// mac is sensible right
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
    } else if format != "bin" && format != "octal" && format != "dance" {
        panic!("Cannot output to format \"{format}\". Must be one of: \"bin\", \"octal\", \"dance\".");
    }

    let encoded = encode(&input);
    let mut write: String = String::new();

    if format == "bin" {
        for line in &encoded {
            println!("{line}");
        }

        write = encoded.join("\n");

    } else if format == "octal" {
        let display = translate_to_octal(&encoded);

        for octal in &display {
            print!("{octal}");
        }
        println!();

        write = display
            .iter()
            .map(|n| n.to_string())
            .collect();

    } else if format == "dance" {
        let display = translate_to_dance(&encoded);

        for line in display {
            println!("{}", &line);
            write.push_str(
                line
                    .strip_prefix("\x1b[4m")
                    .unwrap()
                    .strip_suffix("\x1b[0m")
                    .unwrap()
            );
            write.push_str(
                "\n-----------------------------------------\n"
            );
        }

    }

    if output == "" {
        println!("\nNo output path provided. Shutting down...");
        return;
    }

    let res = fs::write(output, write);
    if res.is_err() {
        println!("\nError writing to output: {}", res.unwrap_err());
    } else {
        println!("\nSuccessfully written to output!");
    }
}
