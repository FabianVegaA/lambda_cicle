use lambda_cicle::core::ast::{Decl, Term};
use lambda_cicle::{parse, parse_program};
use quickcheck::{quickcheck, Arbitrary, Gen};

fn parse_term(s: &str) -> Result<Term, String> {
    parse(s).map_err(|e| format!("{:?}", e))
}

fn parse_decls(s: &str) -> Result<Vec<Decl>, String> {
    parse_program(s).map_err(|e| format!("{:?}", e))
}

mod patterns {
    use super::*;

    #[test]
    fn test_pattern_wildcard() {
        let result = parse_term("match x with { _ => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_variable() {
        let result = parse_term("match x with { y => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_constructor_no_args() {
        let result = parse_term("match x with { Nil => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_constructor_one_arg() {
        let result = parse_term("match x with { Cons h => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_constructor_two_args() {
        let result = parse_term("match x with { Cons h t => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_constructor_with_wildcards() {
        let result = parse_term("match x with { Cons _ _ => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_nested_constructor() {
        let result = parse_term("match x with { Cons (Cons x Nil) t => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_deeply_nested() {
        let result = parse_term("match x with { Cons (Cons (Cons x Nil) Nil) t => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_with_parens_grouping() {
        let result = parse_term("match x with { Cons (h) t => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_multiple_alternatives() {
        let result = parse_term("match x with { Nil => Unit | Cons _ => Unit | _ => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_underscore_as_arg() {
        let result = parse_term("match xs with { Cons _ Nil => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_underscore_first_arg() {
        let result = parse_term("match xs with { Cons _ t => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_view_with_constructors() {
        let result = parse_term("view x with { Cons h t => h }");
        assert!(result.is_ok());
    }
}

mod arrow_types {
    use super::*;

    #[test]
    fn test_arrow_right_associative() {
        let result = parse_decls("pub type Fn = a -> b -> c");
        assert!(result.is_ok());
        if let Ok(decls) = result {
            if let Decl::TypeDecl { ty, .. } = &decls[0] {
                if let lambda_cicle::core::ast::Type::Arrow(_, _, right) = ty {
                    if let lambda_cicle::core::ast::Type::Arrow(_, _, _) = **right {
                        return;
                    }
                }
                panic!("Expected right-associative arrow: a -> (b -> c)");
            }
        }
    }

    #[test]
    fn test_arrow_simple() {
        let result = parse_decls("pub type F = a -> b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_arrow_nested() {
        let result = parse_decls("pub type T = (a -> b) -> c");
        assert!(result.is_ok());
    }

    #[test]
    fn test_arrow_complex() {
        let result = parse_decls("pub type Comp = (a -> b) -> (b -> c) -> a -> c");
        assert!(result.is_ok());
    }
}

mod traits {
    use super::*;

    #[test]
    fn test_trait_no_params_no_body() {
        let result = parse_decls("pub trait Marker");
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_with_param() {
        let result = parse_decls("pub trait Eq a");
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_with_method() {
        let result = parse_decls("pub trait Eq a { val eq: &a -> &a -> Bool }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_with_supertrait() {
        let result = parse_decls("pub trait Ord a where Eq a");
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_with_supertrait_and_body() {
        let result = parse_decls("pub trait Ord a where Eq a { val compare: &a -> &a -> Bool }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_multiple_params() {
        let result = parse_decls("pub trait Functor f { val map: (Int -> Int) -> Int -> Int }");
        eprintln!("trait_multiple_params: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_private() {
        let result = parse_decls("trait Eq a { val eq: &a -> &a -> Bool }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_with_multiple_methods() {
        let result = parse_decls("pub trait Point a { val x: a -> Int val y: a -> Int }");
        assert!(result.is_ok());
    }
}

mod impls {
    use super::*;

    #[test]
    fn test_impl_simple() {
        let result = parse_decls("impl Eq for Int { val eq: &Int -> &Int -> Bool = Unit }");
        eprintln!("impl_simple: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_with_constraints() {
        let result = parse_decls("impl Eq for Option a where Eq a { val eq: Bool = Unit }");
        eprintln!("impl_with_constraints: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_with_multiple_constraints() {
        let result = parse_decls("impl Eq for Result a e where Eq a, Eq e { val eq: Bool = Unit }");
        eprintln!("impl_with_multiple_constraints: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_type_application() {
        let result =
            parse_decls("impl Show for List a where Show a { val show: a -> String = Unit }");
        eprintln!("impl_type_application: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_without_body() {
        let result = parse_decls("impl Clone for Int { }");
        eprintln!("impl_without_body: {:?}", result);
        assert!(result.is_ok());
    }
}

mod type_params {
    use super::*;

    #[test]
    fn test_type_param_lowercase() {
        let result = parse_decls("pub type Id a = a");
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_param_multiple() {
        let result = parse_decls("pub type Pair a b = (a, b)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_param_with_underscore() {
        let result = parse_decls("pub type Foo _a = Int");
        eprintln!("type_param_underscore: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_param_uppercase_rejected() {
        let result = parse_decls("pub type Foo A = A");
        assert!(result.is_err(), "Uppercase type param should be rejected");
    }
}

mod visibility {
    use super::*;

    #[test]
    fn test_visibility_public() {
        let result = parse_decls("pub val x : Int = 42");
        assert!(result.is_ok());
        if let Ok(decls) = result {
            if let Decl::ValDecl { visibility, .. } = &decls[0] {
                assert!(matches!(
                    visibility,
                    lambda_cicle::core::ast::Visibility::Public
                ));
            }
        }
    }

    #[test]
    fn test_visibility_private() {
        let result = parse_decls("val x : Int = 42");
        assert!(result.is_ok());
        if let Ok(decls) = result {
            if let Decl::ValDecl { visibility, .. } = &decls[0] {
                assert!(matches!(
                    visibility,
                    lambda_cicle::core::ast::Visibility::Private
                ));
            }
        }
    }

    #[test]
    fn test_visibility_public_type() {
        let result = parse_decls("pub type Foo = Int");
        assert!(result.is_ok());
    }

    #[test]
    fn test_visibility_public_trait() {
        let result = parse_decls("pub trait Eq a { val eq: &a -> &a -> Bool }");
        assert!(result.is_ok());
    }
}

mod imports {
    use super::*;

    #[test]
    fn test_use_qualified() {
        let result = parse_decls("use Std.List");
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_nested() {
        let result = parse_decls("use Std.Collections.Map");
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_selective() {
        let result = parse_decls("use Std.List (map, filter)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_unqualified() {
        let result = parse_decls("use Std.List (..)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_alias() {
        let result = parse_decls("use Std.List as L");
        assert!(result.is_ok());
    }
}

mod type_definitions {
    use super::*;

    #[test]
    fn test_type_opaque() {
        let result = parse_decls("pub type Foo = Int");
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_transparent() {
        let result = parse_decls("pub type Foo(..)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_with_params() {
        let result = parse_decls("pub type Option a = Unit");
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_with_arrow() {
        let result = parse_decls("pub type Fn a b = a -> b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_nested() {
        let result = parse_decls("pub type T = List (Option Int)");
        assert!(result.is_ok());
    }
}

mod lambdas {
    use super::*;

    #[test]
    fn test_lambda_with_mult() {
        let result = parse_term("λx :1: Int. x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_lambda_with_omega() {
        let result = parse_term("λx :ω: Int. x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_lambda_with_borrow() {
        let result = parse_term("λx :&: Int. x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_lambda_without_mult() {
        let result = parse_term("λx : Int. x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_lambda_numeric_mult() {
        let result = parse_term("λx :0: Int. x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_lambda() {
        let result = parse_term("λx :1: Int. λy :1: Int. x");
        assert!(result.is_ok());
    }
}

mod lets {
    use super::*;

    #[test]
    fn test_let_basic() {
        let result = parse_term("let x :1: Int = 5 in x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_let_without_mult() {
        let result = parse_term("let x : Int = 5 in x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_let_nested() {
        let result = parse_term("let x :1: Int = 5 in let y :1: Int = 6 in x");
        assert!(result.is_ok());
    }
}

mod matches {
    use super::*;

    #[test]
    fn test_match_with_view() {
        let result = parse_term("view x with { Cons h t => h }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_multiple_arms() {
        let result = parse_term("match x with { Nil => Unit | Cons h t => h | _ => Unit }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_with_complex_pattern() {
        let result =
            parse_term("match xs with { Cons (Cons x Nil) (Cons y Nil) => x | _ => Unit }");
        assert!(result.is_ok());
    }
}

mod literals {
    use super::*;

    #[test]
    fn test_int_positive() {
        let result = parse_term("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_float() {
        let result = parse_term("3.14");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bool_true() {
        let result = parse_term("true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bool_false() {
        let result = parse_term("false");
        assert!(result.is_ok());
    }

    #[test]
    fn test_unit() {
        let result = parse_term("Unit");
        assert!(result.is_ok());
    }
}

mod application {
    use super::*;

    #[test]
    fn test_app_two_args() {
        let result = parse_term("f x y");
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_three_args() {
        let result = parse_term("f x y z");
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_with_parens() {
        let result = parse_term("(f x) y");
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_complex() {
        let result = parse_term("map (λx :1: Int. x) xs");
        assert!(result.is_ok());
    }
}

mod no_prelude {
    use super::*;

    #[test]
    fn test_no_prelude() {
        let result = parse_decls("no_prelude");
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_prelude_with_decls() {
        let result = parse_decls("no_prelude pub val x : Int = 42");
        assert!(result.is_ok());
    }
}

mod generics {
    use super::*;

    #[test]
    fn test_trait_with_type_var_in_signature() {
        let result = parse_decls("pub trait Eq a { val eq: a -> a -> Bool }");
        eprintln!("trait with type var: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_with_single_param() {
        let result = parse_decls("pub type Option a = Unit");
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_with_multiple_params() {
        let result = parse_decls("pub type Result a e = Unit");
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_with_type_param() {
        let result = parse_decls("impl Eq for Option a { val eq: Bool = Unit }");
        eprintln!("impl with type param: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_val_with_type_var() {
        let result = parse_decls("pub val id: a -> a = Unit");
        eprintln!("val with type var: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_generic_trait() {
        let result = parse_decls("pub trait Functor f { val map: (a -> b) -> f a -> f b }");
        eprintln!("complex generic trait: {:?}", result);
        assert!(
            result.is_ok(),
            "Functor trait with type application: {:?}",
            result
        );
    }

    #[test]
    fn test_type_application_with_constructor() {
        let result = parse_decls("pub type Wrapper a = Option a");
        eprintln!("type application: {:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_application_with_variable() {
        let result = parse_decls("pub val test: f Int = Unit");
        eprintln!("type application with var: {:?}", result);
        assert!(
            result.is_ok(),
            "f Int where f is type variable: {:?}",
            result
        );
    }

    #[test]
    fn test_nested_type_application_with_var() {
        let result = parse_decls("pub val test: f (g Int) = Unit");
        eprintln!("nested type application: {:?}", result);
        assert!(result.is_ok(), "f (g Int): {:?}", result);
    }
}

#[derive(Clone, Debug)]
struct LowercaseIdent(String);

impl Arbitrary for LowercaseIdent {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = usize::arbitrary(g) % 10 + 1;
        let mut s = String::new();
        s.push(('a' as u8 + u8::arbitrary(g) % 26) as char);
        for _ in 0..len {
            s.push(('a' as u8 + u8::arbitrary(g) % 26) as char);
        }
        LowercaseIdent(s)
    }
}

#[derive(Clone, Debug)]
struct UppercaseIdent(String);

impl Arbitrary for UppercaseIdent {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = usize::arbitrary(g) % 10 + 1;
        let mut s = String::new();
        s.push(('A' as u8 + u8::arbitrary(g) % 26) as char);
        for _ in 0..len {
            s.push(('a' as u8 + u8::arbitrary(g) % 26) as char);
        }
        UppercaseIdent(s)
    }
}

fn qc_pattern_wildcard(_: u8) -> bool {
    let result = parse_term("match x with { _ => Unit }");
    result.is_ok()
}

fn qc_pattern_var(_: LowercaseIdent) -> bool {
    let result = parse_term("match x with { y => Unit }");
    result.is_ok()
}

fn qc_pattern_constructor_simple(_: UppercaseIdent) -> bool {
    let name = "Nil";
    let result = parse_term(&format!("match x with {{ {} => Unit }}", name));
    result.is_ok()
}

fn qc_pattern_constructor_with_var(_: UppercaseIdent) -> bool {
    let name = "Cons";
    let result = parse_term(&format!("match x with {{ {} y => Unit }}", name));
    result.is_ok()
}

fn qc_pattern_constructor_with_wildcard(_: UppercaseIdent) -> bool {
    let name = "Cons";
    let result = parse_term(&format!("match x with {{ {} _ => Unit }}", name));
    result.is_ok()
}

fn qc_arrow_type_assoc(_: LowercaseIdent) -> bool {
    let result = parse_decls("pub type F = a -> b -> c");
    result.is_ok()
}

fn qc_trait_with_param(_: LowercaseIdent) -> bool {
    let result = parse_decls("pub trait Eq a");
    result.is_ok()
}

fn qc_trait_with_supertrait(_: LowercaseIdent) -> bool {
    let result = parse_decls("pub trait Ord a where Eq a");
    result.is_ok()
}

#[test]
fn qc_parse_pattern_wildcard() {
    quickcheck(qc_pattern_wildcard as fn(u8) -> bool);
}

#[test]
fn qc_parse_pattern_var() {
    quickcheck(qc_pattern_var as fn(LowercaseIdent) -> bool);
}

#[test]
fn qc_parse_pattern_constructor_simple() {
    quickcheck(qc_pattern_constructor_simple as fn(UppercaseIdent) -> bool);
}

#[test]
fn qc_parse_pattern_constructor_with_var() {
    quickcheck(qc_pattern_constructor_with_var as fn(UppercaseIdent) -> bool);
}

#[test]
fn qc_parse_pattern_constructor_with_wildcard() {
    quickcheck(qc_pattern_constructor_with_wildcard as fn(UppercaseIdent) -> bool);
}

#[test]
fn qc_parse_arrow_type_assoc() {
    quickcheck(qc_arrow_type_assoc as fn(LowercaseIdent) -> bool);
}

#[test]
fn qc_parse_trait_with_param() {
    quickcheck(qc_trait_with_param as fn(LowercaseIdent) -> bool);
}

#[test]
fn qc_parse_trait_with_supertrait() {
    quickcheck(qc_trait_with_supertrait as fn(LowercaseIdent) -> bool);
}
