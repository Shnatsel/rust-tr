use std::io;
use std::io::prelude::*;

fn main() {
    let stdin = io::stdin();
    let mut result = String::new();

    for line in stdin.lock().lines() {
        for character in line.unwrap().chars() {
            if character == ' ' {result.push('_')} else {result.push(character)}
        }
    println!("{}", result);
    }
}
