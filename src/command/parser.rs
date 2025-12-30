use crate::command::placeholder::PlaceholderEnum;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{is_not, tag},
    combinator::{all_consuming, map},
    multi::many0,
    sequence::delimited,
};

fn parse_string(input: &str) -> IResult<&str, PlaceholderEnum> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut consumed_len = 0;

    while let Some(&ch) = chars.peek() {
        if ch == '\\' {
            chars.next();
            consumed_len += 1;
            if let Some(next_ch) = chars.next() {
                consumed_len += 1;
                if next_ch == '{' || next_ch == '}' {
                    result.push(next_ch);
                } else {
                    result.push('\\');
                    result.push(next_ch);
                }
            } else {
                result.push('\\');
            }
        } else if ch == '{' {
            break;
        } else {
            result.push(chars.next().unwrap());
            consumed_len += 1;
        }
    }

    if result.is_empty() {
        return Err(nom::Err::Error(nom::error::make_error(input, nom::error::ErrorKind::IsNot)));
    }

    let remaining = &input[consumed_len..];
    Ok((remaining, PlaceholderEnum::new_string(&result)))
}

fn parse_placeholder(input: &str) -> IResult<&str, PlaceholderEnum> {
    let inner = delimited(tag("{"), is_not("}"), tag("}"));
    map(inner, PlaceholderEnum::new)(input)
}

pub(crate) fn parse_all(input: &str) -> IResult<&str, Vec<PlaceholderEnum>> {
    all_consuming(many0(alt((parse_string, parse_placeholder))))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_escaped_braces() {
        let result = parse_string(r#"test\{key\} string"#);
        assert!(result.is_ok());
        println!("result: {:?}", result)
    }

    #[test]
    fn test_parse_mixed() {
        let (remaining, args) = match parse_all(r#"aa test_\{key sequence 100\} aaaaabbb {key sequence 100 alias pk} {reference pk}"#) {
            Ok((nm, args)) => (nm, args),
            Err(e) => {
                println!("Error: {:?}", e);
                panic!("Parsing failed");
            }
        };
        println!("nm: {:?}, args: {:?}", remaining, args);
    }

    #[test]
    fn test_root() {
        let (nm, args) = match parse_all("aa test_{key sequence 100} bbb {key sequence 100 alias pk} {reference pk}") {
            Ok((nm, args)) => (nm, args),
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        };
        println!("nm: {:?}, args: {:?}", nm, args);
    }
}
