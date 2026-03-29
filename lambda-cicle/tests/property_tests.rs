use lambda_cicle::core::ast::types::{Multiplicity, Type};
use lambda_cicle::core::multiplicity::semiring::{quantity_add, quantity_mul, Quantity};
use lambda_cicle::core::multiplicity::Context;
use quickcheck::{quickcheck, Arbitrary, Gen};

#[derive(Clone, Debug)]
pub enum Q {
    Zero,
    One,
    Omega,
}

impl Arbitrary for Q {
    fn arbitrary(g: &mut Gen) -> Self {
        let idx = usize::arbitrary(g) % 3;
        match idx {
            0 => Q::Zero,
            1 => Q::One,
            _ => Q::Omega,
        }
    }
}

fn q_to_quantity(q: &Q) -> Quantity {
    match q {
        Q::Zero => Quantity::Zero,
        Q::One => Quantity::One,
        Q::Omega => Quantity::Omega,
    }
}

fn q_to_multiplicity(q: &Q) -> Multiplicity {
    match q {
        Q::Zero => Multiplicity::Zero,
        Q::One => Multiplicity::One,
        Q::Omega => Multiplicity::Omega,
    }
}

fn semiring_add_identity(q: Q) -> bool {
    let result = quantity_add(Quantity::Zero, q_to_quantity(&q));
    result == q_to_quantity(&q)
}

fn semiring_add_commutative(q1: Q, q2: Q) -> bool {
    let result1 = quantity_add(q_to_quantity(&q1), q_to_quantity(&q2));
    let result2 = quantity_add(q_to_quantity(&q2), q_to_quantity(&q1));
    result1 == result2
}

fn semiring_add_associative(q1: Q, q2: Q, q3: Q) -> bool {
    let result1 = quantity_add(
        quantity_add(q_to_quantity(&q1), q_to_quantity(&q2)),
        q_to_quantity(&q3),
    );
    let result2 = quantity_add(
        q_to_quantity(&q1),
        quantity_add(q_to_quantity(&q2), q_to_quantity(&q3)),
    );
    result1 == result2
}

fn semiring_mul_zero(q: Q) -> bool {
    let result = quantity_mul(Quantity::Zero, q_to_quantity(&q));
    result == Quantity::Zero
}

fn semiring_mul_one(q: Q) -> bool {
    let result = quantity_mul(Quantity::One, q_to_quantity(&q));
    result == q_to_quantity(&q)
}

fn semiring_mul_commutative(q1: Q, q2: Q) -> bool {
    let result1 = quantity_mul(q_to_quantity(&q1), q_to_quantity(&q2));
    let result2 = quantity_mul(q_to_quantity(&q2), q_to_quantity(&q1));
    result1 == result2
}

fn semiring_mul_associative(q1: Q, q2: Q, q3: Q) -> bool {
    let result1 = quantity_mul(
        quantity_mul(q_to_quantity(&q1), q_to_quantity(&q2)),
        q_to_quantity(&q3),
    );
    let result2 = quantity_mul(
        q_to_quantity(&q1),
        quantity_mul(q_to_quantity(&q2), q_to_quantity(&q3)),
    );
    result1 == result2
}

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

#[test]
fn qc_semiring_add_identity() {
    quickcheck(semiring_add_identity as fn(Q) -> bool);
}

#[test]
fn qc_semiring_add_commutative() {
    quickcheck(semiring_add_commutative as fn(Q, Q) -> bool);
}

#[test]
fn qc_semiring_add_associative() {
    quickcheck(semiring_add_associative as fn(Q, Q, Q) -> bool);
}

#[test]
fn qc_semiring_mul_zero() {
    quickcheck(semiring_mul_zero as fn(Q) -> bool);
}

#[test]
fn qc_semiring_mul_one() {
    quickcheck(semiring_mul_one as fn(Q) -> bool);
}

#[test]
fn qc_semiring_mul_commutative() {
    quickcheck(semiring_mul_commutative as fn(Q, Q) -> bool);
}

#[test]
fn qc_semiring_mul_associative() {
    quickcheck(semiring_mul_associative as fn(Q, Q, Q) -> bool);
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
