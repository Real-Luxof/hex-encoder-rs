mod encoder_utils;
mod file_utils;
mod used_types;

use std::env;
use std::fs;
use std::io;

use crate::encoder_utils::encode_line;
use crate::encoder_utils::end_op;
use crate::file_utils::get_file;
use crate::file_utils::translate_to_bin;
use crate::file_utils::translate_to_octal;

// with permission from petra i scraped the forums
// i found the most popular patterns
// fun fact: somehow, Introspection is used 2067 times but Retrospection is used only 1757 times
// problem in my scraper? i dunno. One must imagine Luxof lazy.
const POPULAR_PATTERNS: [&str; 175] = ["Introspection", "Retrospection", "Jester's Gambit", "Gemini Decomposition", "Rotation Gambit", "Additive Distillation", "Flock's Disintegration", "Hermes' Gambit", "Muninn's Reflection", "Augur's Exaltation", "Flock's Gambit", "Subtractive Distillation", "Mind's Reflection", "Huginn's Gambit", "Multiplicative Distillation", "Compass' Purification", "Fisherman's Gambit", "Division Distillation", "Selection Distillation", "Prospector's Gambit", "Thoth's Gambit", "Dioscuri Gambit", "Gemini Gambit", "Alidade's Purification", "Equality Distillation", "Undertaker's Gambit", "Vector Disintegration", "Length Purification", "Flock's Reflection", "Vector Exaltation", "Retrograde Purification", "Speaker's Decomposition", "Nullary Reflection", "Archer's Distillation", "Maximus Distillation", "Surgeon's Exaltation", "Consideration", "Augur's Purification", "Minimus Distillation", "Integration Distillation", "Swindler's Gambit", "Selection Exaltation", "Vacant Reflection", "Break Block", "Charon's Gambit", "Scribe's Reflection", "Vector Reflection +Y", "Single's Purification", "Power Distillation", "Floor Purification", "Modulus Distillation", "Speaker's Distillation", "Reveal", "Architect's Distillation", "Locator's Distillation", "Conjunction Distillation", "Vector Reflection -Y", "Inequality Distillation", "Vector Reflection Zero", "Negation Purification", "Stadiometer's Purification", "Scribe's Gambit", "Locate Sentinel", "Impulse", "Vector Reflection +X", "Pace Purification", "Chronicler's Purification", "Cosine Purification", "Zone Distillation: Item", "Arc's Reflection", "Axial Purification", "Vector Reflection +Z", "Place Block", "False Reflection", "Summon Greater Sentinel", "Uniqueness Purification", "Greater Teleport", "Inverse Cosine Purification", "Vector Reflection -Z", "Tangent Purification", "Scout's Distillation", "Excisor's Distillation", "Entity Purification", "Zone Distillation: Non-Living", "Disjunction Distillation", "Vector Reflection -X", "Inverse Tangent Distillation", "Inverse Tangent Purification", "True Reflection", "Conjure Block", "Explosion", "Zone Distillation: Non-Item", "Akasha's Distillation", "Chronicler's Gambit", "Akasha's Gambit", "Circle's Reflection", "Conjure Light", "Thanatos' Reflection", "Zone Distillation: Non-Player", "Sine Purification", "Waystone Reflection", "Zone Distillation: Living", "Entity Purification: Living", "Create Water", "Zone Distillation: Non-Animal", "Zone Distillation: Non-Monster", "Entity Purification: Item", "Auditor's Reflection", "Ceiling Purification", "Exclusion Distillation", "Lesser Fold Reflection", "Blink", "Greater Fold Reflection", "Recharge Item", "Banish Sentinel", "Zone Distillation: Monster", "Flay Mind", "White Sun's Zenith", "Iris' Gambit", "Erase Item", "Euler's Reflection", "Derivation Decomposition", "Gulliver's Purification", "Make Note", "Blue Sun's Nadir", "Summon Lightning", "White Sun's Nadir", "Entropy Reflection", "Lodestone Reflection", "Assessor's Reflection", "Auditor's Purification", "Wayfind Sentinel", "Zone Distillation: Player", "Alter Scale", "Altiora", "Anchorite's Flight", "Assessor's Purification", "Aviator's Purification", "Black Sun's Nadir", "Black Sun's Zenith", "Blue Sun's Zenith", "Caster's Glamour", "Compass' Purification II", "Craft Artifact", "Craft Cypher", "Craft Phial", "Craft Trinket", "Create Lava", "Destroy Liquid", "Dispel Rain", "Edify Sapling", "Entity Purification: Animal", "Entity Purification: Monster", "Entity Purification: Player", "Evanition", "Extinguish Area", "Fireball", "Fisherman's Gambit II", "Green Sun's Nadir", "Green Sun's Zenith", "Ignite", "Internalize Pigment", "Inverse Sine Purification", "Logarithmic Distillation", "Maximus Distillation II", "Minimus Distillation II", "Overgrow", "Red Sun's Nadir", "Red Sun's Zenith", "Rotation Gambit II", "Summon Rain", "Summon Sentinel", "Wayfarer's Flight", "Zone Distillation: Animal", "Zone Distillation: Any"];

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

/// tests files in paths `{base_path}{0..untilFileNotFound}` to see where the encoder shits itself.
fn run_tests(
    base_path: String
) {
    let mut i: usize = 0;
    while fs::exists(format!("{base_path}{i}.hexpattern")).unwrap_or(false) {
        let path = format!("{base_path}{i}.hexpattern");
        i += 1;

        if !fs::exists(&path).unwrap_or(false) {
            println!("Path \"{path}\" is inaccessible. Tests have ended.");
            break;
        }

        let _ = encode(&path);
        println!("Test \"{path}\" successful.");
    }
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

fn main() {
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

    if args.contains(&String::from("-t")) {
        run_tests(input);
        println!("All tests successful!");
        return;
    }

    let encoded = encode(&input);

    if format == "bin" {
        let display = translate_to_bin(&encoded);
        for line in display {
            for bin in line {
                let ch: u8 = if bin { 1 } else { 0 };
                print!("{ch}");
            }
            println!();
        }

    } else if format == "octal" {
        let display = translate_to_octal(&encoded);
        for octal in display {
            print!("{octal}");
        }
        println!();

    }
}
