use crate::ast::{Declaration, FuncReturnType, Statement, TypeVariant};
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
    assert_eq!(tokens.next(), Some((0, Token::Float(".123"), 4)));
    assert_eq!(tokens.next(), Some((5, Token::Float("1.23"), 9)))
}

#[test]
fn simple_mixed_numbers() {
    let input = "1.23 456";
    let mut errors = Vec::new();
    let mut tokens = Lexer::new(input, &mut errors);
    assert_eq!(tokens.next(), Some((0, Token::Float("1.23"), 4)));
    assert_eq!(tokens.next(), Some((5, Token::Number("456"), 8)))
}

#[test]
fn comment_token() {
    let input = "# hey\nident";
    let mut errors = Vec::new();
    let mut tokens = Lexer::new(input, &mut errors);
    assert_eq!(tokens.next(), Some((6, Token::Identifier("ident"), 11)))
}

const SRC: &str = r#"
@init
@(any)
fn () init(proposal: string, 
        start_block: int, 
        max_size: int, 
        end_block: int) 
when () -> BeginState
{
    move BeginState {
        proposal,
        start_block,
        end_block,
        max_size
    };
}
"#;

#[test]
fn test_simple_func() {
    let mut lexer_errors = Vec::new();
    let tokens = Lexer::new(SRC, &mut lexer_errors);
    let mut parser_errors = Vec::new();
    let tree = folidity::FolidityTreeParser::new()
        .parse(&mut parser_errors, tokens)
        .unwrap();
    assert!(tree.declarations.len() == 1);
    let func = &tree.declarations[0];
    assert!(matches!(func, &Declaration::FunDeclaration(_)));

    if let Declaration::FunDeclaration(func_decl) = func {
        assert!(func_decl.is_init);
        assert_eq!(func_decl.access_attributes[0].members.len(), 1);
        assert_eq!(func_decl.params.len(), 4);
        assert_eq!(func_decl.name.name, String::from("init"));
        assert!(matches!(func_decl.return_ty, FuncReturnType::Type(_)));

        if let FuncReturnType::Type(ty) = &func_decl.return_ty {
            assert!(matches!(&ty.ty, TypeVariant::Unit))
        }

        assert_eq!(func_decl.body.statements.len(), 1);
        let statement = &func_decl.body.statements[0];
        assert!(matches!(statement, Statement::StateTransition(_)))
    }
}

const FACTORIAL_SRC: &str = r#"
state NoState;
fn (out: int) calculate(value: int)
st {
    value > 0,
    out < 10000
}
{
    if value == 1 {
        move SimpleState{};
        return value;
    } else {
        return calculate(
                # `:> or(int)` specify what happens when operation fails
                    value * (value - 1) :> or(1)
                );
    }
}

@(any)
fn int get_factorial(value: int)
st value < 100
{
    return calculate(value);
}
"#;

#[test]
fn test_factorial() {
    let mut lexer_errors = Vec::new();
    let tokens = Lexer::new(FACTORIAL_SRC, &mut lexer_errors);
    let mut parser_errors = Vec::new();
    let tree = folidity::FolidityTreeParser::new()
        .parse(&mut parser_errors, tokens)
        .unwrap();
    assert!(tree.declarations.len() == 3);

    let first_decl = &tree.declarations[0];
    assert!(matches!(first_decl, Declaration::StateDeclaration(_)));
    if let Declaration::StateDeclaration(state) = first_decl {
        assert_eq!(state.name.name, "NoState");
        assert_eq!(state.body, None);
        assert_eq!(state.from, None);
        assert_eq!(state.st_block, None);
    }
}
