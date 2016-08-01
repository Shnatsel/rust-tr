use std::io;
use std::io::prelude::*;

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer = String::new();
    let mut result = String::new();

    while stdin.read_line(&mut buffer).unwrap() > 0 {
        for character in buffer.chars() {
            if character == ' ' {result.push('_')} else {result.push(character)}
        }
        print!("{}", result);
        buffer.clear();
        result.clear();
    }
}
