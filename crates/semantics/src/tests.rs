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

const WORKING: &str = r#"

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

state BoundedState {
    field: int
} st [
    field > 0
]

@init
@(any)
fn (r: bool) start(init: int) when () -> (StartState s) 
st [
    r == true,
    s.c < 10
]
{
    let a = a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY";
    let h = hex"1234";
    let b = [1, 2, 3];
    let c = -5;
    let d = s"Hello World";

    move StartState : { a, b, c, d };
    loops(0.0);
    conditionals(false, 5);
    move_state();
    return true;
}

fn () loops(value: float) {
    for (let mut i = 0; i < 10; i + 1) {
        let value = value + 123.0;
        skip;
    }
    let some_list = [-3, 4, 5];

    for (n in some_list) {
        let calc = n * 2;
    }
}

fn () conditionals(cond: bool, value: int) {
    let scoped = -10;
    let mut s = s"Hello";
    s = s + s" " + s"World";
    if cond {
        let a = scoped + 3;
    } else if value > 1 {
        let b = scoped + 4;
    } else {
        let c = scoped + 5;
    }
}

fn () move_state() when (StartState s1) -> (StartState s2) {
    let a = a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY";
    let b = [1, 2, 3];
    let c = -5;
    let d = s"Hello World";
    let gawd = s1.c;
    # this will fails as we can't access the final state variable.
    # let va = s2.c;

    let m = MyModel : { a, b, c, d };
    move StartState : { m };
}

@(any)
view(StartState s) fn int get_value() {
    return s.c;
}
"#;

#[test]
fn test_program() {
    let result = parse(WORKING);
    let Ok(tree) = &result else {
        panic!("{:#?}", &result.err().unwrap());
    };

    let contract = resolve_semantics(tree);
    assert_eq!(contract.diagnostics.len(), 0, "{:#?}", contract.diagnostics);
    assert_eq!(contract.models.len(), 2);
    assert_eq!(contract.states.len(), 3);
    assert_eq!(contract.functions.len(), 5);
    assert_eq!(contract.structs.len(), 0);

    let model = contract
        .models
        .iter()
        .find(|x| &x.name.name == "MyModel")
        .unwrap();

    assert_eq!(model.bounds.len(), 3);

    let func = &contract.functions[0];
    let vars: Vec<&VariableSym> = func.scope.vars.values().collect();
    assert_eq!(vars.len(), 9, "{:#?}", vars);

    assert_eq!(func.return_ty.ty(), &TypeVariant::Bool);
    assert_eq!(func.params.len(), 1);
}

const NOT_WORKING: &str = r#"

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
fn (r: bool) start(init: int) when () -> (StartState s) 
st [
    r == true,
    s.c < 10
]
{
    let a = a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY";
    let b = [1, 2, 3];
    let c = -5;
    let d = s"Hello World";

    move StartState : { a, b, c, d };
    # return true;
}

fn () loops(value: float) {
    for (let mut i = 0; i < 10; i + 1) {
        let value = value + 123.0;
        value = 1;
    }
    let some_list = [-3, 4, 5];

    for (n in some_list) {
        let calc = n * 2;
    }
}

fn () conditionals(cond: bool, value: int) {
    let mut scoped = -10;
    scoped = s"String";
    if cond {
        let a = scoped + 3;
    } else if value > 1 {
        let b = scoped + 4;
    } else {
        let c = scoped + 5;
    }
}

fn () move_state() when () -> (StartState s) {
    let a = a"2FMLYJHYQWRHMFKRHKTKX5UNB5DGO65U57O3YVLWUJWKRE4YYJYC2CWWBY";
    let b = [s"s", 2, 3];
    let c = -5;
    let d = s"Hello World";

    let m = MyModel : { a, b, c, d };
    # move StartState : { m };
}

fn () fail_move_state() when () -> (StartState s) {
}
"#;

#[test]
fn test_err_program() {
    folidity_diagnostics::disable_pretty_print();

    let result = parse(NOT_WORKING);
    let Ok(tree) = &result else {
        panic!("{:#?}", &result.err().unwrap());
    };

    let contract = resolve_semantics(tree);
    // assert_eq!(contract.diagnostics.len(), 0, "{:#?}", contract.diagnostics);
    let mut errors = contract.diagnostics.iter().map(|r| r.message.clone());
    assert_eq!(
        "Expected function to return a value of type bool",
        &errors.next().unwrap()
    );
    assert_eq!(
        "Variable is immutable. Annotate with `mut` keyword to allow mutation.",
        &errors.next().unwrap()
    );
    assert_eq!(
        "Mismatched types: expected to resolve to int, but expression can only resolve to string",
        &errors.next().unwrap()
    );
    assert_eq!(
        "Mismatched types: expected to resolve to string, but expression can only resolve to int",
        &errors.next().unwrap()
    );
    assert_eq!(
        "Mismatched types: expected to resolve to string, but expression can only resolve to int",
        &errors.next().unwrap()
    );
    assert_eq!(
        "List elements appear to be of different types.",
        &errors.next().unwrap()
    );
    assert_eq!(
        "Expected function to transition to states [StartState]",
        &errors.next().unwrap()
    );
}
