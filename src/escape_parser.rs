use std;

#[derive(PartialEq)]
#[derive(Debug)]
enum EscapeParserMode {
    Normal,
    EscapedOctalDigits,
    CharacterRange,
}

// Escape parser for command-line arguments SET1 and SET2
pub fn parse(input_string: String) -> String {
    let mut parser_mode = EscapeParserMode::Normal;
    let mut previous_character: std::option::Option<char> = Option::None;
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
                    parser_mode = EscapeParserMode::CharacterRange;
                    // on the next iteration we will get the upper bound,
                    // calculate the range and push the chars to output string
                } else {
                    output_string.push(character);
                };
            },
            EscapeParserMode::CharacterRange => {
                let lower_bound = output_string.chars().rev().nth(0).unwrap() as u32 + 1;
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
                           output_string.chars().rev().nth(0).unwrap(), character);
                }
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
        EscapeParserMode::CharacterRange => output_string.push('-'), // wrap up unclosed ranges
        EscapeParserMode::Normal => if previous_character.unwrap_or(' ') == '\\' {output_string.push('\\')}, //wrap up parsing a trailing \
        _ => {}, //TODO: wrap up parsing unclosed octals after the loop
    };
    return output_string;
}

#[test]
fn literal_input() {
    assert_eq!(parse("abcd".to_string()), "abcd".to_string());
}

#[test]
fn character_ranges() {
    assert_eq!(parse("ab-d".to_string()), "abcd".to_string());
    assert_eq!(parse("a-def".to_string()), "abcdef".to_string());
    assert_eq!(parse("a-a".to_string()), "a".to_string());
    assert_eq!(parse("a-".to_string()), "a-".to_string());
    assert_eq!(parse("-a".to_string()), "-a".to_string());
}

#[test]
#[should_panic(expected = "tr: range-endpoints of 'd-a' are in reverse collating sequence order")]
fn invalid_character_range() {
    parse("d-a".to_string());
}

#[test]
fn escape_sequences() {
    assert_eq!(parse("a\\nb".to_string()), "a\nb".to_string());
    assert_eq!(parse("a\\12b".to_string()), "a\nb".to_string());
    assert_eq!(parse("a\\123".to_string()), "aS".to_string());
    assert_eq!(parse("\\123".to_string()), "S".to_string());
    println!("a\\12 translates to: \"{}\"", parse("a\\12".to_string())); //TODO
    println!("\\53 translates to: \"{}\"", parse("\\53".to_string())); //TODO
}

#[test]
fn trailing_slashes() {
    assert_eq!(parse("\\\\".to_string()), "\\".to_string());
    assert_eq!(parse("\\".to_string()), "\\".to_string());
    assert_eq!(parse("a\\".to_string()), "a\\".to_string());
}

#[test]
#[ignore] //not implemented yet
fn ranges_on_escape_sequences() {
    assert_eq!(parse("\\120-\\123".to_string()), "PQRS".to_string());
}

#[test]
#[ignore] //not implemented yet
fn overflown_octals() {
    // GNU tr detects potentially overflowing octals and cuts them off at 2 digits
    assert_eq!(parse("\\525".to_string()), "*5".to_string());
}
