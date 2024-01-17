use crate::folidity;
use crate::lexer::{Lexer, Token};

#[test]
fn simple_int() {
    let input = "123";
    let mut errors = Vec::new();
    let mut tokens = Lexer::new(input, &mut errors);
    assert_eq!(tokens.next(), Some((0, Token::Number("123"), 3)))
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
    assert_eq!(tokens.next(), Some((5, Token::Number("456"), 8)))
}

#[test]
fn comment_token() {
    let input = "# hey\nident";
    let mut errors = Vec::new();
    let mut tokens = Lexer::new(input, &mut errors);
    assert_eq!(tokens.next(), Some((6, Token::Identifier("ident"), 11)))
}

#[test]
fn simple_maths_tree() {
    let input = "123 + 4 + (1 - 2) * 2 / 12\n#hey";
    let mut lexer_errors = Vec::new();
    let tokens = Lexer::new(input, &mut lexer_errors);
    let parser = folidity::ExpressionParser::new();
    let mut parser_errors = Vec::new();
    let tree = parser.parse(&mut parser_errors, tokens).unwrap();
    let output = r#"Add(
    BinaryExpression {
        loc: 0..26,
        left: Add(
            BinaryExpression {
                loc: 0..7,
                left: Number(
                    UnaryExpression {
                        loc: 0..3,
                        element: "123",
                    },
                ),
                right: Number(
                    UnaryExpression {
                        loc: 6..7,
                        element: "4",
                    },
                ),
            },
        ),
        right: Divide(
            BinaryExpression {
                loc: 10..26,
                left: Multiply(
                    BinaryExpression {
                        loc: 10..21,
                        left: Subtract(
                            BinaryExpression {
                                loc: 11..16,
                                left: Number(
                                    UnaryExpression {
                                        loc: 11..12,
                                        element: "1",
                                    },
                                ),
                                right: Number(
                                    UnaryExpression {
                                        loc: 15..16,
                                        element: "2",
                                    },
                                ),
                            },
                        ),
                        right: Number(
                            UnaryExpression {
                                loc: 20..21,
                                element: "2",
                            },
                        ),
                    },
                ),
                right: Number(
                    UnaryExpression {
                        loc: 24..26,
                        element: "12",
                    },
                ),
            },
        ),
    },
)"#;
    assert_eq!(output, format!("{:#?}", tree))
}
