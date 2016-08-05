use std::io;
use std::env;
use std::io::prelude::*;

enum TrMode {
    ReplaceWith(Vec<char>),
    Delete,
}

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::new();
    let mut result = String::new();
    let mut operation_mode = TrMode::ReplaceWith(Vec::new()); //default mode
    let mut squeeze_repeats = false;
    let mut chars_to_squeeze: Vec<char> = Vec::new();
    let mut complement_set = false;
    let mut truncate_set = false;
    let mut first_argument_requires_escaping_dash = true; //for GNU-compatible option parsing

    // parsing of command-line arguments
    if env::args().count() > 2 {
        let mut first_non_option_argument = 0;
        for (arg_number, arg) in env::args().skip(1).enumerate() {
            match arg.to_string().as_str() {
                "-c" | "-C" | "--complement" => complement_set = true,
                "-d" | "--delete" => operation_mode = TrMode::Delete,
                "-s" | "--squeeze-repeats" => squeeze_repeats = true,
                "-t" | "--truncate-set1" => truncate_set = true,
                "--" => {first_argument_requires_escaping_dash = false;
                         first_non_option_argument = arg_number + 1; break},
                _ => if arg.to_string().chars().nth(0).unwrap() == '-' {panic!("Unknown argument: {}", arg)}
                     else {first_non_option_argument = arg_number; break},
            }
        }
        let chars_to_replace: Vec<char> = env::args().skip(1).nth(first_non_option_argument).unwrap().chars().collect();
        match operation_mode {
            // if we're replacing, read the chars to replace with from command line
            TrMode::ReplaceWith(_) => operation_mode = TrMode::ReplaceWith(env::args().skip(1).nth(first_non_option_argument + 1).unwrap().chars().collect()),
            _ => {},
        };

        // main loop
        while stdin.read_line(&mut buffer).unwrap() > 0 {
            for character in buffer.chars() {
                if chars_to_replace.contains(&character) ^ complement_set {
                    match operation_mode {
                         //TODO: handle this as set instead of inserting single char
                        TrMode::ReplaceWith(ref chars_to_insert) => result.push(chars_to_insert[0]),
                        TrMode::Delete => {}, //do not copy char to output buffer
                    }
                } else {
                    result.push(character)
                };
            };
            print!("{}", result);
            // output is line-buffered, but we do not always print \n,
            // so without the following line we may never output anything
            io::stdout().flush();
            buffer.clear();
            result.clear();
        }
    } else {
        println!("Usage: tr [OPTION]... SET1 [SET2]");
    }
}
