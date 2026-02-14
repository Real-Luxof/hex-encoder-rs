mod encoder_utils;
mod file_utils;

use std::env;
use std::fs;
use std::io;

use crate::encoder_utils::Chunked;
use crate::encoder_utils::encode_line;
use crate::encoder_utils::end_op;
use crate::file_utils::get_file;

// with permission from petra i scraped the forums
// i found the most popular patterns
// fun fact: somehow, Introspection is used 2067 times but Retrospection is used only 1757 times
// problem in my scraper? i dunno. One must imagine Luxof lazy.
const POPULAR_PATTERNS: [&str; 175] = ["Introspection", "Retrospection", "Gemini Decomposition", "Jester's Gambit", "Rotation Gambit", "Mind's Reflection", "Additive Distillation", "Hermes' Gambit", "Flock's Disintegration", "Augur's Exaltation", "Muninn's Reflection", "Compass' Purification", "Multiplicative Distillation", "Huginn's Gambit", "Flock's Gambit", "Subtractive Distillation", "Selection Distillation", "Thoth's Gambit", "Fisherman's Gambit", "Division Distillation", "Prospector's Gambit", "Dioscuri Gambit", "Alidade's Purification", "Undertaker's Gambit", "Consideration", "Gemini Gambit", "Equality Distillation", "Surgeon's Exaltation", "Rotation Gambit II", "Speaker's Decomposition", "Length Purification", "Vector Exaltation", "Vector Disintegration", "Augur's Purification", "Flock's Reflection", "Archer's Distillation", "Scribe's Reflection", "Retrograde Purification", "Nullary Reflection", "Maximus Distillation", "Integration Distillation", "Break Block", "Vector Reflection +Y", "Vacant Reflection", "Minimus Distillation", "Fisherman's Gambit II", "Modulus Distillation", "Swindler's Gambit", "Selection Exaltation", "Power Distillation", "Reveal", "Speaker's Distillation", "Floor Purification", "Charon's Gambit", "Vector Reflection -Y", "Explosion", "Summon Greater Sentinel", "Architect's Distillation", "Scribe's Gambit", "Inequality Distillation", "Impulse", "Single's Purification", "Locator's Distillation", "Pace Purification", "Locate Sentinel", "Stadiometer's Purification", "Zone Distillation: Item", "Ceiling Purification", "Create Water", "Vector Reflection Zero", "Chronicler's Purification", "Entity Purification", "Flay Mind", "Axial Purification", "Akasha's Distillation", "Arc's Reflection", "Banish Sentinel", "Compass' Purification II", "Place Block", "Summon Lightning", "Vector Reflection +Z", "Vector Reflection -X", "Negation Purification", "Scout's Distillation", "Vector Reflection -Z", "Waystone Reflection", "Chronicler's Gambit", "Erase Item", "Exclusion Distillation", "Greater Teleport", "Tangent Purification", "White Sun's Zenith", "Conjunction Distillation", "Cosine Purification", "Craft Artifact", "Disjunction Distillation", "Entity Purification: Living", "False Reflection", "Iris' Gambit", "Lesser Fold Reflection", "Uniqueness Purification", "Zone Distillation: Living", "Zone Distillation: Non-Living", "Zone Distillation: Non-Player", "Blink", "Conjure Light", "Zone Distillation: Monster", "Akasha's Gambit", "Altiora", "Assessor's Reflection", "Auditor's Purification", "Auditor's Reflection", "Blue Sun's Nadir", "Circle's Reflection", "Conjure Block", "Derivation Decomposition", "Entity Purification: Item", "Entropy Reflection", "Euler's Reflection", "Excisor's Distillation", "Greater Fold Reflection", "Gulliver's Purification", "Inverse Tangent Purification", "Lodestone Reflection", "Make Note", "Recharge Item", "Sine Purification", "Thanatos' Reflection", "True Reflection", "Vector Reflection +X", "Wayfind Sentinel", "White Sun's Nadir", "Zone Distillation: Non-Animal", "Zone Distillation: Non-Item", "Zone Distillation: Non-Monster", "Zone Distillation: Player", "Alter Scale", "Anchorite's Flight", "Assessor's Purification", "Aviator's Purification", "Black Sun's Nadir", "Black Sun's Zenith", "Blue Sun's Zenith", "Caster's Glamour", "Craft Cypher", "Craft Phial", "Craft Trinket", "Create Lava", "Destroy Liquid", "Dispel Rain", "Edify Sapling", "Entity Purification: Animal", "Entity Purification: Monster", "Entity Purification: Player", "Evanition", "Extinguish Area", "Fireball", "Green Sun's Nadir", "Green Sun's Zenith", "Ignite", "Internalize Pigment", "Inverse Cosine Purification", "Inverse Sine Purification", "Inverse Tangent Distillation", "Logarithmic Distillation", "Maximus Distillation II", "Minimus Distillation II", "Overgrow", "Red Sun's Nadir", "Red Sun's Zenith", "Summon Rain", "Summon Sentinel", "Wayfarer's Flight", "Zone Distillation: Animal", "Zone Distillation: Any"];

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
            println!("Paths \"{path}\" does not exist. Tests have ended.");
            break;
        }

        let _ = encode(&path);
        println!("Test \"{path}\" successful.");
    }
}

fn bin_to_double(str: String) -> f64 {
    let mut chars = str.chars();

    let neg = if chars.next().unwrap() == '1' { -1.0 } else { 1.0 };
    let exponent_chars = chars.next_chunk_of(11).unwrap();
    let mantissa_chars = chars.next_chunk_of(52).unwrap();

    let mut exponent: f64 = 0.0;
    let mut n: i32 = 10;
    for bit in exponent_chars {
        if bit == '1' {
            exponent += 2_f64.powi(n);
        }
        n -= 1;
    }
    let mut mantissa: f64 = 1.0;
    for bit in mantissa_chars {
        if bit == '1' {
            mantissa += 2_f64.powi(n);
        }
        n -= 1;
    }
    return neg * mantissa * 2_f64.powi(exponent as i32 - 1023);
}   

fn main() {
    let args: Vec<String> = env::args().collect();

    /*//let test: f64 = -2.5;
    //let test: f64 = 3.0;
    //let test: f64 = -3.5;
    let test: f64 = 2.0;
    let test_str = format!("{:064b}", test.to_bits());
    //let bits = u64::from_str_radix(&test_str, 2).unwrap();
    //let test2 = f64::from_bits(bits);
    dbg!(test);
    dbg!(&test_str);
    let test2= bin_to_double(test_str);
    dbg!(test2);*/

    let mut path = String::new();
    if args.len() > 1 {
        path += &args[1];
    } else {
        println!("Path to file >");
        io::stdin().read_line(&mut path).expect("Could not read input.");
        path.truncate(path.len() - 2); // mfw \r\n
    }

    if args.contains(&String::from("-t")) {
        run_tests(path);
    } else {
        //dbg!(encode(&path));
    }
}
