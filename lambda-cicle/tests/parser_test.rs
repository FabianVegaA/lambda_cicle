use lambda_cicle::core::ast::{Literal, Term};
use lambda_cicle::parse;

#[test]
fn test_parse_int_literal() {
    let result = parse("42");
    assert!(result.is_ok());
    let term = result.unwrap();
    match term {
        Term::NativeLiteral(Literal::Int(n)) => assert_eq!(n, 42),
        _ => panic!("Expected int literal"),
    }
}

#[test]
fn test_parse_bool_literal() {
    let result = parse("true");
    assert!(result.is_ok());
    let term = result.unwrap();
    match term {
        Term::NativeLiteral(Literal::Bool(b)) => assert!(b),
        _ => panic!("Expected bool literal"),
    }
}

#[test]
fn test_parse_variable() {
    let result = parse("x");
    assert!(result.is_ok());
    let term = result.unwrap();
    match term {
        Term::Var(name) => assert_eq!(name, "x"),
        _ => panic!("Expected variable"),
    }
}

#[test]
fn test_parse_let() {
    let result = parse("let x:1:Int = 5 in x");
    assert!(result.is_ok());
}

#[test]
fn test_parse_application() {
    let result = parse("f x");
    assert!(result.is_ok());
    let term = result.unwrap();
    match term {
        Term::App { fun, arg } => match *fun {
            Term::Var(name) => assert_eq!(name, "f"),
            _ => panic!("Expected f"),
        },
        _ => panic!("Expected application"),
    }
}

#[test]
fn test_parse_nested_application() {
    let result = parse("f x y");
    assert!(result.is_ok());
}
