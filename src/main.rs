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

    if env::args().count() > 2 {
        let mut first_non_option_arg_number = 0;
        for (arg, arg_number) in env::args().enumerate() {
            match arg {
                "-c" => complement_set = true,
                "-C" => complement_set = true,
                "--complement" => complement_set = true,
                "-d" => operation_mode = TrMode::Delete,
                "--delete" => operation_mode = TrMode::Delete,
                "-s" => operation_mode = TrMode::SqueezeRepeats,
                "--squeeze-repeats" => operation_mode = TrMode::SqueezeRepeats,
                "-t" => truncate_set = true,
                "--truncate-set1" => truncate_set = true,
                _ => if first_non_option_arg_number == 0 {}, //first_non_option_arg_number = arg_number
            }
        }
        println!("arg_number: {}", first_non_option_arg_number);
        //TODO: leave them as strings and check if char is in string
        let char_to_replace = ' '; // args[1].chars().nth(0).unwrap();
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
