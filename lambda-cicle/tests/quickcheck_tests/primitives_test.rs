use lambda_cicle::runtime::primitives::{PrimOp, PrimVal};
use quickcheck::quickcheck;

fn prim_iadd_commutative(input: (i64, i64)) -> bool {
    let (a, b) = input;
    let result1 = PrimOp::IAdd.apply(&[PrimVal::Int(a), PrimVal::Int(b)]);
    let result2 = PrimOp::IAdd.apply(&[PrimVal::Int(b), PrimVal::Int(a)]);
    result1 == result2
}

fn prim_iadd_associative(input: (i64, i64, i64)) -> bool {
    let (a, b, c) = input;
    let bc = match PrimOp::IAdd.apply(&[PrimVal::Int(b), PrimVal::Int(c)]) {
        Some(PrimVal::Int(n)) => n,
        _ => return true,
    };
    let ab = match PrimOp::IAdd.apply(&[PrimVal::Int(a), PrimVal::Int(b)]) {
        Some(PrimVal::Int(n)) => n,
        _ => return true,
    };
    let result1 = PrimOp::IAdd.apply(&[PrimVal::Int(a), PrimVal::Int(bc)]);
    let result2 = PrimOp::IAdd.apply(&[PrimVal::Int(ab), PrimVal::Int(c)]);
    result1 == result2
}

fn prim_imul_associative(input: (i64, i64, i64)) -> bool {
    let (a, b, c) = input;
    let bc = match PrimOp::IMul.apply(&[PrimVal::Int(b), PrimVal::Int(c)]) {
        Some(PrimVal::Int(n)) => n,
        _ => return true,
    };
    let ab = match PrimOp::IMul.apply(&[PrimVal::Int(a), PrimVal::Int(b)]) {
        Some(PrimVal::Int(n)) => n,
        _ => return true,
    };
    let result1 = PrimOp::IMul.apply(&[PrimVal::Int(a), PrimVal::Int(bc)]);
    let result2 = PrimOp::IMul.apply(&[PrimVal::Int(ab), PrimVal::Int(c)]);
    result1 == result2
}

fn prim_ieq_reflexive(input: i64) -> bool {
    let result = PrimOp::IEq.apply(&[PrimVal::Int(input), PrimVal::Int(input)]);
    result == Some(PrimVal::Bool(true))
}

fn prim_ieq_symmetric(input: (i64, i64)) -> bool {
    let (a, b) = input;
    let result_ab = PrimOp::IEq.apply(&[PrimVal::Int(a), PrimVal::Int(b)]);
    let result_ba = PrimOp::IEq.apply(&[PrimVal::Int(b), PrimVal::Int(a)]);
    result_ab == result_ba
}

fn prim_not_involution(input: bool) -> bool {
    let result = PrimOp::Not.apply(&[PrimVal::Bool(input)]);
    match result {
        Some(PrimVal::Bool(b)) => {
            PrimOp::Not.apply(&[PrimVal::Bool(b)]) == Some(PrimVal::Bool(input))
        }
        _ => false,
    }
}

fn prim_ilt_antisymmetric(input: (i64, i64)) -> bool {
    let (a, b) = input;
    if a == b {
        return true;
    }
    let lt_ab = PrimOp::ILt.apply(&[PrimVal::Int(a), PrimVal::Int(b)]);
    let lt_ba = PrimOp::ILt.apply(&[PrimVal::Int(b), PrimVal::Int(a)]);
    match (lt_ab, lt_ba) {
        (Some(PrimVal::Bool(x)), Some(PrimVal::Bool(y))) => !(x && y),
        _ => true,
    }
}

fn prim_ord_chr_inverse(input: u8) -> bool {
    let c = char::from_u32(input as u32).unwrap_or('a');
    let result = PrimOp::Ord.apply(&[PrimVal::Char(c)]);
    match result {
        Some(PrimVal::Int(n)) => {
            let c2 = PrimOp::Chr.apply(&[PrimVal::Int(n)]);
            c2 == Some(PrimVal::Char(c))
        }
        _ => true,
    }
}

fn prim_and_idempotent(input: bool) -> bool {
    let result = PrimOp::And.apply(&[PrimVal::Bool(input), PrimVal::Bool(input)]);
    result == Some(PrimVal::Bool(input))
}

fn prim_or_idempotent(input: bool) -> bool {
    let result = PrimOp::Or.apply(&[PrimVal::Bool(input), PrimVal::Bool(input)]);
    result == Some(PrimVal::Bool(input))
}

#[test]
fn qc_prim_iadd_commutative() {
    quickcheck(prim_iadd_commutative as fn((i64, i64)) -> bool);
}

#[test]
fn qc_prim_iadd_associative() {
    quickcheck(prim_iadd_associative as fn((i64, i64, i64)) -> bool);
}

#[test]
fn qc_prim_imul_associative() {
    quickcheck(prim_imul_associative as fn((i64, i64, i64)) -> bool);
}

#[test]
fn qc_prim_ieq_reflexive() {
    quickcheck(prim_ieq_reflexive as fn(i64) -> bool);
}

#[test]
fn qc_prim_ieq_symmetric() {
    quickcheck(prim_ieq_symmetric as fn((i64, i64)) -> bool);
}

#[test]
fn qc_prim_not_involution() {
    quickcheck(prim_not_involution as fn(bool) -> bool);
}

#[test]
fn qc_prim_ilt_antisymmetric() {
    quickcheck(prim_ilt_antisymmetric as fn((i64, i64)) -> bool);
}

#[test]
fn qc_prim_ord_chr_inverse() {
    quickcheck(prim_ord_chr_inverse as fn(u8) -> bool);
}

#[test]
fn qc_prim_and_idempotent() {
    quickcheck(prim_and_idempotent as fn(bool) -> bool);
}

#[test]
fn qc_prim_or_idempotent() {
    quickcheck(prim_or_idempotent as fn(bool) -> bool);
}
