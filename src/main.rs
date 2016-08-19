use std::io;
use std::env;
use std::io::prelude::*;

mod escape_parser;

#[derive(PartialEq)]
#[derive(Debug)]
enum TrMode {
    ReplaceWith(Vec<char>),
    Delete,
    //TODO: case conversion
}

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::new();
    let mut result = String::new();
    let mut squeezed_result = String::new();
    let mut operation_mode = TrMode::ReplaceWith(Vec::new()); //default mode
    let mut squeeze_repeats = false;
    let mut only_squeeze_repeats = false;
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
        let mut chars_to_replace: Vec<char> = escape_parser::parse(env::args().skip(1).nth(first_non_option_argument).unwrap());
        match operation_mode {
            // if we're replacing, read the chars to replace with from command line
            TrMode::ReplaceWith(_) => {
                if env::args().skip(1).nth(first_non_option_argument + 1).is_some() {
                    let chars_to_insert: Vec<char> = escape_parser::parse(env::args().skip(1).nth(first_non_option_argument + 1).unwrap());
                    if truncate_set {chars_to_replace.truncate(chars_to_insert.len())};
                    operation_mode = TrMode::ReplaceWith(chars_to_insert);
                } else if squeeze_repeats {
                    // squeeze-repats mode and no second argument given, skipping translation
                    only_squeeze_repeats = true
                }
            },
            // Deleting characters and then running squeeze-repeats on them makes no sense.
            // GNU tr aborts in this case. We do the same.
            TrMode::Delete => if squeeze_repeats && env::args().skip(1).nth(first_non_option_argument + 1).is_none() {
                            panic!("tr: missing operand after ‘{}’\n\
                            Two strings must be given when both deleting and squeezing repeats.\n\
                            Try 'tr --help' for more information.",
                            env::args().skip(1).nth(first_non_option_argument).unwrap())
            },
        };

        // determine which characters to use for squeezing repeats
        let chars_to_squeeze: Vec<char> = if only_squeeze_repeats {
            escape_parser::parse(env::args().skip(1).nth(first_non_option_argument).unwrap())
        } else if (operation_mode == TrMode::Delete) && squeeze_repeats {
            escape_parser::parse(env::args().skip(1).nth(first_non_option_argument + 1).unwrap())
        } else {
            Vec::new()
        };

        // main tr loop
        while stdin.read_line(&mut buffer).unwrap() > 0 {
            if ! only_squeeze_repeats { //TODO: skip this if set2 is empty and -t is given
                for character in buffer.chars() {
                    if chars_to_replace.contains(&character) ^ complement_set {
                        match operation_mode {
                             //TODO: handle this as set instead of inserting single char
                            TrMode::ReplaceWith(ref chars_to_insert) => result.push(translate(character, &chars_to_replace, chars_to_insert)),
                            TrMode::Delete => {}, //do not copy char to output buffer
                        }
                    } else {
                        result.push(character)
                    };
                };
            };

            // separate pass for squeezing repeated characters
            // FIXME: doesn't squeeze repeated '\n' characters because it's line-buffered.
            // This should be fixed by reading the input char-by-char,
            // but that is an unstable feature as of Rust 1.10
            if squeeze_repeats {
                let mut previous_character = Option::None;
                for character in if only_squeeze_repeats {buffer.chars()} else {result.chars()} {
                    if previous_character.is_some()
                    && previous_character.unwrap() == character
                    && (chars_to_squeeze.contains(&character) ^ complement_set) {
                        // do not copy this char to output buffer
                    } else {
                        squeezed_result.push(character)
                    };
                    previous_character = Option::Some(character);
                }
                print!("{}", squeezed_result);
            } else {
                print!("{}", result);
            }
            // output is line-buffered, but we do not always print \n,
            // so without the following line we may never output anything
            io::stdout().flush().unwrap();
            buffer.clear();
            result.clear();
            squeezed_result.clear();
        }

    } else {
        println!("Usage: tr [OPTION]... SET1 [SET2]");
    }
}


fn translate(character: char, set1: &Vec<char>, set2: &Vec<char>) -> char {
    match set1.iter().rposition(|&c| c == character) {
        None => character,
        Some(index) => match set2.get(index) {
            Some(char_to_insert) => *char_to_insert,
            None => set2[set2.len() - 1],
        }
    }
}



