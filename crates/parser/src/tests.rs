use crate::{
    ast::{
        self, AccessAttribute, BinaryExpression, Declaration, Expression, FuncReturnType,
        FunctionCall, FunctionDeclaration, FunctionVisibility, Identifier, IfElse, List, Mapping,
        MappingRelation, Param, Set, Source, StBlock, StateDeclaration, Statement, StatementBlock,
        StructInit, TypeVariant, UnaryExpression, Variable,
    },
    lexer::{Lexer, Token},
    parse,
};

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

fn unwrap_tree(src: &str) -> Result<Source, String> {
    parse(src).map_err(|errs| {
        errs.iter()
            .fold("Errors occurred:".to_string(), |init, count| {
                format!("{}\n{:?}", init, count)
            })
    })
}

const SRC: &str = r#"
@init
@(any)
fn () init(proposal: string,
        start_block: int,
        max_size: int,
        end_block: int)
when () -> BeginState
= move BeginState {
    proposal,
    start_block,
    end_block,
    max_size
};
"#;

#[test]
fn test_simple_func() -> Result<(), String> {
    let tree = unwrap_tree(SRC)?;
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

        let statement = &func_decl.body;
        assert!(
            matches!(statement, Statement::StateTransition(_)),
            "Got {:?}",
            statement
        )
    }
    Ok(())
}

const FACTORIAL_SRC: &str = r#"
state EmptyState;
fn (out: int) calculate(value: int)
st [
    value > 0,
    out < 10000
]
{
    if value == 1 {
        move EmptyState{};
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
st value < 100 = return calculate(value);
"#;

#[test]
fn test_factorial() -> Result<(), String> {
    let tree = unwrap_tree(FACTORIAL_SRC)?;
    assert!(tree.declarations.len() == 3);

    let first_decl = &tree.declarations[0];
    assert!(matches!(first_decl, Declaration::StateDeclaration(_)));
    if let Declaration::StateDeclaration(state) = first_decl {
        assert_eq!(state.name.name, "EmptyState");
        assert_eq!(state.body, None);
        assert_eq!(state.from, None);
        assert_eq!(state.st_block, None);
    }
    Ok(())
}

#[test]
fn test_factorial_tree() -> Result<(), String> {
    let tree = unwrap_tree(FACTORIAL_SRC)?;
    let parsed = Source {
        declarations: vec![
            Declaration::StateDeclaration(Box::new(StateDeclaration {
                loc: 1..18,
                name: Identifier {
                    loc: 7..17,
                    name: "EmptyState".to_string(),
                },
                body: None,
                from: None,
                st_block: None,
            })),
            Declaration::FunDeclaration(Box::new(FunctionDeclaration {
                loc: 19..351,
                is_init: false,
                access_attributes: vec![],
                vis: FunctionVisibility::Priv,
                return_ty: FuncReturnType::ParamType(Param {
                    loc: 23..31,
                    ty: ast::Type {
                        loc: 28..31,
                        ty: TypeVariant::Int,
                    },
                    name: Identifier {
                        loc: 23..26,
                        name: "out".to_string(),
                    },
                    is_mut: true,
                }),
                name: Identifier {
                    loc: 33..42,
                    name: "calculate".to_string(),
                },
                params: vec![Param {
                    loc: 43..53,
                    ty: ast::Type {
                        loc: 50..53,
                        ty: TypeVariant::Int,
                    },
                    name: Identifier {
                        loc: 43..48,
                        name: "value".to_string(),
                    },
                    is_mut: true,
                }],
                state_bound: None,
                st_block: Some(StBlock {
                    loc: 55..92,
                    expr: Expression::List(UnaryExpression::new(
                        58,
                        92,
                        vec![
                            Expression::Greater(BinaryExpression {
                                loc: 64..73,
                                left: Box::new(Expression::Variable(Identifier {
                                    loc: 64..69,
                                    name: "value".to_string(),
                                })),
                                right: Box::new(Expression::Number(UnaryExpression {
                                    loc: 72..73,
                                    element: "0".to_string(),
                                })),
                            }),
                            Expression::Less(BinaryExpression {
                                loc: 79..90,
                                left: Box::new(Expression::Variable(Identifier {
                                    loc: 79..82,
                                    name: "out".to_string(),
                                })),
                                right: Box::new(Expression::Number(UnaryExpression {
                                    loc: 85..90,
                                    element: "10000".to_string(),
                                })),
                            }),
                        ],
                    )),
                }),
                body: Statement::Block(StatementBlock {
                    loc: 93..351,
                    statements: vec![Statement::IfElse(IfElse {
                        loc: 99..349,
                        condition: Expression::Equal(BinaryExpression {
                            loc: 102..112,
                            left: Box::new(Expression::Variable(Identifier {
                                loc: 102..107,
                                name: "value".to_string(),
                            })),
                            right: Box::new(Expression::Number(UnaryExpression {
                                loc: 111..112,
                                element: "1".to_string(),
                            })),
                        }),
                        body: Box::new(StatementBlock {
                            loc: 113..169,
                            statements: vec![
                                Statement::StateTransition(StructInit {
                                    loc: 128..140,
                                    name: Identifier {
                                        loc: 128..138,
                                        name: "EmptyState".to_string(),
                                    },
                                    args: vec![],
                                    auto_object: None,
                                }),
                                Statement::Return(Expression::Variable(Identifier {
                                    loc: 157..162,
                                    name: "value".to_string(),
                                })),
                            ],
                        }),
                        else_part: Some(Box::new(Statement::Block(StatementBlock {
                            loc: 175..349,
                            statements: vec![Statement::Return(Expression::FunctionCall(
                                FunctionCall {
                                    loc: 192..342,
                                    name: Identifier {
                                        loc: 192..201,
                                        name: "calculate".to_string(),
                                    },
                                    args: vec![Expression::Pipe(BinaryExpression {
                                        loc: 296..324,
                                        left: Box::new(Expression::Multiply(BinaryExpression {
                                            loc: 296..315,
                                            left: Box::new(Expression::Variable(Identifier {
                                                loc: 296..301,
                                                name: "value".to_string(),
                                            })),
                                            right: Box::new(Expression::Subtract(
                                                BinaryExpression {
                                                    loc: 305..314,
                                                    left: Box::new(Expression::Variable(
                                                        Identifier {
                                                            loc: 305..310,
                                                            name: "value".to_string(),
                                                        },
                                                    )),
                                                    right: Box::new(Expression::Number(
                                                        UnaryExpression {
                                                            loc: 313..314,
                                                            element: "1".to_string(),
                                                        },
                                                    )),
                                                },
                                            )),
                                        })),
                                        right: Box::new(Expression::FunctionCall(FunctionCall {
                                            loc: 319..324,
                                            name: Identifier {
                                                loc: 319..321,
                                                name: "or".to_string(),
                                            },
                                            args: vec![Expression::Number(UnaryExpression {
                                                loc: 322..323,
                                                element: "1".to_string(),
                                            })],
                                        })),
                                    })],
                                },
                            ))],
                        }))),
                    })],
                }),
            })),
            Declaration::FunDeclaration(Box::new(FunctionDeclaration {
                loc: 353..434,
                is_init: false,
                access_attributes: vec![AccessAttribute {
                    loc: 353..359,
                    members: vec![Expression::Variable(Identifier {
                        loc: 355..358,
                        name: "any".to_string(),
                    })],
                }],
                vis: FunctionVisibility::Pub,
                return_ty: FuncReturnType::Type(ast::Type {
                    loc: 363..366,
                    ty: TypeVariant::Int,
                }),
                name: Identifier {
                    loc: 367..380,
                    name: "get_factorial".to_string(),
                },
                params: vec![Param {
                    loc: 381..391,
                    ty: ast::Type {
                        loc: 388..391,
                        ty: TypeVariant::Int,
                    },
                    name: Identifier {
                        loc: 381..386,
                        name: "value".to_string(),
                    },
                    is_mut: true,
                }],
                state_bound: None,
                st_block: Some(StBlock {
                    loc: 393..407,
                    expr: Expression::Less(BinaryExpression {
                        loc: 396..407,
                        left: Box::new(Expression::Variable(Identifier {
                            loc: 396..401,
                            name: "value".to_string(),
                        })),
                        right: Box::new(Expression::Number(UnaryExpression {
                            loc: 404..407,
                            element: "100".to_string(),
                        })),
                    }),
                }),
                body: Statement::Return(Expression::FunctionCall(FunctionCall {
                    loc: 417..433,
                    name: Identifier {
                        loc: 417..426,
                        name: "calculate".to_string(),
                    },
                    args: vec![Expression::Variable(Identifier {
                        loc: 427..432,
                        name: "value".to_string(),
                    })],
                })),
            })),
        ],
    };
    assert_eq!(tree, parsed, "Invalid tree: {:#?}", parsed);
    Ok(())
}

const LISTS_SRC: &str = r#"
fn () lists() {
    let mut ls : list<int> = [1, 2, 3];
    let mut ss : set<int> = [1, 2, 3];
    let mut mm : mapping<int -/> string> = init();
}
"#;

#[test]
fn test_lists() -> Result<(), String> {
    let tree = unwrap_tree(LISTS_SRC)?;
    let parsed = Source {
        declarations: vec![Declaration::FunDeclaration(Box::new(FunctionDeclaration {
            loc: 1..148,
            is_init: false,
            access_attributes: vec![],
            vis: FunctionVisibility::Priv,
            return_ty: FuncReturnType::Type(ast::Type {
                loc: 4..6,
                ty: TypeVariant::Unit,
            }),
            name: Identifier {
                loc: 7..12,
                name: "lists".to_string(),
            },
            params: vec![],
            state_bound: None,
            st_block: None,
            body: Statement::Block(StatementBlock {
                loc: 15..148,
                statements: vec![
                    Statement::Variable(Variable {
                        loc: 21..55,
                        names: vec![Identifier {
                            loc: 29..31,
                            name: "ls".to_string(),
                        }],
                        mutable: true,
                        ty: Some(ast::Type {
                            loc: 34..43,
                            ty: TypeVariant::List(List {
                                ty: Box::new(ast::Type {
                                    loc: 39..42,
                                    ty: TypeVariant::Int,
                                }),
                            }),
                        }),
                        value: Some(Expression::List(UnaryExpression {
                            loc: 46..55,
                            element: vec![
                                Expression::Number(UnaryExpression {
                                    loc: 47..48,
                                    element: "1".to_string(),
                                }),
                                Expression::Number(UnaryExpression {
                                    loc: 50..51,
                                    element: "2".to_string(),
                                }),
                                Expression::Number(UnaryExpression {
                                    loc: 53..54,
                                    element: "3".to_string(),
                                }),
                            ],
                        })),
                    }),
                    Statement::Variable(Variable {
                        loc: 61..94,
                        names: vec![Identifier {
                            loc: 69..71,
                            name: "ss".to_string(),
                        }],
                        mutable: true,
                        ty: Some(ast::Type {
                            loc: 74..82,
                            ty: TypeVariant::Set(Set {
                                ty: Box::new(ast::Type {
                                    loc: 78..81,
                                    ty: TypeVariant::Int,
                                }),
                            }),
                        }),
                        value: Some(Expression::List(UnaryExpression {
                            loc: 85..94,
                            element: vec![
                                Expression::Number(UnaryExpression {
                                    loc: 86..87,
                                    element: "1".to_string(),
                                }),
                                Expression::Number(UnaryExpression {
                                    loc: 89..90,
                                    element: "2".to_string(),
                                }),
                                Expression::Number(UnaryExpression {
                                    loc: 92..93,
                                    element: "3".to_string(),
                                }),
                            ],
                        })),
                    }),
                    Statement::Variable(Variable {
                        loc: 100..145,
                        names: vec![Identifier {
                            loc: 108..110,
                            name: "mm".to_string(),
                        }],
                        mutable: true,
                        ty: Some(ast::Type {
                            loc: 113..136,
                            ty: TypeVariant::Mapping(Mapping {
                                from_ty: Box::new(ast::Type {
                                    loc: 121..124,
                                    ty: TypeVariant::Int,
                                }),
                                relation: MappingRelation {
                                    loc: 125..128,
                                    injective: false,
                                    partial: true,
                                    surjective: false,
                                },
                                to_ty: Box::new(ast::Type {
                                    loc: 129..135,
                                    ty: TypeVariant::String,
                                }),
                            }),
                        }),
                        value: Some(Expression::FunctionCall(FunctionCall {
                            loc: 139..145,
                            name: Identifier {
                                loc: 139..143,
                                name: "init".to_string(),
                            },
                            args: vec![],
                        })),
                    }),
                ],
            }),
        }))],
    };
    assert_eq!(tree, parsed, "Invalid tree: {:#?}", parsed);
    Ok(())
}

const STRUCTS_SRC: &str = r#"
fn () structs() {
    let obj = MyStruct { 2, 3 };
    let { one, reset } = MyStruct { ..obj };
}
"#;
