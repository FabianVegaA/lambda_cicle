use lambda_cicle::core::ast::types::{Multiplicity, Type};
use lambda_cicle::core::multiplicity::semiring::Quantity;
use lambda_cicle::core::multiplicity::Context;
use quickcheck::quickcheck;

fn context_add_with_empty_is_identity(_dummy: u8) -> bool {
    let ctx1 = Context::new();
    let ctx2 = Context::new();
    match ctx1.add(&ctx2) {
        Ok(result) => result.is_empty(),
        Err(_) => false,
    }
}

fn context_scale_by_one_is_identity(_dummy: u8) -> bool {
    let ctx = Context::new().extend("x".to_string(), Multiplicity::One, Type::unit());
    match ctx.scale(Quantity::One) {
        Ok(scaled) => scaled.get("x") == ctx.get("x"),
        Err(_) => false,
    }
}

fn context_scale_by_zero_makes_all_zero(_dummy: u8) -> bool {
    let ctx = Context::new()
        .extend("x".to_string(), Multiplicity::One, Type::unit())
        .extend("y".to_string(), Multiplicity::Omega, Type::unit());
    match ctx.scale(Quantity::Zero) {
        Ok(scaled) => {
            scaled.get("x").map(|(m, _)| *m == Multiplicity::Zero) == Some(true)
                && scaled.get("y").map(|(m, _)| *m == Multiplicity::Zero) == Some(true)
        }
        Err(_) => false,
    }
}

fn context_add_is_commutative(_dummy: u8) -> bool {
    let ctx1 = Context::new().extend("a".to_string(), Multiplicity::One, Type::unit());
    let ctx2 = Context::new().extend("b".to_string(), Multiplicity::Omega, Type::unit());

    match (ctx1.clone().add(&ctx2), ctx2.clone().add(&ctx1)) {
        (Ok(r1), Ok(r2)) => r1.get("a") == r2.get("a") && r1.get("b") == r2.get("b"),
        _ => false,
    }
}

fn context_scale_distributes_over_add(_dummy: u8) -> bool {
    let ctx1 = Context::new().extend("x".to_string(), Multiplicity::One, Type::unit());
    let ctx2 = Context::new().extend("y".to_string(), Multiplicity::Omega, Type::unit());

    let scaled_sum = match ctx1.add(&ctx2) {
        Ok(sum) => sum.scale(Quantity::One),
        Err(_) => return false,
    };

    let sum_scaled = match (ctx1.scale(Quantity::One), ctx2.scale(Quantity::One)) {
        (Ok(s1), Ok(s2)) => s1.add(&s2),
        _ => return false,
    };

    match (scaled_sum, sum_scaled) {
        (Ok(s1), Ok(s2)) => s1.get("x") == s2.get("x") && s1.get("y") == s2.get("y"),
        _ => false,
    }
}

fn context_extend_lookup_consistency(_dummy: u8) -> bool {
    let ctx = Context::new()
        .extend("a".to_string(), Multiplicity::One, Type::int())
        .extend("b".to_string(), Multiplicity::Omega, Type::bool());

    ctx.get("a") == Some(&(Multiplicity::One, Type::int()))
        && ctx.get("b") == Some(&(Multiplicity::Omega, Type::bool()))
        && ctx.get("c").is_none()
}

#[test]
fn qc_context_add_with_empty_is_identity() {
    quickcheck(context_add_with_empty_is_identity as fn(u8) -> bool);
}

#[test]
fn qc_context_scale_by_one_is_identity() {
    quickcheck(context_scale_by_one_is_identity as fn(u8) -> bool);
}

#[test]
fn qc_context_scale_by_zero_makes_all_zero() {
    quickcheck(context_scale_by_zero_makes_all_zero as fn(u8) -> bool);
}

#[test]
fn qc_context_add_is_commutative() {
    quickcheck(context_add_is_commutative as fn(u8) -> bool);
}

#[test]
fn qc_context_scale_distributes_over_add() {
    quickcheck(context_scale_distributes_over_add as fn(u8) -> bool);
}

#[test]
fn qc_context_extend_lookup_consistency() {
    quickcheck(context_extend_lookup_consistency as fn(u8) -> bool);
}
