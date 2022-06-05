use std::str::Chars;

use crate::json_object::JSONObject;

fn consume_whitespace(chars: Chars) -> Chars {
    chars.as_str().trim_start().chars()
}

pub fn parse(json_string: &str) -> Result<JSONObject, String> {
    let (object, _) = parse_any(json_string)?;
    Ok(object)
}

fn parse_any(json_string: &str) -> Result<(JSONObject, &str), String> {
    let trimmed = json_string.trim();
    let first_char = trimmed
        .chars()
        .next()
        .ok_or_else(|| "No chars to parse".to_string())?;

    match first_char {
        '0'..='9' => parse_number(json_string),
        '{' => parse_object(json_string),
        '\"' => parse_string(json_string),
        '[' => parse_array(json_string),
        'a'..='z' => parse_keyword(json_string),
        _ => {
            let excerpt: &str = &trimmed[0..250.min(trimmed.len())];
            Err(format!(
                "Unrecognized symbol {first_char} at beginning of element, followed by: {excerpt}"
            ))
        }
    }
}

fn parse_keyword(json_string: &str) -> Result<(JSONObject, &str), String> {
    let mut just_keyword: &str = json_string;
    let mut remainder: &str = json_string.get(json_string.len()..).unwrap();

    for (i, c) in json_string.chars().enumerate() {
        match c {
            'a'..='z' => {}
            _ => {
                just_keyword = json_string
                    .get(0..i)
                    .ok_or_else(|| "Found non-alphabet character in keyword".to_string())?;

                remainder = json_string.get(i..).unwrap();
                break;
            }
        }
    }

    match just_keyword {
        "true" => Ok((JSONObject::Bool { value: true }, remainder)),
        "false" => Ok((JSONObject::Bool { value: false }, remainder)),
        "null" => Ok((JSONObject::Null, remainder)),
        _ => Err(format!("Unrecognized keyword {}", just_keyword)),
    }
}

fn parse_number(json_string: &str) -> Result<(JSONObject, &str), String> {
    let mut just_number: &str = json_string;
    let mut remainder: &str = json_string.get(json_string.len()..).unwrap();

    let mut float: bool = false;
    for (i, c) in json_string.chars().enumerate() {
        match c {
            '0'..='9' => {}
            '.' => float = true,
            _ => {
                just_number = json_string.get(0..i).ok_or_else(|| {
                    "Found non-numeric character while parsing number".to_string()
                })?;

                remainder = json_string.get(i..).unwrap();
                break;
            }
        }
    }

    match float {
        true => {
            let number = just_number
                .parse::<f32>()
                .map_err(|_| format!("Failed to parse number {just_number} as f32"))?;

            Ok((JSONObject::Float { value: number }, remainder))
        }
        false => {
            let number = just_number
                .parse::<i64>()
                .map_err(|_| format!("Failed to parse number {just_number} as i64"))?;

            Ok((JSONObject::Integer { value: number }, remainder))
        }
    }
}

fn parse_string(json_string: &str) -> Result<(JSONObject, &str), String> {
    let mut chars = json_string.trim_start().chars();

    let first_char = chars
        .next()
        .ok_or_else(|| "No chars found while parsing string".to_string())?;

    if first_char != '\"' {
        return Err(format!(
            "Strings must start with \" but {} was found",
            first_char
        ));
    }

    let mut result_string = String::new();

    loop {
        let next_char = chars
            .next()
            .ok_or_else(|| "Input ended without closing string".to_string())?;
        match next_char {
            '\\' => {
                let next_char = chars.next().ok_or_else(|| {
                    "Input ended while parsing string escape character".to_string()
                })?;
                result_string.push(match next_char {
                    // Escaped characters
                    '\"' => next_char,
                    't' => '\t',
                    'n' => '\n',
                    'r' => '\r',
                    '\\' => '\\',
                    'u' => {
                        let mut unicode_seq: String = String::new();
                        for _ in 0..4 {
                            unicode_seq.push(chars.next().ok_or_else(|| {
                                "Ran out of characters while parsing Unicode sequence".to_string()
                            })?);
                        }
                        let unicode_integer: u32 =
                            i64::from_str_radix(&unicode_seq, 16).map_err(|_| {
                                format!("Could not parse Unicode sequence {unicode_seq}")
                            })? as u32;
                        let unicode_char: char = char::from_u32(unicode_integer).ok_or(format!(
                            "Failed to convert Unicode sequence {unicode_seq} to char"
                        ))?;
                        unicode_char
                    }
                    _ => next_char,
                });
            }
            '\"' => {
                break;
            }
            _ => result_string.push(next_char),
        }
    }

    Ok((
        JSONObject::String {
            value: result_string,
        },
        chars.as_str(),
    ))
}

fn parse_array(json_string: &str) -> Result<(JSONObject, &str), String> {
    let mut values = Vec::<JSONObject>::new();
    let mut chars = json_string.chars();

    let first_char = chars
        .next()
        .ok_or_else(|| "No chars found while parsing array".to_string())?;
    if first_char != '[' {
        return Err(format!("Arrays must start with [ but found {}", first_char));
    }

    // Allow any whitespace after [
    chars = consume_whitespace(chars);

    loop {
        let next_token = chars.as_str();

        // Allow a , or a ]
        let next_char = chars
            .next()
            .ok_or_else(|| "Input ended without closing array".to_string())?;
        match next_char {
            ',' => {
                // Allow any whitespace after ,
                chars = consume_whitespace(chars);
            }
            ']' => {
                // Done with array
                break;
            }
            _ => {
                let (object, remainder) = parse_any(next_token)?;
                chars = remainder.chars();

                values.push(object);

                // Allow any whitespace after value
                chars = consume_whitespace(chars);
            }
        }
    }

    Ok((JSONObject::Array { values }, chars.as_str()))
}

fn parse_object(json_string: &str) -> Result<(JSONObject, &str), String> {
    let mut key: JSONObject;
    let mut value: JSONObject;
    let mut children = Vec::<(JSONObject, JSONObject)>::new();

    let mut chars = json_string.trim_start().chars();

    // Parse opening
    {
        let first_char = chars
            .next()
            .ok_or_else(|| "No chars found while parsing object".to_string())?;

        if first_char != '{' {
            return Err("Object must start with {".to_string());
        }

        // Allow any amount of whitespace after opening brace
        chars = consume_whitespace(chars);
    }

    loop {
        // Parse key
        {
            let token_start: &str = chars.as_str();
            let next_char = chars
                .next()
                .ok_or_else(|| "Input ended without closing object".to_string())?;

            match next_char {
                '}' => {
                    return Ok((JSONObject::Object { children }, chars.as_str()));
                }
                _ => {
                    let (newkey, remainder) = parse_any(token_start)?;
                    key = newkey;
                    chars = remainder.chars();
                }
            }

            // Allow any amount of whitespace after key
            chars = consume_whitespace(chars);
        }

        // Parse separator
        {
            let next_char = chars
                .next()
                .ok_or_else(|| "Input ended while expecting ':'".to_string())?;
            if next_char != ':' {
                return Err(format!(
                    "Missing : between key and value (found {next_char})"
                ));
            }
            // Allow any amount of whitespace after :
            chars = consume_whitespace(chars);
        }

        // Parse value
        {
            let (newvalue, remainder) = parse_any(chars.as_str())?;
            value = newvalue;
            chars = remainder.chars();
            // Can have any amount of whitespace after value
            chars = consume_whitespace(chars);
        }

        // Add key:value to children
        children.push((key, value));

        // Needs to be either ',' or '}' next
        {
            let next_char = chars
                .next()
                .ok_or_else(|| "Input ended without closing object field".to_string())?;
            match next_char {
                ',' => {
                    // Can have any amount of whitespace after ,
                    chars = consume_whitespace(chars);
                }
                '}' => {
                    return Ok((JSONObject::Object { children }, chars.as_str()));
                }
                _ => {
                    return Err(format!(
                        "Unexpected symbol {} while parsing object",
                        next_char
                    ));
                }
            }
        }
    }
}
