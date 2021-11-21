use std::iter::{Peekable};
use std::str::Chars;

#[derive(Eq, PartialEq)]
#[derive(Debug)]
pub(crate) enum Token {
    Let,
    Function,
    Identifier(String),
    String(String),
    Number(String),
    Whitespace,
    NewLine,
    Equals,
    Plus,
    SemiColon,
    EOF,
}

#[derive(Eq, PartialEq)]
#[derive(Debug)]
enum LexingError {
    UnexpectedCharacter { expected: char, actual: char },
    InvalidChar(char),
    UnexpectedEndOfInput,
}

fn lex(input: &str) -> Result<Vec<Token>, LexingError> {
    let mut output = vec!();
    let mut chars = input.chars().peekable();

    loop {
        match parse_next_token(&mut chars) {
            Err(err) => return Err(err),
            Ok(Token::EOF) => {
                output.push(Token::EOF);
                break;
            }
            Ok(token) => output.push(token),
        }
    }

    return Result::Ok(output);
}

type LexingResult = Result<Token, LexingError>;

fn parse_next_token(input: &mut Peekable<Chars>) -> LexingResult {
    match input.peek() {
        None => Ok(Token::EOF),
        Some(next_char) => {
            match next_char {
                ' ' => consume(input, Token::Whitespace),
                '\t' => consume(input, Token::Whitespace),
                '=' => consume(input, Token::Equals),
                '+' => consume(input, Token::Plus),
                '\n' => consume(input, Token::NewLine),
                ';' => consume(input, Token::SemiColon),
                '"' => parse_string(input),
                '0'..='9' => parse_number(input),
                'A'..='z' => parse_identifier_or_keyword(input),
                unexpected => Result::Err(LexingError::InvalidChar(*unexpected))
            }
        }
    }
}

fn parse_string(input: &mut Peekable<Chars>) -> LexingResult {
    return expect_next(input, '"')
        .and_then(|_| {
            let string_contents = take_while(input, |c| c != '"');
            Ok(Token::String(stringify(string_contents)))
        })
        .and_then(|res| expect_next(input, '"').map(|_| res));
}

fn consume(input: &mut Peekable<Chars>, token: Token) -> Result<Token, LexingError> {
    input.next();
    return Ok(token);
}

fn parse_number(input: &mut Peekable<Chars>) -> Result<Token, LexingError> {
    let mut num_contents = take_while(input, |c| c.is_digit(10));

    if let Some('.') = input.peek() {
        input.next();
        num_contents.push('.');
        num_contents.append(&mut take_while(input, |c| c.is_digit(10)));
    }

    if num_contents.is_empty() {
        Err(LexingError::UnexpectedEndOfInput)
    } else {
        Ok(Token::Number(stringify(num_contents)))
    }
}

fn parse_identifier_or_keyword(input: &mut Peekable<Chars>) -> Result<Token, LexingError> {
    let identifier_or_keyword = take_while(input, |c| c.is_alphanumeric());

    let token = match stringify(identifier_or_keyword).as_str() {
        "let" => Token::Let,
        "function" => Token::Function,
        name => Token::Identifier(name.to_string())
    };

    Ok(token)
}

fn expect_next(input: &mut Peekable<Chars>, expected_char: char) -> Result<(), LexingError> {
    if let Some(c) = input.next() {
        if c == expected_char {
            Ok(())
        } else {
            Err(LexingError::UnexpectedCharacter { actual: c, expected: expected_char })
        }
    } else {
        Err(LexingError::UnexpectedEndOfInput)
    }
}

fn test_word_or_rewind<'a>(input: &mut Peekable<Chars>, word: &'a str) -> Option<&'a str> {
    for c in word.chars() {
        if let Some(peeked_char) = input.peek() {
            if peeked_char == &c {
                return Option::None;
            }
        }
    }
    return Some(word);
}

fn take_while(input: &mut Peekable<Chars>, predicate: impl Fn(char) -> bool) -> Vec<char> {
    let mut output: Vec<char> = vec![];

    while let Some(c) = input.next_if(|c| predicate(*c)) {
        output.push(c)
    }

    return output;
}

fn stringify(chars: Vec<char>) -> String {
    return chars.iter()
        .cloned()
        .collect::<String>();
}

#[cfg(test)]
mod tests {
    use crate::lexer::{lex, LexingError, Token};

    #[test]
    fn empty_returns_eof() {
        let result = lex(&"");
        assert_eq!(result.unwrap(), vec![Token::EOF])
    }

    #[test]
    fn space_and_tab_are_whitespace() {
        let result = lex(&" \t");
        assert_eq!(result.unwrap(), vec![Token::Whitespace, Token::Whitespace, Token::EOF])
    }

    #[test]
    fn strings_and_whitespace_parsed_correctly() {
        let input = "\"wow\" \"this\" \"string\"";
        let output = lex(&input);

        assert_eq!(output.unwrap(), vec![
            Token::String(String::from("wow")),
            Token::Whitespace,
            Token::String(String::from("this")),
            Token::Whitespace,
            Token::String(String::from("string")),
            Token::EOF,
        ])
    }

    #[test]
    fn string_missing_terminator_fails() {
        let input = "\"wow this is a string";
        let output = lex(&input);

        assert_eq!(output.unwrap_err(), LexingError::UnexpectedEndOfInput);
    }

    #[test]
    fn integer_is_parsed_correctly() {
        let input = "1 2 3 45";
        let output = lex(&input);

        assert_eq!(
            output.unwrap(),
            vec![
                Token::Number(String::from("1")),
                Token::Whitespace,
                Token::Number(String::from("2")),
                Token::Whitespace,
                Token::Number(String::from("3")),
                Token::Whitespace,
                Token::Number(String::from("45")),
                Token::EOF,
            ]
        )
    }

    #[test]
    fn float_parsed_correctly() {
        let input = "1.21";
        let result = lex(&input);

        assert_eq!(result.unwrap(), vec![Token::Number(String::from("1.21")), Token::EOF])
    }

    #[test]
    fn invalid_float_lookalike_returns_lexer_error() {
        let input = "1.1.1";
        let result = lex(&input);

        assert_eq!(result.unwrap_err(), LexingError::InvalidChar('.'));
    }

    #[test]
    fn non_keywords_parsed_as_identifiers() {
        let input = "let someValue = 10;";
        let result = lex(&input);

        assert_eq!(
            result.unwrap(),
            vec![
                Token::Let,
                Token::Whitespace,
                Token::Identifier(String::from("someValue")),
                Token::Whitespace,
                Token::Equals,
                Token::Whitespace,
                Token::Number(String::from("10")),
                Token::SemiColon,
                Token::EOF,
            ]
        );
    }
}
