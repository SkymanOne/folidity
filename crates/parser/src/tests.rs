use crate::{
    ast::{
        self,
        AccessAttribute,
        BinaryExpression,
        Declaration,
        EnumDeclaration,
        Expression,
        FuncReturnType,
        FunctionCall,
        FunctionDeclaration,
        FunctionVisibility,
        Identifier,
        IfElse,
        List,
        Mapping,
        MappingRelation,
        MemberAccess,
        ModelDeclaration,
        Param,
        Set,
        Source,
        StBlock,
        StateDeclaration,
        Statement,
        StatementBlock,
        StructDeclaration,
        StructInit,
        TypeVariant,
        UnaryExpression,
        Variable,
    },
    lexer::{
        Lexer,
        Token,
    },
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
                format!("{}\n{:#?}", init, count)
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
= move BeginState : {
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
state EmptyState 
fn (out: int) calculate(value: int)
st [
    value > 0,
    out < 10000
]
{
    if value == 1 {
        move EmptyState:{};
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
                loc: 1..17,
                name: Identifier {
                    loc: 7..17,
                    name: "EmptyState".to_string(),
                },
                body: None,
                from: None,
                st_block: None,
            })),
            Declaration::FunDeclaration(Box::new(FunctionDeclaration {
                loc: 19..352,
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
                    is_mut: false,
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
                    loc: 93..352,
                    statements: vec![Statement::IfElse(IfElse {
                        loc: 99..350,
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
                            loc: 113..170,
                            statements: vec![
                                Statement::StateTransition(StructInit {
                                    loc: 128..141,
                                    name: Identifier {
                                        loc: 128..138,
                                        name: "EmptyState".to_string(),
                                    },
                                    args: vec![],
                                    auto_object: None,
                                }),
                                Statement::Return(Expression::Variable(Identifier {
                                    loc: 158..163,
                                    name: "value".to_string(),
                                })),
                            ],
                        }),
                        else_part: Some(Box::new(Statement::Block(StatementBlock {
                            loc: 176..350,
                            statements: vec![Statement::Return(Expression::FunctionCall(
                                FunctionCall {
                                    loc: 193..343,
                                    name: Identifier {
                                        loc: 193..202,
                                        name: "calculate".to_string(),
                                    },
                                    args: vec![Expression::Pipe(BinaryExpression {
                                        loc: 297..325,
                                        left: Box::new(Expression::Multiply(BinaryExpression {
                                            loc: 297..316,
                                            left: Box::new(Expression::Variable(Identifier {
                                                loc: 297..302,
                                                name: "value".to_string(),
                                            })),
                                            right: Box::new(Expression::Subtract(
                                                BinaryExpression {
                                                    loc: 306..315,
                                                    left: Box::new(Expression::Variable(
                                                        Identifier {
                                                            loc: 306..311,
                                                            name: "value".to_string(),
                                                        },
                                                    )),
                                                    right: Box::new(Expression::Number(
                                                        UnaryExpression {
                                                            loc: 314..315,
                                                            element: "1".to_string(),
                                                        },
                                                    )),
                                                },
                                            )),
                                        })),
                                        right: Box::new(Expression::FunctionCall(FunctionCall {
                                            loc: 320..325,
                                            name: Identifier {
                                                loc: 320..322,
                                                name: "or".to_string(),
                                            },
                                            args: vec![Expression::Number(UnaryExpression {
                                                loc: 323..324,
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
                loc: 354..435,
                is_init: false,
                access_attributes: vec![AccessAttribute {
                    loc: 354..360,
                    members: vec![Expression::Variable(Identifier {
                        loc: 356..359,
                        name: "any".to_string(),
                    })],
                }],
                vis: FunctionVisibility::Pub,
                return_ty: FuncReturnType::Type(ast::Type {
                    loc: 364..367,
                    ty: TypeVariant::Int,
                }),
                name: Identifier {
                    loc: 368..381,
                    name: "get_factorial".to_string(),
                },
                params: vec![Param {
                    loc: 382..392,
                    ty: ast::Type {
                        loc: 389..392,
                        ty: TypeVariant::Int,
                    },
                    name: Identifier {
                        loc: 382..387,
                        name: "value".to_string(),
                    },
                    is_mut: false,
                }],
                state_bound: None,
                st_block: Some(StBlock {
                    loc: 394..408,
                    expr: Expression::Less(BinaryExpression {
                        loc: 397..408,
                        left: Box::new(Expression::Variable(Identifier {
                            loc: 397..402,
                            name: "value".to_string(),
                        })),
                        right: Box::new(Expression::Number(UnaryExpression {
                            loc: 405..408,
                            element: "100".to_string(),
                        })),
                    }),
                }),
                body: Statement::Return(Expression::FunctionCall(FunctionCall {
                    loc: 418..434,
                    name: Identifier {
                        loc: 418..427,
                        name: "calculate".to_string(),
                    },
                    args: vec![Expression::Variable(Identifier {
                        loc: 428..433,
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
struct MyStruct {
    a: int,
    b: address
}

enum MyEnum {
    A, 
    B
}

fn () structs() {
    let obj = MyStruct : { 2, 3 };
    let { one, reset } = MyStruct : { ..obj };
    let a_enum = MyEnum.A;
}

model MyModel: ParentModel {
}
"#;

#[test]
fn test_structs_enums() -> Result<(), String> {
    let parsed = unwrap_tree(STRUCTS_SRC)?;

    let tree = Source {
        declarations: vec![
            Declaration::StructDeclaration(Box::new(StructDeclaration {
                loc: 1..47,
                name: Identifier {
                    loc: 8..16,
                    name: "MyStruct".to_string(),
                },
                fields: vec![
                    Param {
                        loc: 23..29,
                        ty: ast::Type {
                            loc: 26..29,
                            ty: TypeVariant::Int,
                        },
                        name: Identifier {
                            loc: 23..24,
                            name: "a".to_string(),
                        },
                        is_mut: true,
                    },
                    Param {
                        loc: 35..45,
                        ty: ast::Type {
                            loc: 38..45,
                            ty: TypeVariant::Address,
                        },
                        name: Identifier {
                            loc: 35..36,
                            name: "b".to_string(),
                        },
                        is_mut: true,
                    },
                ],
            })),
            Declaration::EnumDeclaration(Box::new(EnumDeclaration {
                loc: 49..78,
                name: Identifier {
                    loc: 54..60,
                    name: "MyEnum".to_string(),
                },
                variants: vec![
                    Identifier {
                        loc: 67..68,
                        name: "A".to_string(),
                    },
                    Identifier {
                        loc: 75..76,
                        name: "B".to_string(),
                    },
                ],
            })),
            Declaration::FunDeclaration(Box::new(FunctionDeclaration {
                loc: 80..208,
                is_init: false,
                access_attributes: vec![],
                vis: FunctionVisibility::Priv,
                return_ty: FuncReturnType::Type(ast::Type {
                    loc: 83..85,
                    ty: TypeVariant::Unit,
                }),
                name: Identifier {
                    loc: 86..93,
                    name: "structs".to_string(),
                },
                params: vec![],
                state_bound: None,
                st_block: None,
                body: Statement::Block(StatementBlock {
                    loc: 96..208,
                    statements: vec![
                        Statement::Variable(Variable {
                            loc: 102..131,
                            names: vec![Identifier {
                                loc: 106..109,
                                name: "obj".to_string(),
                            }],
                            mutable: false,
                            ty: None,
                            value: Some(Expression::StructInit(UnaryExpression {
                                loc: 112..131,
                                element: StructInit {
                                    loc: 112..131,
                                    name: Identifier {
                                        loc: 112..120,
                                        name: "MyStruct".to_string(),
                                    },
                                    args: vec![
                                        Expression::Number(UnaryExpression {
                                            loc: 125..126,
                                            element: "2".to_string(),
                                        }),
                                        Expression::Number(UnaryExpression {
                                            loc: 128..129,
                                            element: "3".to_string(),
                                        }),
                                    ],
                                    auto_object: None,
                                },
                            })),
                        }),
                        Statement::Variable(Variable {
                            loc: 137..178,
                            names: vec![
                                Identifier {
                                    loc: 143..146,
                                    name: "one".to_string(),
                                },
                                Identifier {
                                    loc: 148..153,
                                    name: "reset".to_string(),
                                },
                            ],
                            mutable: false,
                            ty: None,
                            value: Some(Expression::StructInit(UnaryExpression {
                                loc: 158..178,
                                element: StructInit {
                                    loc: 158..178,
                                    name: Identifier {
                                        loc: 158..166,
                                        name: "MyStruct".to_string(),
                                    },
                                    args: vec![],
                                    auto_object: Some(Identifier {
                                        loc: 173..176,
                                        name: "obj".to_string(),
                                    }),
                                },
                            })),
                        }),
                        Statement::Variable(Variable {
                            loc: 184..205,
                            names: vec![Identifier {
                                loc: 188..194,
                                name: "a_enum".to_string(),
                            }],
                            mutable: false,
                            ty: None,
                            value: Some(Expression::MemberAccess(MemberAccess {
                                loc: 197..205,
                                expr: Box::new(Expression::Variable(Identifier {
                                    loc: 197..203,
                                    name: "MyEnum".to_string(),
                                })),
                                member: Identifier {
                                    loc: 204..205,
                                    name: "A".to_string(),
                                },
                            })),
                        }),
                    ],
                }),
            })),
            Declaration::ModelDeclaration(Box::new(ModelDeclaration {
                loc: 210..240,
                name: Identifier {
                    loc: 216..223,
                    name: "MyModel".to_string(),
                },
                fields: vec![],
                parent: Some(Identifier {
                    loc: 225..236,
                    name: "ParentModel".to_string(),
                }),
                st_block: None,
            })),
        ],
    };

    assert_eq!(tree, parsed, "Invalid tree: {:#?}", parsed);
    Ok(())
}

const COMPLETE_SRC: &str = r#"
# This is a comment
enum Choice {
    None,
    Yay,
    Nay
}

# This is 
# a multiline comment


model BeginModel {
    start_block: int,
    end_block: int,
    voters: set<Address>,
    proposal: String,
    max_size: int,
} st [
    start_block > (current_block + 10),
    end_block > (start_block + 10),
    voters.balance > 1000,
    max_size > 0,
    voter.size <= max_size
]

model VotingModel: BeginModel {
    commits: mapping<address >-/> hex>
} st [
    commits.key in voters
]

model RevealModel {
    proposal: string,
    end_block: int,
    commit: mapping<hex -> Choice>
} st [
    end_block > (current_block + 15),
    yays >= 0,
    nays >= 0,
    (yays + nays) <= commits.size
]

model ExecuteModel {
    proposal: string,
    passed: bool
}

state BeginState(BeginModel)
state VotingState(VotingModel)

state RevealState(RevealModel) from (VotingState vst)
st current_block > vst.end_block


state ExecuteState(ExecuteModel) from RevealState st [
    current_block > RevealModel.end_block 
]

state ExecuteState {
    proposal: string,
    passed: bool
} from (RevealState rst) st [
    current_block > rst.end_block 
]

@init
@(any)
fn () init(proposal: string, 
          start_block: int, 
          max_size: int, 
          end_block: int) 
when () -> BeginState
{
    move BeginState : {
        proposal,
        start_block,
        end_block,
        max_size
    };
}

@(any)
fn () join() when (BeginState s) -> BeginState {
    let caller = caller();
    let { voters, params } = s;
    voters = voters + caller;
    move BeginState : {
        voters
        | ..params
    };
}

@(voters)
fn () start_voting() when (BeginState s) -> VotingState {
    let commits = Set();
    move VotingState : {
        commits
        | ..s 
    };
}

@(voters)
fn () commit(h: hex) when (VotingState s) -> VotingState {
    let caller = caller();
    let { commits, params } = s;

    commits = commits :> add(caller, h);


    move VotingState : {
        commits
        | ..params
    };
}


@(any)
fn () start_reveal() when (VotingState s) -> RevealState {
    let { end_block, proposal, commits, params } = s;
    move RevealState : {
        endblock + 10,
        proposal,
        # we need to add lambda to grammar later
        commits :> map(map_lambda),
        0,
        0
    };
}

fn (out: int) map_lambda(item: int)
st out < 1000 {
    if item == 1 {
        return item;
    } else if item > 1000 {
        return 1000;
    } else {
        return item;
    }
}

@(any)
fn int reveal(salt: int, vote: Choice) 
when (RevealState s1) -> (RevealState s2), (ExecuteState s3)
st s1.commits.size == s2.commits.size
{
    let calc_hash = hash(caller(), vote, salt);
    let { commits, params } = s1;

    commits = commits :> add(calc_hash, vote);

    if s1.current_block > s1.end_block {
        execute();
    } else {
        move RevealState : {
            commits
            | ..params
        }; 
        return commits.size;
    }
}

@(any)
fn () execute() when (RevealState s) -> ExecuteState {
    let votes = s.commits.values;
    # add lambda later
    # let yay = votes :> filter(|v| v == Choice::Yay).sum();
    let mut passed = false;
    [1, 2, 3] :> func1 :> func2 :> func3;
    func3();
    if votes.size / yay > 0.5 {
        passed = true;
        move ExecuteState : {
            ss.proposal,
            passed
        };
    } else {
        move ExecuteState : {
            s.proposal,
            passed
        };
    }
}

view(BeginState s) fn list<Address> get_voters() {
    return s.voters;
}
"#;

#[test]
fn parse_complete_program() {
    let res = parse(COMPLETE_SRC);
    match res {
        Ok(_) => {}
        Err(errs) => {
            panic!("{:#?}", errs)
        }
    }
}
