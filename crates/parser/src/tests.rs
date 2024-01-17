use crate::lexer::{Lexer, Token};

#[test]
fn simple_int() {
    let input = "123";
    let mut errors = Vec::new();
    let mut tokens = Lexer::new(input, &mut errors);
    assert_eq!(tokens.next(), Some((0, Token::Integer(123), 3)))
}

#[test]
fn simple_floats() {
    let input = ".123 1.23";
    let mut errors = Vec::new();
    let mut tokens = Lexer::new(input, &mut errors);
    assert_eq!(tokens.next(), Some((0, Token::Float(0.123), 4)));
    assert_eq!(tokens.next(), Some((5, Token::Float(1.23), 9)))
}

#[test]
fn simple_mixed_numbers() {
    let input = "1.23 456";
    let mut errors = Vec::new();
    let mut tokens = Lexer::new(input, &mut errors);
    assert_eq!(tokens.next(), Some((0, Token::Float(1.23), 4)));
    assert_eq!(tokens.next(), Some((5, Token::Integer(456), 8)))
}