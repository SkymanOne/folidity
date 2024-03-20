use folidity_parser::{
    ast::{
        self as parsed_ast,
        Identifier,
    },
    Span,
};
use indexmap::IndexMap;

use crate::{
    ast::{
        Expression,
        FuncReturnType,
        Function,
        FunctionVisibility,
        Param,
        StructDeclaration,
        Type,
        TypeVariant,
    },
    contract::ContractDefinition,
    global_symbol::{
        GlobalSymbol,
        SymbolInfo,
    },
    symtable::{
        Scope,
        VariableKind,
    },
    types::ExpectedType,
};

use super::expression;

#[test]
fn test_list() {
    let loc = Span { start: 0, end: 0 };
    let mut contract = ContractDefinition::default();
    let mut scope = Scope::default();
    let parsed_list = parsed_ast::Expression::List(parsed_ast::UnaryExpression {
        loc: loc.clone(),
        element: vec![
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "1".to_string(),
            }),
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "2".to_string(),
            }),
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "3".to_string(),
            }),
        ],
    });
    let resolved_expr = expression(
        &parsed_list,
        ExpectedType::Concrete(TypeVariant::List(Box::new(TypeVariant::Int))),
        &mut scope,
        &mut contract,
    );
    assert!(resolved_expr.is_ok());

    let resolved_expr_err = expression(
        &parsed_list,
        ExpectedType::Concrete(TypeVariant::List(Box::new(TypeVariant::Float))),
        &mut scope,
        &mut contract,
    );
    assert!(resolved_expr_err.is_err());

    let resolved_expr = expression(
        &parsed_list,
        ExpectedType::Dynamic(vec![]),
        &mut scope,
        &mut contract,
    );

    assert!(resolved_expr.is_ok(), "Errors: {:#?}", contract.diagnostics);
    let expr = resolved_expr.unwrap();
    if let Expression::List(e) = expr {
        assert_eq!(e.ty, TypeVariant::List(Box::new(TypeVariant::Int)));
        assert_eq!(e.element.len(), 3)
    }
}

#[test]
fn test_var() {
    let loc = Span { start: 0, end: 0 };
    let mut contract = ContractDefinition::default();
    let mut scope = Scope::default();

    let ident = Identifier {
        loc: loc.clone(),
        name: String::from("my_var"),
    };

    scope.tables[scope.current].add(
        &mut contract,
        &ident,
        TypeVariant::Int,
        None,
        VariableKind::Local,
        true,
    );

    let parsed_var = parsed_ast::Expression::Variable(ident.clone());
    let resolved_expr = expression(
        &parsed_var,
        ExpectedType::Concrete(TypeVariant::Int),
        &mut scope,
        &mut contract,
    );

    assert!(resolved_expr.is_ok(), "Errors: {:#?}", contract.diagnostics);
    let resolved_expr = resolved_expr.unwrap();
    if let Expression::Variable(var) = resolved_expr {
        assert_eq!(var.element, 0);
        assert_eq!(var.ty, TypeVariant::Int);
        let sym = scope.find_symbol(&var.element).unwrap();
        assert!(!sym.assigned());
        assert_eq!(&sym.ident, &ident);
        assert_eq!(&sym.ty, &TypeVariant::Int);
    }
}

#[test]
fn test_mul() {
    let loc = Span { start: 0, end: 0 };
    let mut contract = ContractDefinition::default();
    let mut scope = Scope::default();
    let nums = vec![
        parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
            loc: loc.clone(),
            element: "1".to_string(),
        }),
        parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
            loc: loc.clone(),
            element: "2".to_string(),
        }),
        parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
            loc: loc.clone(),
            element: "3".to_string(),
        }),
    ];

    let mul_expr = parsed_ast::Expression::Multiply(parsed_ast::BinaryExpression {
        loc: loc.clone(),
        left: Box::new(nums[0].clone()),
        right: Box::new(nums[1].clone()),
    });

    let resolved_expr = expression(
        &mul_expr,
        ExpectedType::Dynamic(vec![]),
        &mut scope,
        &mut contract,
    );

    assert!(resolved_expr.is_ok(), "Errors: {:#?}", contract.diagnostics);

    let resolved_expr = resolved_expr.unwrap();

    if let Expression::Multiply(mul) = resolved_expr {
        assert_eq!(mul.ty, TypeVariant::Int);
        assert!(mul.left.is_literal() && mul.right.is_literal());
    }
}

#[test]
fn test_eval() {
    let loc = Span { start: 0, end: 0 };
    let mut contract = ContractDefinition::default();
    let mut scope = Scope::default();
    let nums = vec![
        parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
            loc: loc.clone(),
            element: "4".to_string(),
        }),
        parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
            loc: loc.clone(),
            element: "10".to_string(),
        }),
    ];

    let mul_expr = parsed_ast::Expression::Multiply(parsed_ast::BinaryExpression {
        loc: loc.clone(),
        left: Box::new(nums[0].clone()),
        right: Box::new(nums[1].clone()),
    });

    let resolved_expr = expression(
        &mul_expr,
        ExpectedType::Dynamic(vec![]),
        &mut scope,
        &mut contract,
    );

    assert!(resolved_expr.is_ok(), "Errors: {:#?}", contract.diagnostics);

    let resolved_expr = resolved_expr.unwrap();
    assert!(matches!(resolved_expr, Expression::Int(_)));

    if let Expression::Int(u) = resolved_expr {
        assert_eq!(u.element, 40.into());
    }
}

#[test]
fn test_func() {
    let loc = Span { start: 0, end: 0 };
    let mut contract = ContractDefinition::default();
    let mut scope = Scope::default();

    let mut params = IndexMap::new();
    params.insert(
        "a".to_string(),
        Param {
            loc: loc.clone(),
            ty: Type {
                loc: loc.clone(),
                ty: TypeVariant::Int,
            },
            name: Identifier {
                loc: loc.clone(),
                name: "a".to_string(),
            },
            is_mut: true,
            recursive: false,
        },
    );

    params.insert(
        "b".to_string(),
        Param {
            loc: loc.clone(),
            ty: Type {
                loc: loc.clone(),
                ty: TypeVariant::String,
            },
            name: Identifier {
                loc: loc.clone(),
                name: "b".to_string(),
            },
            is_mut: true,
            recursive: false,
        },
    );

    params.insert(
        "c".to_string(),
        Param {
            loc: loc.clone(),
            ty: Type {
                loc: loc.clone(),
                ty: TypeVariant::List(Box::new(TypeVariant::Generic(vec![
                    TypeVariant::Int,
                    TypeVariant::Uint,
                    TypeVariant::Float,
                    TypeVariant::String,
                ]))),
            },
            name: Identifier {
                loc: loc.clone(),
                name: "c".to_string(),
            },
            is_mut: true,
            recursive: false,
        },
    );

    let func_ident = Identifier {
        loc: loc.clone(),
        name: "my_func".to_string(),
    };
    contract.functions.push(Function::new(
        loc.clone(),
        false,
        FunctionVisibility::Priv,
        FuncReturnType::Type(Type {
            loc: loc.clone(),
            ty: TypeVariant::List(Box::new(TypeVariant::Generic(vec![
                TypeVariant::Int,
                TypeVariant::String,
            ]))),
        }),
        func_ident.clone(),
        params,
        None,
    ));

    contract.add_global_symbol(
        &func_ident,
        GlobalSymbol::Function(SymbolInfo {
            loc: loc.clone(),
            i: 0,
        }),
    );

    let number = parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
        loc: loc.clone(),
        element: "1".to_string(),
    });

    let string = parsed_ast::Expression::String(parsed_ast::UnaryExpression {
        loc: loc.clone(),
        element: "Hello World".to_string(),
    });

    let list = parsed_ast::Expression::List(parsed_ast::UnaryExpression {
        loc: loc.clone(),
        element: vec![
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "1".to_string(),
            }),
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "2".to_string(),
            }),
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "3".to_string(),
            }),
        ],
    });

    let parsed_call = parsed_ast::Expression::FunctionCall(parsed_ast::FunctionCall {
        loc: loc.clone(),
        name: func_ident.clone(),
        args: vec![number.clone(), string.clone(), list.clone()],
    });

    let resolved_expr = expression(
        &parsed_call,
        ExpectedType::Concrete(TypeVariant::List(Box::new(TypeVariant::Int))),
        &mut scope,
        &mut contract,
    );

    assert!(resolved_expr.is_ok(), "Errors: {:#?}", contract.diagnostics);

    let Expression::FunctionCall(func_call) = resolved_expr.unwrap() else {
        panic!("Expected function resolved");
    };

    let a = expression(
        &number,
        ExpectedType::Concrete(TypeVariant::Int),
        &mut scope,
        &mut contract,
    )
    .unwrap();
    let b = expression(
        &string,
        ExpectedType::Concrete(TypeVariant::String),
        &mut scope,
        &mut contract,
    )
    .unwrap();
    let c = expression(
        &list,
        ExpectedType::Concrete(TypeVariant::List(Box::new(TypeVariant::Int))),
        &mut scope,
        &mut contract,
    )
    .unwrap();

    assert_eq!(func_call.name, func_ident);
    assert_eq!(
        func_call.returns,
        TypeVariant::List(Box::new(TypeVariant::Int))
    );
    assert_eq!(func_call.args, vec![a, b, c]);
}

#[test]
fn member_access() {
    let loc = Span { start: 0, end: 0 };
    let mut contract = ContractDefinition::default();
    let mut scope = Scope::default();

    let a = Param {
        loc: loc.clone(),
        ty: Type {
            loc: loc.clone(),
            ty: TypeVariant::String,
        },
        name: Identifier {
            loc: loc.clone(),
            name: "a".to_string(),
        },
        is_mut: false,
        recursive: false,
    };
    let b = Param {
        loc: loc.clone(),
        ty: Type {
            loc: loc.clone(),
            ty: TypeVariant::Int,
        },
        name: Identifier {
            loc: loc.clone(),
            name: "b".to_string(),
        },
        is_mut: false,
        recursive: false,
    };
    let c = Param {
        loc: loc.clone(),
        ty: Type {
            loc: loc.clone(),
            ty: TypeVariant::Address,
        },
        name: Identifier {
            loc: loc.clone(),
            name: "c".to_string(),
        },
        is_mut: false,
        recursive: false,
    };
    contract.structs.push(StructDeclaration {
        loc: loc.clone(),
        name: Identifier {
            loc: loc.clone(),
            name: "MyStruct".to_string(),
        },
        fields: vec![a.clone(), b.clone(), c.clone()],
    });

    contract.add_global_symbol(
        &Identifier {
            loc: loc.clone(),
            name: "MyStruct".to_string(),
        },
        GlobalSymbol::Struct(SymbolInfo {
            loc: loc.clone(),
            i: 0,
        }),
    );

    let ident = Identifier {
        loc: loc.clone(),
        name: String::from("my_var"),
    };

    scope.tables[scope.current].add(
        &mut contract,
        &ident,
        TypeVariant::Struct(SymbolInfo {
            loc: loc.clone(),
            i: 0,
        }),
        None,
        VariableKind::Local,
        true,
    );

    let parsed_var = parsed_ast::Expression::Variable(ident.clone());
    let expr = parsed_ast::Expression::MemberAccess(parsed_ast::MemberAccess {
        loc: loc.clone(),
        expr: Box::new(parsed_var),
        member: c.name.clone(),
    });

    let resolved_expr = expression(
        &expr,
        ExpectedType::Concrete(TypeVariant::Address),
        &mut scope,
        &mut contract,
    );
    assert!(resolved_expr.is_ok(), "Errors: {:#?}", contract.diagnostics);

    let Expression::MemberAccess(m_a) = resolved_expr.unwrap() else {
        panic!("Expected member access");
    };

    assert!(matches!(m_a.expr.as_ref(), Expression::Variable(_)));
    let Expression::Variable(var) = m_a.expr.as_ref() else {
        panic!("Expected var as an expression");
    };
    assert_eq!(var.element, 0);
    assert_eq!(
        var.ty,
        TypeVariant::Struct(SymbolInfo {
            loc: loc.clone(),
            i: 0
        })
    );
    assert_eq!(m_a.ty, TypeVariant::Address);
    assert_eq!(m_a.member, (2, loc.clone()));
}

#[test]
fn pipe() {
    let loc = Span { start: 0, end: 0 };
    let mut contract = ContractDefinition::default();
    let mut scope = Scope::default();

    let mut params = IndexMap::new();
    params.insert(
        "a".to_string(),
        Param {
            loc: loc.clone(),
            ty: Type {
                loc: loc.clone(),
                ty: TypeVariant::List(Box::new(TypeVariant::Int)),
            },
            name: Identifier {
                loc: loc.clone(),
                name: "a".to_string(),
            },
            is_mut: true,
            recursive: false,
        },
    );

    let func_ident = Identifier {
        loc: loc.clone(),
        name: "my_func".to_string(),
    };
    contract.functions.push(Function::new(
        loc.clone(),
        false,
        FunctionVisibility::Priv,
        FuncReturnType::Type(Type {
            loc: loc.clone(),
            ty: TypeVariant::List(Box::new(TypeVariant::Int)),
        }),
        func_ident.clone(),
        params,
        None,
    ));

    contract.add_global_symbol(
        &func_ident,
        GlobalSymbol::Function(SymbolInfo {
            loc: loc.clone(),
            i: 0,
        }),
    );

    let list = parsed_ast::Expression::List(parsed_ast::UnaryExpression {
        loc: loc.clone(),
        element: vec![
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "1".to_string(),
            }),
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "2".to_string(),
            }),
            parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
                loc: loc.clone(),
                element: "3".to_string(),
            }),
        ],
    });

    let parsed_call = parsed_ast::Expression::FunctionCall(parsed_ast::FunctionCall {
        loc: loc.clone(),
        name: func_ident.clone(),
        args: vec![],
    });

    let parsed_pipe = parsed_ast::Expression::Pipe(parsed_ast::BinaryExpression {
        loc: loc.clone(),
        left: Box::new(list.clone()),
        right: Box::new(parsed_call.clone()),
    });

    let resolved_expr = expression(
        &parsed_pipe,
        ExpectedType::Concrete(TypeVariant::List(Box::new(TypeVariant::Int))),
        &mut scope,
        &mut contract,
    );

    assert!(resolved_expr.is_ok(), "Errors: {:#?}", contract.diagnostics);

    let Expression::FunctionCall(func_call) = resolved_expr.unwrap() else {
        panic!("Expected function resolved");
    };

    assert_eq!(
        func_call.args,
        vec![expression(
            &list,
            ExpectedType::Dynamic(vec![]),
            &mut scope,
            &mut contract
        )
        .unwrap()]
    );
    assert_eq!(
        func_call.returns,
        TypeVariant::List(Box::new(TypeVariant::Int))
    );
}

#[test]
fn init_struct() {
    let loc = Span { start: 0, end: 0 };
    let mut contract = ContractDefinition::default();
    let mut scope = Scope::default();

    let a = Param {
        loc: loc.clone(),
        ty: Type {
            loc: loc.clone(),
            ty: TypeVariant::String,
        },
        name: Identifier {
            loc: loc.clone(),
            name: "a".to_string(),
        },
        is_mut: false,
        recursive: false,
    };
    let b = Param {
        loc: loc.clone(),
        ty: Type {
            loc: loc.clone(),
            ty: TypeVariant::Uint,
        },
        name: Identifier {
            loc: loc.clone(),
            name: "b".to_string(),
        },
        is_mut: false,
        recursive: false,
    };
    contract.structs.push(StructDeclaration {
        loc: loc.clone(),
        name: Identifier {
            loc: loc.clone(),
            name: "MyStruct".to_string(),
        },
        fields: vec![a.clone(), b.clone()],
    });

    contract.add_global_symbol(
        &Identifier {
            loc: loc.clone(),
            name: "MyStruct".to_string(),
        },
        GlobalSymbol::Struct(SymbolInfo {
            loc: loc.clone(),
            i: 0,
        }),
    );

    let ident = Identifier {
        loc: loc.clone(),
        name: String::from("my_var"),
    };

    scope.tables[scope.current].add(
        &mut contract,
        &ident,
        TypeVariant::Struct(SymbolInfo {
            loc: loc.clone(),
            i: 0,
        }),
        None,
        VariableKind::Local,
        true,
    );

    let number = parsed_ast::Expression::Number(parsed_ast::UnaryExpression {
        loc: loc.clone(),
        element: "1".to_string(),
    });

    let string = parsed_ast::Expression::String(parsed_ast::UnaryExpression {
        loc: loc.clone(),
        element: "Hello World".to_string(),
    });

    let parsed_init = parsed_ast::Expression::StructInit(parsed_ast::StructInit {
        loc: loc.clone(),
        name: Identifier {
            loc: loc.clone(),
            name: "MyStruct".to_string(),
        },
        args: vec![string.clone(), number.clone()],
        auto_object: None,
    });

    let resolved_expr = expression(
        &parsed_init,
        ExpectedType::Concrete(TypeVariant::Struct(SymbolInfo {
            loc: loc.clone(),
            i: 0,
        })),
        &mut scope,
        &mut contract,
    );

    assert!(resolved_expr.is_ok(), "Errors: {:#?}", contract.diagnostics);
    let Expression::StructInit(init) = resolved_expr.unwrap() else {
        panic!()
    };
    assert_eq!(
        init.ty,
        TypeVariant::Struct(SymbolInfo {
            loc: loc.clone(),
            i: 0
        })
    );
    let resolved_args: Vec<Expression> = vec![string, number]
        .iter()
        .map(|e| {
            expression(
                e,
                ExpectedType::Dynamic(vec![TypeVariant::String, TypeVariant::Uint]),
                &mut scope,
                &mut contract,
            )
            .unwrap()
        })
        .collect();
    assert_eq!(init.args, resolved_args)
}
