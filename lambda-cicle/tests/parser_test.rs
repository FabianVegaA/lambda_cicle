use lambda_cicle::core::ast::{Decl, Literal, Term, Visibility};
use lambda_cicle::{parse, parse_program};

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
    assert!(result.is_ok(), "Failed to parse: {:?}", result);
}

#[test]
fn test_parse_let_multi_digit() {
    for value in &[5, 10, 42, 100] {
        let source = format!("let x:1:Int = {} in x", value);
        let result = parse(&source);
        assert!(result.is_ok(), "Failed to parse {}: {:?}", value, result);
    }
}

#[test]
fn test_parse_lambda_no_annotation() {
    let source = r"\x. x";
    let result = parse(source);
    eprintln!("Testing: {:?}", source);
    eprintln!("Result: {:?}", result);
    assert!(result.is_ok(), "Failed to parse: {:?}", result);
}

#[test]
fn test_parse_lambda_with_type() {
    let source = r"\x : Int. x";
    let result = parse(source);
    eprintln!("Testing: {:?}", source);
    eprintln!("Result: {:?}", result);
    assert!(result.is_ok(), "Failed to parse: {:?}", result);
}

#[test]
fn test_parse_lambda_nested() {
    // Test just "1" alone
    let result0 = parse("1");
    eprintln!("Parse '1': {:?}", result0);

    // Test simpler cases first
    let source1 = "(\\x. x) 1";
    let result1 = parse(source1);
    eprintln!("Testing: {:?}", source1);
    eprintln!("Result: {:?}", result1);

    assert!(result1.is_ok(), "Failed: {:?}", result1);
}

#[test]
fn test_parse_application() {
    let result = parse("f x");
    assert!(result.is_ok());
    let term = result.unwrap();
    match term {
        Term::App { fun, arg: _ } => match *fun {
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

#[test]
fn test_parse_lambda_syntax() {
    let result = parse("λx:1:Int.x");
    assert!(
        result.is_ok(),
        "Lambda (λ) syntax should parse: {:?}",
        result
    );
    let term = result.unwrap();
    match term {
        Term::Abs { var, .. } => assert_eq!(var, "x"),
        _ => panic!("Expected lambda abstraction"),
    }
}

#[test]
fn test_parse_lambda_no_mult() {
    // \x : Int. x - type but no multiplicity
    let source = r"(\x : Int. x) 5";
    let result = parse(source);
    eprintln!("Testing: {:?}", source);
    eprintln!("Result: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_parse_lambda_with_mult() {
    // \x :1: Int. x - explicit multiplicity
    let source = r"(\x :1: Int. x) 5";
    let result = parse(source);
    eprintln!("Testing: {:?}", source);
    eprintln!("Result: {:?}", result);
    assert!(result.is_ok(), "Failed: {:?}", result);
}

#[test]
fn test_parse_unit() {
    let result = parse("Unit");
    assert!(result.is_ok(), "Unit should parse: {:?}", result);
    let term = result.unwrap();
    match term {
        Term::NativeLiteral(Literal::Unit) => (),
        _ => panic!("Expected unit literal, got {:?}", term),
    }
}

#[test]
fn test_parse_type_decl_opaque() {
    let result = parse_program("pub type Foo = Int");
    assert!(result.is_ok(), "Type decl should parse: {:?}", result);
    let decls = result.unwrap();
    assert_eq!(decls.len(), 1);
    match &decls[0] {
        Decl::TypeDecl {
            name,
            visibility,
            transparent,
            ..
        } => {
            assert_eq!(name, "Foo");
            assert!(matches!(visibility, Visibility::Public));
            assert!(!*transparent);
        }
        _ => panic!("Expected type declaration"),
    }
}

#[test]
fn test_parse_type_decl_transparent() {
    let result = parse_program("pub type Foo(..)");
    assert!(
        result.is_ok(),
        "Transparent type should parse: {:?}",
        result
    );
    let decls = result.unwrap();
    assert_eq!(decls.len(), 1);
    match &decls[0] {
        Decl::TypeDecl {
            name, transparent, ..
        } => {
            assert_eq!(name, "Foo");
            assert!(*transparent);
        }
        _ => panic!("Expected type declaration"),
    }
}

#[test]
fn test_parse_val_decl() {
    let result = parse_program("pub val x : Int = 42");
    assert!(result.is_ok(), "Val decl should parse: {:?}", result);
    let decls = result.unwrap();
    assert_eq!(decls.len(), 1);
    match &decls[0] {
        Decl::ValDecl {
            name, visibility, ..
        } => {
            assert_eq!(name, "x");
            assert!(matches!(visibility, Visibility::Public));
        }
        _ => panic!("Expected val declaration"),
    }
}

#[test]
fn test_parse_val_decl_private() {
    let result = parse_program("val x : Int = 42");
    assert!(
        result.is_ok(),
        "Private val decl should parse: {:?}",
        result
    );
    let decls = result.unwrap();
    match &decls[0] {
        Decl::ValDecl { visibility, .. } => {
            assert!(matches!(visibility, Visibility::Private));
        }
        _ => panic!("Expected val declaration"),
    }
}

#[test]
fn test_parse_use_qualified() {
    let result = parse_program("use Std.List");
    assert!(result.is_ok(), "Use qualified should parse: {:?}", result);
    let decls = result.unwrap();
    assert_eq!(decls.len(), 1);
    match &decls[0] {
        Decl::UseDecl { path, .. } => {
            assert_eq!(path, &vec!["Std".to_string(), "List".to_string()]);
        }
        _ => panic!("Expected use declaration"),
    }
}

#[test]
fn test_parse_use_selective() {
    let result = parse_program("use Std.List (map, filter)");
    assert!(result.is_ok(), "Use selective should parse: {:?}", result);
    let decls = result.unwrap();
    match &decls[0] {
        Decl::UseDecl { path, mode } => {
            assert_eq!(path, &vec!["Std".to_string(), "List".to_string()]);
            if let lambda_cicle::core::ast::UseMode::Selective(items) = mode {
                assert_eq!(items, &vec!["map".to_string(), "filter".to_string()]);
            } else {
                panic!("Expected selective mode");
            }
        }
        _ => panic!("Expected use declaration"),
    }
}

#[test]
fn test_parse_no_prelude() {
    let result = parse_program("no_prelude");
    assert!(result.is_ok(), "no_prelude should parse: {:?}", result);
    let decls = result.unwrap();
    assert_eq!(decls.len(), 1);
    assert!(matches!(decls[0], Decl::NoPrelude));
}

#[test]
fn test_exports_from_decl_filters_private() {
    let result = parse_program("val x : Int = 42");
    assert!(result.is_ok());
    let decls = result.unwrap();

    let exports = lambda_cicle::modules::Exports::from_decl(&decls);

    // Private val should not be in public exports
    assert!(exports.public_values().next().is_none());
}

#[test]
fn test_exports_from_decl_includes_public() {
    let result = parse_program("pub val x : Int = 42");
    assert!(result.is_ok());
    let decls = result.unwrap();

    let exports = lambda_cicle::modules::Exports::from_decl(&decls);

    // Public val should be in exports
    let public_vals: Vec<_> = exports.public_values().collect();
    assert_eq!(public_vals.len(), 1);
    assert_eq!(public_vals[0].0, "x");
}

#[test]
fn test_exports_type_opaque() {
    let result = parse_program("pub type Foo = Int");
    assert!(result.is_ok());
    let decls = result.unwrap();

    let exports = lambda_cicle::modules::Exports::from_decl(&decls);

    let public_types: Vec<_> = exports.public_types().collect();
    assert_eq!(public_types.len(), 1);
    assert_eq!(public_types[0].0, "Foo");
    assert!(!public_types[0].1.transparent); // opaque
}

#[test]
fn test_exports_type_transparent() {
    let result = parse_program("pub type Bar(..)");
    assert!(result.is_ok());
    let decls = result.unwrap();

    let exports = lambda_cicle::modules::Exports::from_decl(&decls);

    let public_types: Vec<_> = exports.public_types().collect();
    assert_eq!(public_types.len(), 1);
    assert_eq!(public_types[0].0, "Bar");
    assert!(public_types[0].1.transparent); // transparent
}

#[test]
fn test_parse_arrow_type() {
    let result = parse_program("pub type Fn a = a -> a");
    eprintln!("Arrow type result: {:?}", result);
    assert!(result.is_ok(), "Arrow type should parse: {:?}", result);
}

#[test]
fn test_parse_type_var() {
    let result = parse_program("pub type Option a = Unit");
    eprintln!("Type var result: {:?}", result);
    assert!(result.is_ok(), "Type variable should parse: {:?}", result);
}

#[test]
fn test_parse_ref_type() {
    let result = parse_program("pub type Ref a = &a");
    eprintln!("Ref type result: {:?}", result);
    assert!(result.is_ok(), "Reference type should parse: {:?}", result);
}

#[test]
fn test_parse_trait_with_method() {
    let result = parse_program("pub trait Eq a where { val eq: &a -> &a -> Bool }");
    eprintln!("Trait with method result: {:?}", result);
    assert!(
        result.is_ok(),
        "Trait with method should parse: {:?}",
        result
    );
}

#[test]
fn test_parse_trait_with_supertrait() {
    let result =
        parse_program("pub trait Ord a where Eq a where { val compare: &a -> &a -> Bool }");
    eprintln!("Trait with supertrait result: {:?}", result);
    assert!(
        result.is_ok(),
        "Trait with supertrait should parse: {:?}",
        result
    );
}

#[test]
fn test_parse_impl_for_trait() {
    let result = parse_program("impl Int : Ord where {}");
    eprintln!("Impl result: {:?}", result);
    assert!(result.is_ok(), "Impl should parse: {:?}", result);
}
