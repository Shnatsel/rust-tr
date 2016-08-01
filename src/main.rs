use std::io;
use std::env;
use std::io::prelude::*;

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::new();
    let mut result = String::new();

    let args: Vec<_> = env::args().collect();
    if args.len() > 2 {
        let char_to_replace = args[1].chars().nth(0).unwrap();
        let char_to_insert = args[2].chars().nth(0).unwrap();
        while stdin.read_line(&mut buffer).unwrap() > 0 {
            for character in buffer.chars() {
                if character == char_to_replace {result.push(char_to_insert)} else {result.push(character)};
            }
            print!("{}", result);
            buffer.clear();
            result.clear();
        }
    } else {
        println!("Usage: tr [OPTION]... SET1 [SET2]");
    }
}
