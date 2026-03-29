use lambda_cicle::core::multiplicity::semiring::{quantity_add, quantity_mul, Quantity};
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

fn semiring_left_distrib(q1: Q, q2: Q, q3: Q) -> bool {
    let left = quantity_mul(
        q_to_quantity(&q1),
        quantity_add(q_to_quantity(&q2), q_to_quantity(&q3)),
    );
    let right = quantity_add(
        quantity_mul(q_to_quantity(&q1), q_to_quantity(&q2)),
        quantity_mul(q_to_quantity(&q1), q_to_quantity(&q3)),
    );
    left == right
}

fn semiring_right_distrib(q1: Q, q2: Q, q3: Q) -> bool {
    let left = quantity_mul(
        quantity_add(q_to_quantity(&q1), q_to_quantity(&q2)),
        q_to_quantity(&q3),
    );
    let right = quantity_add(
        quantity_mul(q_to_quantity(&q1), q_to_quantity(&q3)),
        quantity_mul(q_to_quantity(&q2), q_to_quantity(&q3)),
    );
    left == right
}

fn semiring_omega_absorb(q: Q) -> bool {
    let result = quantity_add(Quantity::Omega, q_to_quantity(&q));
    result == Quantity::Omega
}

fn semiring_zero_mul(q: Q) -> bool {
    let result = quantity_mul(q_to_quantity(&q), Quantity::Zero);
    result == Quantity::Zero
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
fn qc_semiring_left_distrib() {
    quickcheck(semiring_left_distrib as fn(Q, Q, Q) -> bool);
}

#[test]
fn qc_semiring_right_distrib() {
    quickcheck(semiring_right_distrib as fn(Q, Q, Q) -> bool);
}

#[test]
fn qc_semiring_omega_absorb() {
    quickcheck(semiring_omega_absorb as fn(Q) -> bool);
}

#[test]
fn qc_semiring_zero_mul() {
    quickcheck(semiring_zero_mul as fn(Q) -> bool);
}
