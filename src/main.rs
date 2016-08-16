use std::io;
use std::env;
use std::io::prelude::*;

#[derive(PartialEq)]
#[derive(Debug)]
enum TrMode {
    ReplaceWith(Vec<char>),
    Delete,
    //TODO: case conversion
}

#[derive(PartialEq)]
#[derive(Debug)]
enum EscapeParserMode {
    Normal,
    EscapedOctalDigits,
    CharacterRange,
}

fn escaped_sequences_to_chars(input_string: String) -> String {
    let mut parser_mode = EscapeParserMode::Normal;
    let mut previous_character: std::option::Option<char> = Option::None;
    let mut lower_character_for_range: std::option::Option<char> = Option::None;
    let mut escaped_octal_digits: Vec<char> = Vec::with_capacity(3);
    let mut output_string = String::new();
    for character in input_string.chars() {
        match parser_mode {
            EscapeParserMode::Normal => {
                if character == '\\' {
                    // escapes special character; do nothing and handle the special char next round
                } else if previous_character.is_some() && previous_character.unwrap() == '\\' {
                    match character {
                        '\\' => output_string.push('\\'),
                        'n' => output_string.push('\n'),
                        'r' => output_string.push('\r'),
                        't' => output_string.push('\t'),
                        'a' => output_string.push('\x07'),
                        'b' => output_string.push('\x08'),
                        'f' => output_string.push('\x0C'),
                        'v' => output_string.push('\x0B'),
                        digit @ '0' ... '7' => {escaped_octal_digits.push(digit);
                                                parser_mode = EscapeParserMode::EscapedOctalDigits},
                        _ => panic!("Unknown escape sequence \\{}", character),
                    }
                } else if previous_character.is_some() && character == '-' {
                    lower_character_for_range = previous_character;
                    parser_mode = EscapeParserMode::CharacterRange;
                    // on the next iteration we will get the upper bound,
                    // calculate the range and push the chars to output string
                } else {
                    output_string.push(character);
                };
            },
            EscapeParserMode::CharacterRange => {
                let lower_bound = lower_character_for_range.unwrap() as u32 + 1;
                let upper_bound = character as u32 + 1;
                if lower_bound < upper_bound {
                    for char_code in lower_bound .. upper_bound {
                        output_string.push(std::char::from_u32(char_code).unwrap());
                    }
                } else if lower_bound == upper_bound {
                    // we've been given a range like "a-a".
                    // in this case GNU tr simply prints "a"
                    // normal-mode loop has already pushed "a" to output string,
                    // so we don't have to do anything here.
                } else {
                    panic!("tr: range-endpoints of '{}-{}' are in reverse collating sequence order",
                           lower_character_for_range.unwrap(), character);
                }
                lower_character_for_range = Option::None;
                parser_mode = EscapeParserMode::Normal;
            },
            EscapeParserMode::EscapedOctalDigits => {
                let is_octal_digit = match character {'0' ... '7' => true, _ => false};
                // process the character we got
                if is_octal_digit {escaped_octal_digits.push(character)};
                // check if we should stop parsing incoming chars as escaped octal digits
                if !is_octal_digit || escaped_octal_digits.len() == 3 {
                    // Octal values of 400 and higher trigger a warning from GNU tr.
                    // It interprets them as two-byte sequence.
                    // We simply error out to avoid dubious feature bloat.
                    match escaped_octal_digits[0] {
                        '4' ... '9' => panic!("Character codes higher than \\399 are not valid ASCII characters"),
                        _ => {},
                    };
                    //TODO: split this block into a function to avoid copypasting this below
                    let mut final_char_code = 0u32;
                    escaped_octal_digits.reverse(); // for use in the loop below
                    for (order, digit_char) in escaped_octal_digits.iter().enumerate() {
                        let octal_digit = digit_char.to_digit(8).unwrap();
                        final_char_code += octal_digit * 8u32.pow(order as u32);
                    }
                    output_string.push(std::char::from_u32(final_char_code).unwrap());
                    escaped_octal_digits.clear();
                    parser_mode = EscapeParserMode::Normal;
                }
                // this block is here because if we encounter a regular character,
                // it should be pushed AFTER we handle the escaped character
                // example sequence: a\12b
                if is_octal_digit {} // already handled above
                else if character == '\\' {} // do nothing, will be handled next round
                else {output_string.push(character)};
            },
        };
        previous_character = Option::Some(character);
    };
    // wrap up parsing after the loop
    //println!("Parser mode at the end: {:?}", parser_mode);
    match parser_mode {
        // wrap up unclosed ranges
        EscapeParserMode::CharacterRange => output_string.push('-'),
        _ => {}, //TODO: wrap up parsing after the loop: trailing \, unclosed octals
    };
    return output_string;
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

/*
// parser regression testing
    println!("ab-d translates to: \"{}\"", escaped_sequences_to_chars("ab-d".to_string()));
    println!("a-def translates to: \"{}\"", escaped_sequences_to_chars("a-def".to_string()));
    println!("a-a translates to: \"{}\"", escaped_sequences_to_chars("a-a".to_string()));
    println!("a- translates to: \"{}\"", escaped_sequences_to_chars("a-".to_string())); //TODO
//    println!("d-a translates to: \"{}\"", escaped_sequences_to_chars("d-a".to_string())); //panics
    println!("a\\nb translates to: \"{}\"", escaped_sequences_to_chars("a\\nb".to_string()));
    println!("a\\12b translates to: \"{}\"", escaped_sequences_to_chars("a\\12b".to_string()));
    println!("a\\12 translates to: \"{}\"", escaped_sequences_to_chars("a\\12".to_string())); //TODO
    println!("\\123 translates to: \"{}\"", escaped_sequences_to_chars("\\123".to_string()));
//    println!("\\755 translates to: \"{}\"", escaped_sequences_to_chars("\\755".to_string())); //panics
    println!("\\12-\\123 translates to: \"{}\"", escaped_sequences_to_chars("\\12-\\123".to_string())); //TODO
*/

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
        let mut chars_to_replace: Vec<char> = escaped_sequences_to_chars(env::args().skip(1).nth(first_non_option_argument).unwrap()).chars().collect();
        match operation_mode {
            // if we're replacing, read the chars to replace with from command line
            TrMode::ReplaceWith(_) => {
                if env::args().skip(1).nth(first_non_option_argument + 1).is_some() {
                    let chars_to_insert: Vec<char> = escaped_sequences_to_chars(env::args().skip(1).nth(first_non_option_argument + 1).unwrap()).chars().collect();
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
            escaped_sequences_to_chars(env::args().skip(1).nth(first_non_option_argument).unwrap()).chars().collect()
        } else if squeeze_repeats {
            // delete mode with squeezing repeats, use second argument
            escaped_sequences_to_chars(env::args().skip(1).nth(first_non_option_argument + 1).unwrap()).chars().collect()
        } else {
            Vec::new()
        };

        // main tr loop
        while stdin.read_line(&mut buffer).unwrap() > 0 {
            if ! only_squeeze_repeats {
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
            };

            // separate pass for squeezing repeated characters
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
