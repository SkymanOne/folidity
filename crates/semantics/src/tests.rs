use crate::resolve_semantics;
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
