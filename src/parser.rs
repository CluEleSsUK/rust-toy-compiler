use crate::lexer::Token;

#[derive(Debug, PartialEq, Clone)]
enum ValueType {
    Integer(i32),
    Float(f32),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
enum Operator {
    Plus,
    Minus,
}

#[derive(Debug, PartialEq, Clone)]
enum Expression {
    Value { value: ValueType },
    Infix { left: Box<Expression>, operator: Operator, right: Box<Expression> },
    Assignment { identifier: String, value: Box<Expression> },
}

#[derive(Debug, PartialEq, Clone)]
enum ParsingError {
    NumberValueInvalid,
    UnknownError,
}

type ParsingResult = Result<Vec<Expression>, ParsingError>;
type CurrentToken = usize;
type IntermediateParsingResult<'a> = Result<(CurrentToken, &'a mut Vec<Expression>), ParsingError>;

fn parse(tokens: &mut Vec<Token>) -> ParsingResult {
    return parse_recursive(tokens, 0, vec![].to_vec());
}

fn parse_recursive(tokens: &Vec<Token>, current: usize, mut output: Vec<Expression>) -> ParsingResult {
    if current >= tokens.len() {
        return Ok(output.to_vec());
    }

    return match parse_next_expr(tokens, current, &mut output) {
        Err(e) => Err(e),
        Ok((next_current, next_output)) => parse_recursive(tokens, next_current, next_output.to_vec())
    };
}

fn parse_next_expr<'a>(input: &Vec<Token>, current: usize, output: &'a mut Vec<Expression>) -> IntermediateParsingResult<'a> {
    if current >= input.len() {
        return Ok((current, output));
    }
    match input.get(current) {
        None => Ok((current + 1, output)),
        Some(token) =>
            match token {
                Token::Number(n) => parse_number_expr(n, current, output),
                _ => Err(ParsingError::UnknownError)
            }
    }
}

fn parse_number_expr<'a>(value: &str, current: usize, output: &'a mut Vec<Expression>) -> IntermediateParsingResult<'a> {
    return match value.parse::<i32>() {
        Err(_) => Err(ParsingError::NumberValueInvalid),
        Ok(value) => {
            output.push(Expression::Value { value: ValueType::Integer(value) });
            return Ok((current + 1, output));
        }
    };
}

mod test {
    use crate::lexer::Token;
    use crate::parser::{Expression, parse, ValueType};

    extern crate speculoos;

    use speculoos::prelude::*;

    #[test]
    fn number_token_parsed_as_value_expr() {
        let result = parse(&mut vec![Token::Number("2".to_string())]);
        let expected_result = vec![Expression::Value { value: ValueType::Integer(2) }];

        assert_that!(result).is_ok_containing(&expected_result);
    }
}