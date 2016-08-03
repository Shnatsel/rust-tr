use std::io;
use std::env;
use std::io::prelude::*;

enum TrMode {
    Replace(Vec<char>, Vec<char>),
    Delete(Vec<char>),
}

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::new();
    let mut result = String::new();
    let mut operation_mode = TrMode::Replace(Vec::new(), Vec::new()); //default mode
    let mut squeeze_repeats = false;
    let mut chars_to_squeeze: Vec<char> = Vec::new();
    let mut complement_set = false;
    let mut truncate_set = false;
    let mut first_argument_requires_escaping_dash = true; //for GNU-compatible option parsing

    if env::args().count() > 2 {
        let mut first_non_option_argument = 0;
        for (arg_number, arg) in env::args().skip(1).enumerate() {
            match arg.to_string().as_str() {
                "-c" | "-C" | "--complement" => complement_set = true,
                "-d" | "--delete" => operation_mode = TrMode::Delete(Vec::new()),
                "-s" | "--squeeze-repeats" => squeeze_repeats = true,
                "-t" | "--truncate-set1" => truncate_set = true,
                "--" => {first_argument_requires_escaping_dash = false;
                         first_non_option_argument = arg_number + 1; break},
                _ => if arg.to_string().chars().nth(0).unwrap() == '-' {panic!("Unknown argument: {}", arg)}
                     else {first_non_option_argument = arg_number; break},
            }
        }
        match operation_mode {
            TrMode::Replace(_,_) => operation_mode = TrMode::Replace(
                                    env::args().skip(1).nth(first_non_option_argument).unwrap().chars().collect(),
                                    env::args().skip(1).nth(first_non_option_argument + 1).unwrap().chars().collect()),
            TrMode::Delete(_) => operation_mode = TrMode::Delete(env::args().skip(1).nth(first_non_option_argument).unwrap().chars().collect()),
        }
        //FIXME: temp hack
        let chars_to_replace = env::args().skip(1).nth(first_non_option_argument).unwrap();
        let char_to_replace = chars_to_replace.chars().nth(0).unwrap(); //temp hack
        let chars_to_insert = env::args().skip(1).nth(first_non_option_argument + 1).unwrap();
        let char_to_insert = chars_to_insert.chars().nth(0).unwrap(); //temp hack
        // main loop
        while stdin.read_line(&mut buffer).unwrap() > 0 {
            for character in buffer.chars() {
                //TODO: replace == with custom function to handle --complement and multiple chars
                if character == char_to_replace {
                    match operation_mode {
                        TrMode::Replace(_,_) => result.push(char_to_insert),
                        TrMode::Delete(_) => {}, //do not copy char to output buffer
                    }
                } else {result.push(character)};
            }
            print!("{}", result);
            buffer.clear();
            result.clear();
        }
    } else {
        println!("Usage: tr [OPTION]... SET1 [SET2]");
    }
}
