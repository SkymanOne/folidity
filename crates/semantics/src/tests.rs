use crate::{
    ast::TypeVariant,
    resolve_semantics,
    symtable::VariableSym,
};
use folidity_parser::parse;

const DECL_SRC: &str = r#"
struct MyStruct {
    a: int,
    b: address
}

enum MyEnum {
    A, 
    B
}

model MyModel: ParentModel {
    c: int,
    b: string,
}

state NoState(MyModel)
"#;

#[test]
fn test_first_pass() {
    let tree = parse(DECL_SRC).unwrap();

    let def = resolve_semantics(&tree);
    assert_eq!(def.structs.len(), 1);
    assert_eq!(def.enums.len(), 1);
    assert_eq!(def.models.len(), 1);
    assert_eq!(def.states.len(), 1);

    let e = &def.enums[0];
    assert_eq!(e.variants.len(), 2);

    assert!(e.variants.contains_key("A"));
    assert!(e.variants.contains_key("B"));
}

const FUNCS: &str = r#"

model ParentModel {
    a: address,
    b: list<int>    
}

model MyModel: ParentModel {
    c: int,
    d: string
} st [
    a == a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY",
    c > -10,
    d == s"Hello World"
]

state StartState(MyModel)

state SecondState(MyModel)

@init
@(any)
fn bool start(init: int) when () -> (StartState s) 
{
    let a = a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY";
    let b = [1, 2, 3];
    let c = -5;
    let d = s"Hello World";

    move StartState : { a, b, c, d };
    do_something(0.0);
    return true;
}

fn () do_something(value: float) {
    for (let mut i = 0; i < 10; i + 1) {
        let mut value = value + 123.0;
    }
}

"#;

#[test]
fn test_function() {
    let result = parse(FUNCS);
    let Ok(tree) = &result else {
        panic!("{:#?}", &result.err().unwrap());
    };

    let contract = resolve_semantics(tree);
    assert_eq!(contract.diagnostics.len(), 0, "{:#?}", contract.diagnostics);
    assert_eq!(contract.models.len(), 2);
    assert_eq!(contract.states.len(), 2);
    assert_eq!(contract.structs.len(), 0);

    let model = contract
        .models
        .iter()
        .find(|x| &x.name.name == "MyModel")
        .unwrap();

    assert_eq!(model.bounds.len(), 3);

    let func = &contract.functions[0];
    let vars: Vec<&VariableSym> = func.scope.vars.values().collect();
    assert_eq!(vars.len(), 6);

    assert_eq!(func.return_ty.ty(), &TypeVariant::Bool);
    assert_eq!(func.params.len(), 1);
}
