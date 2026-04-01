use lambda_cicle::core::parser::parse_program;

#[test]
fn test_sum_type() {
    let result = parse_program("pub type Bool (..) = True | False");
    println!("Sum type error: {:?}", result.as_ref().err());
    assert!(result.is_ok());
}

#[test]
fn test_impl_trait_for_type() {
    let result = parse_program("impl Eq for Int {}");
    assert!(result.is_ok());
}

#[test]
fn test_impl_with_method() {
    let result = parse_program(
        "impl Eq for Int { val eq : Int -> Int -> Bool = \\x : Int. \\y : Int. prim_ieq x y }",
    );
    assert!(result.is_ok());
}

#[test]
fn test_type_with_type_var() {
    let result = parse_program("pub type Option a (..)");
    assert!(result.is_ok());
}

#[test]
fn test_match_with_constructor() {
    let result = parse_program("match xs with { Nil => Unit | Cons h t => Unit }");
    assert!(result.is_ok());
}

#[test]
fn test_match_nested_cons() {
    let result = parse_program("match xs with { Cons (Cons x Nil) t => x }");
    assert!(result.is_ok());
}

#[test]
fn test_match_multiple_arms() {
    let result = parse_program("match x with { Nil => Unit | Cons h t => h | _ => Unit }");
    assert!(result.is_ok());
}

#[test]
fn test_list_type() {
    let result = parse_program("pub type List a (..)");
    assert!(result.is_ok());
}

#[test]
fn test_map_type() {
    let result = parse_program("pub type Map k v (..)");
    assert!(result.is_ok());
}

#[test]
fn test_trait_with_supertrait() {
    let result = parse_program("pub trait Ord a where Eq a { val compare : a -> a -> Ordering }");
    assert!(result.is_ok());
}

#[test]
fn test_fold_right_parsing() {
    let result = parse_program("fold_right (\\x : Int. \\acc : Int. prim_iadd x acc) xs 0");
    assert!(result.is_ok());
}

#[test]
fn test_map_parsing() {
    let result = parse_program("map (\\x : Int. prim_iadd x 1) xs");
    assert!(result.is_ok());
}

#[test]
fn test_let_match() {
    let result = parse_program("let x = match opt with { None => 0 | Some y => y } in x");
    assert!(result.is_ok());
}
