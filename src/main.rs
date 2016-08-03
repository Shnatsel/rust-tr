use std::io;
use std::env;
use std::io::prelude::*;

enum TrMode {
    Replace,
    Delete,
    SqueezeRepeats,
}

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::new();
    let mut result = String::new();
    let mut operation_mode = TrMode::Replace; //default
    let mut complement_set = false;
    let mut truncate_set = false;
    let mut first_argument_requires_escaping_dash = true; //for GNU-compatible option parsing

    if env::args().count() > 2 {
        let mut first_non_option_argument = 0;
        for (arg_number, arg) in env::args().skip(1).enumerate() {
            match arg.to_string().as_str() {
                "-c" | "-C" | "--complement" => complement_set = true,
                "-d" | "--delete" => operation_mode = TrMode::Delete,
                "-s" | "--squeeze-repeats" => operation_mode = TrMode::SqueezeRepeats,
                "-t" | "--truncate-set1" => truncate_set = true,
                "--" => {first_argument_requires_escaping_dash = false;
                         first_non_option_argument = arg_number + 1; break},
                _ => if arg.to_string().chars().nth(0).unwrap() == '-' {panic!("Unknown argument: {}", arg)}
                     else {first_non_option_argument = arg_number; break},
            }
        }
        //TODO: leave them as strings and check if char is in string
        println!("First non-option argument: {}", first_non_option_argument);
        let chars_to_replace = env::args().skip(1).nth(first_non_option_argument).unwrap();
        let char_to_replace = chars_to_replace.chars().nth(0).unwrap(); //temp hack
        let char_to_insert = '_'; //args[2].chars().nth(0).unwrap();
        // main loop
        while stdin.read_line(&mut buffer).unwrap() > 0 {
            for character in buffer.chars() {
                //TODO: replace == with custom function to handle --complement and multiple chars
                if character == char_to_replace {
                    match operation_mode {
                        TrMode::Replace => result.push(char_to_insert),
                        TrMode::Delete => {}, //do not copy char to output buffer
                        TrMode::SqueezeRepeats => panic!("Not implemented"),
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
