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

    let args: Vec<_> = env::args().collect();
    if args.len() > 2 {
        //TODO: leave them as strings and check if char is in string
        let char_to_replace = args[1].chars().nth(0).unwrap();
        let char_to_insert = args[2].chars().nth(0).unwrap();
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
