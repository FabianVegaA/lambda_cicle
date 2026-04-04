use crate::runtime::primitives::operations::PrimOp;
use crate::runtime::primitives::PrimVal;

fn eval_prim(op: PrimOp, args: Vec<PrimVal>) -> Option<PrimVal> {
    op.apply(&args)
}

#[test]
fn test_prim_iadd() {
    assert_eq!(
        eval_prim(PrimOp::IAdd, vec![PrimVal::Int(3), PrimVal::Int(5)]),
        Some(PrimVal::Int(8))
    );
    assert_eq!(
        eval_prim(PrimOp::IAdd, vec![PrimVal::Int(-1), PrimVal::Int(1)]),
        Some(PrimVal::Int(0))
    );
    assert_eq!(
        eval_prim(PrimOp::IAdd, vec![PrimVal::Int(0), PrimVal::Int(0)]),
        Some(PrimVal::Int(0))
    );
}

#[test]
fn test_prim_isub() {
    assert_eq!(
        eval_prim(PrimOp::ISub, vec![PrimVal::Int(10), PrimVal::Int(3)]),
        Some(PrimVal::Int(7))
    );
    assert_eq!(
        eval_prim(PrimOp::ISub, vec![PrimVal::Int(3), PrimVal::Int(10)]),
        Some(PrimVal::Int(-7))
    );
}

#[test]
fn test_prim_imul() {
    assert_eq!(
        eval_prim(PrimOp::IMul, vec![PrimVal::Int(6), PrimVal::Int(7)]),
        Some(PrimVal::Int(42))
    );
    assert_eq!(
        eval_prim(PrimOp::IMul, vec![PrimVal::Int(-3), PrimVal::Int(5)]),
        Some(PrimVal::Int(-15))
    );
}

#[test]
fn test_prim_ineg() {
    assert_eq!(
        eval_prim(PrimOp::INeg, vec![PrimVal::Int(5)]),
        Some(PrimVal::Int(-5))
    );
    assert_eq!(
        eval_prim(PrimOp::INeg, vec![PrimVal::Int(-5)]),
        Some(PrimVal::Int(5))
    );
    assert_eq!(
        eval_prim(PrimOp::INeg, vec![PrimVal::Int(0)]),
        Some(PrimVal::Int(0))
    );
}

#[test]
fn test_prim_ihash() {
    let result = eval_prim(PrimOp::IHash, vec![PrimVal::Int(42)]);
    assert!(matches!(result, Some(PrimVal::Int(n)) if n >= 0));
}

#[test]
fn test_prim_arities() {
    assert_eq!(PrimOp::INeg.arity(), 1);
    assert_eq!(PrimOp::IAdd.arity(), 2);
    assert_eq!(PrimOp::IEq.arity(), 2);
}

#[test]
fn test_prim_fadd() {
    assert_eq!(
        eval_prim(PrimOp::FAdd, vec![PrimVal::Float(1.5), PrimVal::Float(2.5)]),
        Some(PrimVal::Float(4.0))
    );
}

#[test]
fn test_prim_fsub() {
    assert_eq!(
        eval_prim(PrimOp::FSub, vec![PrimVal::Float(5.0), PrimVal::Float(3.0)]),
        Some(PrimVal::Float(2.0))
    );
}

#[test]
fn test_prim_fmul() {
    assert_eq!(
        eval_prim(PrimOp::FMul, vec![PrimVal::Float(3.0), PrimVal::Float(4.0)]),
        Some(PrimVal::Float(12.0))
    );
}

#[test]
fn test_prim_fneg() {
    assert_eq!(
        eval_prim(PrimOp::FNeg, vec![PrimVal::Float(5.0)]),
        Some(PrimVal::Float(-5.0))
    );
}

#[test]
fn test_prim_prim_name_to_op() {
    use crate::runtime::primitives::prim_name_to_op;
    assert!(prim_name_to_op("prim_iadd").is_some());
    assert!(prim_name_to_op("prim_ieq").is_some());
    assert!(prim_name_to_op("prim_idiv").is_some());
    assert!(prim_name_to_op("invalid_prim").is_none());
}
