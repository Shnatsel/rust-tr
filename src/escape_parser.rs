use std;

// Escape parser for command-line arguments SET1 and SET2
pub fn parse(input_string: String) -> Vec<char> {
    let input_chars: Vec<char> = input_string.chars().collect();
    //let mut parser_mode = EscapeParserMode::Normal; //XXX DROP
    //let mut previous_character: std::option::Option<char> = Option::None; //XXX DROP
    let mut escaped_octal_digits: Vec<char> = Vec::with_capacity(3);
    let mut unescaped_characters: Vec<char> = Vec::with_capacity(input_chars.len()); //TODO: also store character classes in this array, wrapped in enums?
    let mut final_character_set: Vec<char> = Vec::with_capacity(input_chars.len());
    let mut index: usize = 0;
    let mut next_index: usize = 0;
    while index < input_chars.len() {
        // On every iteration "index" is unaltered,
        // so we can address characters by index starting from the one we're handling.
        // We also need to keep track of the amount of characters that we have processed
        // (consumed), which is what next_index variable is for.
        // It is incremented every time we consume an *input* character,
        // and the next iteration starts from the previous one's next_index.
        next_index += 1;
        if input_chars[index] == '\\' {
            if is_escaped(index, &input_chars) || index == input_chars.len() - 1 {
                unescaped_characters.push('\\')
            } else { // this backslash escapes a special character
                next_index += 1;
                match input_chars[index+1] {
                    '\\' => unescaped_characters.push('\\'),
                    '[' => unescaped_characters.push('['),
                    'n' => unescaped_characters.push('\n'),
                    'r' => unescaped_characters.push('\r'),
                    't' => unescaped_characters.push('\t'),
                    'a' => unescaped_characters.push('\x07'),
                    'b' => unescaped_characters.push('\x08'),
                    'f' => unescaped_characters.push('\x0C'),
                    'v' => unescaped_characters.push('\x0B'),
                    digit @ '0' ... '7' => {
                        escaped_octal_digits.push(digit);
                        let long_octal_possible = match digit {
                            '0' ... '3' => true,
                            '4' ... '9' => false,
                            _ => panic!("Your changes are bad and you should feel bad."),
                        };
                        if input_chars.get(index+2).is_some()
                        && is_octal_digit(input_chars[index+2]) {
                            next_index += 1;
                            escaped_octal_digits.push(input_chars[index+2]);
                        };
                        if long_octal_possible
                        && input_chars.get(index+3).is_some()
                        && is_octal_digit(input_chars[index+3]) {
                            next_index += 1;
                            escaped_octal_digits.push(input_chars[index+3]);
                        };
                        unescaped_characters.push(octal_digits_to_char(&escaped_octal_digits));
                        escaped_octal_digits.clear();
                    },
                    nonspecial_char => {
                        unescaped_characters.push('\\');
                        unescaped_characters.push(nonspecial_char);
                    },
                };
            }
        } else {
            unescaped_characters.push(input_chars[index]);
        }
        index = next_index;
    }

    // Parsing of ranges implemented as a separate pass
    // because they need to operate on already unescaped characters
    // to handle user input like '\50-\53'
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

fn is_octal_digit(character: char) -> bool {
    match character {'0' ... '7' => true, _ => false}
}

fn octal_digits_to_char(octal_digits: &Vec<char>) -> char {
    let mut final_char_code = 0u32;
    for (order, digit_char) in octal_digits.iter().rev().enumerate() {
        let octal_digit = digit_char.to_digit(8).unwrap();
        final_char_code += octal_digit * 8u32.pow(order as u32);
    }
    return std::char::from_u32(final_char_code).unwrap()
}

// Whether the character with the given index is escaped or not
// "Escaped" means "preceded by a backslash that is not escaped"
fn is_escaped(index: usize, context: &Vec<char>) -> bool {
    match index {
        0 => false,
        1 => context[0] == '\\',
        x => context[x-1] == '\\' && !(context[x-2] == '\\')
    }
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
fn escaped_escape_sequences() {
    assert_eq!(parse("a\\\\nb".to_string()), "a\\nb".chars().collect::<Vec<char>>());
    assert_eq!(parse("\\\\123".to_string()), "\\123".chars().collect::<Vec<char>>());
}

#[test]
fn invalid_escape_sequences() {
    assert_eq!(parse("\\m".to_string()), "\\m".chars().collect::<Vec<char>>());
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

