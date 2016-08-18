use std;

#[derive(PartialEq)]
#[derive(Debug)]
enum EscapeParserMode {
    Normal,
    EscapedOctalDigits,
    //TODO: more modes, or just separate passes?
}

// Escape parser for command-line arguments SET1 and SET2
pub fn parse(input_string: String) -> Vec<char> {
    let mut parser_mode = EscapeParserMode::Normal;
    let mut previous_character: std::option::Option<char> = Option::None;
    let mut escaped_octal_digits: Vec<char> = Vec::with_capacity(3);
    let mut unescaped_characters: Vec<char> = Vec::with_capacity(input_string.len()); //TODO: also store character classes in this array, wrapped in enums?
    let mut final_character_set: Vec<char> = Vec::with_capacity(input_string.len());
    for character in input_string.chars() {
        match parser_mode {
            EscapeParserMode::Normal => {
                if character == '\\' {
                    // escapes special character; do nothing and handle the special char next round
                } else if previous_character.is_some() && previous_character.unwrap() == '\\' {
                    match character {
                        '\\' => unescaped_characters.push('\\'),
                        'n' => unescaped_characters.push('\n'),
                        'r' => unescaped_characters.push('\r'),
                        't' => unescaped_characters.push('\t'),
                        'a' => unescaped_characters.push('\x07'),
                        'b' => unescaped_characters.push('\x08'),
                        'f' => unescaped_characters.push('\x0C'),
                        'v' => unescaped_characters.push('\x0B'),
                        digit @ '0' ... '7' => {escaped_octal_digits.push(digit);
                                                parser_mode = EscapeParserMode::EscapedOctalDigits},
                        _ => panic!("Unknown escape sequence \\{}", character),
                    }
                } else {
                    unescaped_characters.push(character);
                };
            },
            EscapeParserMode::EscapedOctalDigits => {
                let is_octal_digit = match character {'0' ... '7' => true, _ => false};
                let first_digit_too_big_for_long_octal = if escaped_octal_digits.len() == 0 {
                    false
                } else {
                    // Octal values of 400 and higher trigger a warning from GNU tr.
                    // It interprets the first two digits as the character code,
                    // and the last digit as a standalone digit.
                    match escaped_octal_digits[0] {
                        '0' ... '3' => false,
                        '4' ... '9' => true,
                        _ => panic!("Internal error: a non-digit was somehow put in octal digits buffer.\nPlease file a bug report."),
                    }
                };
                // process the character we got
                if is_octal_digit {escaped_octal_digits.push(character)};
                // check if we should stop parsing incoming chars as escaped octal digits
                if !is_octal_digit || (first_digit_too_big_for_long_octal && escaped_octal_digits.len() == 2) || escaped_octal_digits.len() == 3 {
                    unescaped_characters.push(octal_digits_to_char(&escaped_octal_digits));
                    escaped_octal_digits.clear();
                    parser_mode = EscapeParserMode::Normal;
                }
                // this block is here because if we encounter a regular character,
                // it should be pushed AFTER we handle the escaped character
                // example sequence: a\12b
                if is_octal_digit {} // already handled above
                else if character == '\\' {} // do nothing, will be handled next round
                else {unescaped_characters.push(character)};
            },
        };
        previous_character = Option::Some(character);
    };
    // wrap up parsing after the loop
    match parser_mode {
        EscapeParserMode::Normal => if previous_character.unwrap_or(' ') == '\\' {unescaped_characters.push('\\')}, //wrap up parsing a trailing \ 
        EscapeParserMode::EscapedOctalDigits => {unescaped_characters.push(octal_digits_to_char(&escaped_octal_digits))}, // wrap up parsing unclosed octals, e.g. in "abcd\53"
    };
    // XXX Ranges
    for (index, unescaped_char) in unescaped_characters.iter().enumerate() {
        if *unescaped_char == '-' && index > 0 && index < (unescaped_characters.len() - 1) {
            let lower_bound = unescaped_characters[index-1] as u32 + 1;
            let upper_bound = unescaped_characters[index+1] as u32;
            if lower_bound < upper_bound {
                for char_code in lower_bound .. upper_bound {
                    final_character_set.push(std::char::from_u32(char_code).unwrap());
                }
            } else if unescaped_characters[index-1] == unescaped_characters[index+1] {
                // We've been given a range like "a-a", in this case GNU tr simply prints "a".
                // We have already inserted "a" once and are going to insert another one
                // on the next iteration, so remove the one we've already inserted.
                final_character_set.pop();
            } else {
                panic!("tr: range-endpoints of '{}-{}' are in reverse collating sequence order",
                       unescaped_characters[index-1], unescaped_characters[index+1]);
            }
        } else {
            final_character_set.push(*unescaped_char);
        }
    };
    return final_character_set;
}

fn octal_digits_to_char(octal_digits: &Vec<char>) -> char {
    let mut final_char_code = 0u32;
    for (order, digit_char) in octal_digits.iter().rev().enumerate() {
        let octal_digit = digit_char.to_digit(8).unwrap();
        final_char_code += octal_digit * 8u32.pow(order as u32);
    }
    return std::char::from_u32(final_char_code).unwrap()
}


#[test]
fn literal_input() {
    assert_eq!(parse("abcd".to_string()), "abcd".chars().collect::<Vec<char>>());
    assert_eq!(parse("абвг".to_string()), "абвг".chars().collect::<Vec<char>>());
}

#[test]
fn character_ranges() {
    assert_eq!(parse("ab-d".to_string()), "abcd".chars().collect::<Vec<char>>());
    assert_eq!(parse("a-def".to_string()), "abcdef".chars().collect::<Vec<char>>());
    assert_eq!(parse("a-a".to_string()), "a".chars().collect::<Vec<char>>());
    assert_eq!(parse("a-".to_string()), "a-".chars().collect::<Vec<char>>());
    assert_eq!(parse("-a".to_string()), "-a".chars().collect::<Vec<char>>());
}

#[test]
#[should_panic(expected = "tr: range-endpoints of 'd-a' are in reverse collating sequence order")]
fn invalid_character_range() {
    parse("d-a".to_string());
}

#[test]
fn escape_sequences() {
    assert_eq!(parse("a\\nb".to_string()), "a\nb".chars().collect::<Vec<char>>());
    assert_eq!(parse("a\\12b".to_string()), "a\nb".chars().collect::<Vec<char>>());
    assert_eq!(parse("a\\123".to_string()), "aS".chars().collect::<Vec<char>>());
    assert_eq!(parse("\\123".to_string()), "S".chars().collect::<Vec<char>>());
    assert_eq!(parse("a\\12".to_string()), "a\n".chars().collect::<Vec<char>>());
    assert_eq!(parse("\\53".to_string()), "+".chars().collect::<Vec<char>>());
}

#[test]
#[ignore] //known to fail
fn escaped_escape_sequences() {
    assert_eq!(parse("a\\\\nb".to_string()), "a\\nb".chars().collect::<Vec<char>>());
    assert_eq!(parse("\\\\123".to_string()), "\\\\123".chars().collect::<Vec<char>>());
}

#[test]
fn overflown_octals() {
    // GNU tr detects potentially overflowing octals and cuts them off at 2 digits
    assert_eq!(parse("\\525".to_string()), "*5".chars().collect::<Vec<char>>());
}

#[test]
fn trailing_slashes() {
    assert_eq!(parse("\\\\".to_string()), "\\".chars().collect::<Vec<char>>());
    assert_eq!(parse("\\".to_string()), "\\".chars().collect::<Vec<char>>());
    assert_eq!(parse("a\\".to_string()), "a\\".chars().collect::<Vec<char>>());
}

#[test]
fn ranges_on_escape_sequences() {
    assert_eq!(parse("\\120-\\123".to_string()), "PQRS".chars().collect::<Vec<char>>());
    assert_eq!(parse("\\50-\\57".to_string()), "()*+,-./".chars().collect::<Vec<char>>());
    assert_eq!(parse("\\53-\\53".to_string()), "+".chars().collect::<Vec<char>>());
}

