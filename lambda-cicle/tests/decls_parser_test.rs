use lambda_cicle::core::parser::parse_program;

#[test]
fn test_parse_val_declaration() {
    let source = "val x : Int = 42";
    let result = parse_program(source);
    assert!(result.is_ok(), "Should parse val: {:?}", result.err());
}

#[test]
fn test_parse_val_with_arrow() {
    let source = "val add : Int -> Int = \\x : Int. x";
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse val with arrow: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_use_directive() {
    let source = "use Std.List";
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse use directive: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_match_expression() {
    let source = r#"
        val test : Int = match x with { True => 1 | False => 0 }
    "#;
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse match expression with =>: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_lambda() {
    let source = r#"
        val foo : Int -> Int = \x : Int. x
    "#;
    let result = parse_program(source);
    assert!(result.is_ok(), "Should parse lambda: {:?}", result.err());
}

#[test]
fn test_parse_nested_match() {
    let source = r#"
        val test : Int = match xs with { Nil => 0 | Cons h t => h }
    "#;
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse nested match: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_constructor_in_match() {
    let source = r#"
        val test : Int = match opt with { None => 0 | Some x => x }
    "#;
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse constructor in match: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_multiple_val_decls() {
    let source = r#"
        val x : Int = 1
        val y : Int = 2
        val z : Int = 3
    "#;
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse multiple val declarations: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_let_binding() {
    let source = "let a : Int = 1 in a";
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse let binding: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_nested_let() {
    let source = "let a : Int = 1 in let b : Int = a in b";
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse nested let: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_application() {
    let source = "(\\x : Int. x) 5";
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse application: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_type_annotations() {
    let source = "val x : Int = 42";
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse type annotations: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_multiplicity_in_lambda() {
    let source = "val id : Int -> Int = \\x :1:Int. x";
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse multiplicity: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_type_var() {
    let source = "val identity : a -> a = \\x : a. x";
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse type variable: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_nested_type_app() {
    let source = "val test : Option (List Int) = None";
    let result = parse_program(source);
    assert!(
        result.is_ok(),
        "Should parse nested type app: {:?}",
        result.err()
    );
}

#[test]
fn test_parse_ref_type() {
    let source = "val borrow : &Int = x";
    let result = parse_program(source);
    assert!(result.is_ok(), "Should parse ref type: {:?}", result.err());
}
