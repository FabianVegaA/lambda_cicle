use lambda_cicle::core::ast::patterns::Pattern;
use lambda_cicle::core::parser::parse;

#[test]
fn test_match_nil_pattern() {
    let result = parse("match xs with { Nil => 0 }");
    assert!(result.is_ok(), "Match with Nil pattern should parse");
}

#[test]
fn test_match_cons_pattern() {
    let result = parse("match xs with { Cons h t => h }");
    assert!(result.is_ok(), "Match with Cons pattern should parse");
}

#[test]
fn test_match_nested_cons_pattern() {
    let result = parse("match xs with { Cons (Cons x Nil) t => x }");
    assert!(
        result.is_ok(),
        "Match with nested Cons pattern should parse"
    );
}

#[test]
fn test_match_multiple_cons_args() {
    let result = parse("match xs with { Cons h t => prim_iadd h t }");
    assert!(
        result.is_ok(),
        "Match with Cons having multiple args should parse"
    );
}

#[test]
fn test_match_with_wildcard_arm() {
    let result = parse("match x with { A => 1 | B => 2 | _ => 0 }");
    assert!(result.is_ok(), "Match with wildcard arm should parse");
}

#[test]
fn test_match_nested_match() {
    let result =
        parse("match opt with { None => 0 | Some x => match x with { Nil => 0 | Cons h t => h } }");
    assert!(result.is_ok(), "Nested match should parse");
}

#[test]
fn test_match_many_arms() {
    let result = parse("match x with { A => 1 | B => 2 | C => 3 | D => 4 | _ => 0 }");
    assert!(result.is_ok(), "Match with many arms should parse");
}

#[test]
fn test_pattern_show_wildcard() {
    let p = Pattern::wildcard();
    assert_eq!(format!("{}", p), "_");
}

#[test]
fn test_pattern_show_var() {
    let p = Pattern::var("x");
    assert_eq!(format!("{}", p), "x");
}

#[test]
fn test_pattern_show_constructor_no_args() {
    let p = Pattern::constructor("Nil", vec![]);
    assert_eq!(format!("{}", p), "Nil");
}

#[test]
fn test_pattern_show_constructor_with_args() {
    let p = Pattern::constructor("Cons", vec![Pattern::var("h"), Pattern::var("t")]);
    let s = format!("{}", p);
    assert!(s.contains("Cons"));
    assert!(s.contains("h"));
    assert!(s.contains("t"));
}

#[test]
fn test_pattern_bindings_wildcard() {
    let p = Pattern::wildcard();
    assert!(p.bindings().is_empty());
}

#[test]
fn test_pattern_bindings_var() {
    let p = Pattern::var("x");
    let bindings = p.bindings();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].0, "x");
}

#[test]
fn test_pattern_bindings_constructor() {
    let p = Pattern::constructor("Cons", vec![Pattern::var("h"), Pattern::var("t")]);
    let bindings = p.bindings();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].0, "h");
    assert_eq!(bindings[1].0, "t");
}

#[test]
fn test_pattern_nested_bindings() {
    let p = Pattern::constructor(
        "Cons",
        vec![Pattern::constructor("Nil", vec![]), Pattern::var("t")],
    );
    let bindings = p.bindings();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].0, "t");
}

#[test]
fn test_pattern_multiplicity_var() {
    let p = Pattern::var("x");
    assert_eq!(
        p.binding_multiplicity(),
        lambda_cicle::core::ast::Multiplicity::One
    );
}

#[test]
fn test_pattern_multiplicity_constructor() {
    let p = Pattern::constructor("Cons", vec![Pattern::var("h"), Pattern::var("t")]);
    assert_eq!(
        p.binding_multiplicity(),
        lambda_cicle::core::ast::Multiplicity::One
    );
}

#[test]
fn test_pattern_multiplicity_wildcard() {
    let p = Pattern::wildcard();
    assert_eq!(
        p.binding_multiplicity(),
        lambda_cicle::core::ast::Multiplicity::Zero
    );
}

#[test]
fn test_deeply_nested_pattern() {
    let result = parse("match xs with { Cons (Cons (Cons x Nil) Nil) t => x }");
    assert!(result.is_ok(), "Deeply nested pattern should parse");
}
