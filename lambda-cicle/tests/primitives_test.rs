use lambda_cicle::runtime::primitives::{
    is_valid_primitive, prim_name_to_op, IOOp, NativeKind, PrimOp, PrimVal, INTRINSICS_TABLE,
};

#[test]
fn test_intrinsics_table_count() {
    assert_eq!(
        INTRINSICS_TABLE.len(),
        33,
        "Expected exactly 33 intrinsics (30 closed + 3 IO)"
    );
}

#[test]
fn test_intrinsics_table_all_valid() {
    for name in INTRINSICS_TABLE {
        assert!(is_valid_primitive(name), "Expected {} to be valid", name);
    }
}

#[test]
fn test_invalid_primitive_rejected() {
    assert!(!is_valid_primitive("prim_invalid"));
    assert!(!is_valid_primitive("prim_add")); // wrong prefix
    assert!(!is_valid_primitive("prim_iadd_extra"));
    assert!(!is_valid_primitive(""));
}

#[test]
fn test_binary_ops_have_arity_2() {
    let binary_ops = [
        PrimOp::IAdd,
        PrimOp::ISub,
        PrimOp::IMul,
        PrimOp::IDiv,
        PrimOp::IRem,
        PrimOp::IEq,
        PrimOp::IFEq,
        PrimOp::IGt,
        PrimOp::IGe,
        PrimOp::ILt,
        PrimOp::ILe,
        PrimOp::FEq,
        PrimOp::FNe,
        PrimOp::FGt,
        PrimOp::FGe,
        PrimOp::FLt,
        PrimOp::FLe,
        PrimOp::FAdd,
        PrimOp::FSub,
        PrimOp::FMul,
        PrimOp::FDiv,
        PrimOp::And,
        PrimOp::Or,
    ];
    for op in binary_ops {
        assert_eq!(op.arity(), 2, "Expected {:?} to have arity 2", op);
    }
}

#[test]
fn test_unary_ops_have_arity_1() {
    let unary_ops = [
        PrimOp::INeg,
        PrimOp::FNeg,
        PrimOp::Not,
        PrimOp::Chr,
        PrimOp::Ord,
    ];
    for op in unary_ops {
        assert_eq!(op.arity(), 1, "Expected {:?} to have arity 1", op);
    }
}

#[test]
fn test_prim_name_to_op_iadd() {
    assert_eq!(prim_name_to_op("prim_iadd"), Some(PrimOp::IAdd));
}

#[test]
fn test_prim_name_to_op_ineg() {
    assert_eq!(prim_name_to_op("prim_ineg"), Some(PrimOp::INeg));
}

#[test]
fn test_prim_name_to_op_invalid() {
    assert_eq!(prim_name_to_op("prim_invalid"), None);
    assert_eq!(prim_name_to_op("prim_iaddx"), None);
    assert_eq!(prim_name_to_op("add"), None);
}

#[test]
fn test_iadd() {
    let result = PrimOp::IAdd.apply(&[PrimVal::Int(2), PrimVal::Int(3)]);
    assert_eq!(result, Some(PrimVal::Int(5)));
}

#[test]
fn test_isub() {
    assert_eq!(
        PrimOp::ISub.apply(&[PrimVal::Int(5), PrimVal::Int(3)]),
        Some(PrimVal::Int(2))
    );
}

#[test]
fn test_imul() {
    assert_eq!(
        PrimOp::IMul.apply(&[PrimVal::Int(4), PrimVal::Int(3)]),
        Some(PrimVal::Int(12))
    );
}

#[test]
fn test_idiv() {
    assert_eq!(
        PrimOp::IDiv.apply(&[PrimVal::Int(10), PrimVal::Int(3)]),
        Some(PrimVal::Int(3))
    );
}

#[test]
fn test_irem() {
    assert_eq!(
        PrimOp::IRem.apply(&[PrimVal::Int(10), PrimVal::Int(3)]),
        Some(PrimVal::Int(1))
    );
}

#[test]
fn test_ineg() {
    assert_eq!(
        PrimOp::INeg.apply(&[PrimVal::Int(5)]),
        Some(PrimVal::Int(-5))
    );
}

#[test]
fn test_fadd() {
    assert_eq!(
        PrimOp::FAdd.apply(&[PrimVal::Float(1.5), PrimVal::Float(2.5)]),
        Some(PrimVal::Float(4.0))
    );
}

#[test]
fn test_fsub() {
    assert_eq!(
        PrimOp::FSub.apply(&[PrimVal::Float(5.0), PrimVal::Float(3.0)]),
        Some(PrimVal::Float(2.0))
    );
}

#[test]
fn test_fmul() {
    assert_eq!(
        PrimOp::FMul.apply(&[PrimVal::Float(4.0), PrimVal::Float(3.0)]),
        Some(PrimVal::Float(12.0))
    );
}

#[test]
fn test_fdiv() {
    assert_eq!(
        PrimOp::FDiv.apply(&[PrimVal::Float(10.0), PrimVal::Float(3.0)]),
        Some(PrimVal::Float(10.0 / 3.0))
    );
}

#[test]
fn test_fneg() {
    assert_eq!(
        PrimOp::FNeg.apply(&[PrimVal::Float(5.0)]),
        Some(PrimVal::Float(-5.0))
    );
}

#[test]
fn test_ieq_true() {
    assert_eq!(
        PrimOp::IEq.apply(&[PrimVal::Int(42), PrimVal::Int(42)]),
        Some(PrimVal::Bool(true))
    );
}

#[test]
fn test_ieq_false() {
    assert_eq!(
        PrimOp::IEq.apply(&[PrimVal::Int(42), PrimVal::Int(1)]),
        Some(PrimVal::Bool(false))
    );
}

#[test]
fn test_ifeq() {
    assert_eq!(
        PrimOp::IFEq.apply(&[PrimVal::Int(5), PrimVal::Int(5)]),
        Some(PrimVal::Int(1))
    );
    assert_eq!(
        PrimOp::IFEq.apply(&[PrimVal::Int(5), PrimVal::Int(6)]),
        Some(PrimVal::Int(0))
    );
}

#[test]
fn test_ilt() {
    assert_eq!(
        PrimOp::ILt.apply(&[PrimVal::Int(1), PrimVal::Int(2)]),
        Some(PrimVal::Bool(true))
    );
    assert_eq!(
        PrimOp::ILt.apply(&[PrimVal::Int(2), PrimVal::Int(1)]),
        Some(PrimVal::Bool(false))
    );
}

#[test]
fn test_igt() {
    assert_eq!(
        PrimOp::IGt.apply(&[PrimVal::Int(2), PrimVal::Int(1)]),
        Some(PrimVal::Bool(true))
    );
}

#[test]
fn test_ile() {
    assert_eq!(
        PrimOp::ILe.apply(&[PrimVal::Int(1), PrimVal::Int(2)]),
        Some(PrimVal::Bool(true))
    );
    assert_eq!(
        PrimOp::ILe.apply(&[PrimVal::Int(2), PrimVal::Int(2)]),
        Some(PrimVal::Bool(true))
    );
}

#[test]
fn test_ige() {
    assert_eq!(
        PrimOp::IGe.apply(&[PrimVal::Int(2), PrimVal::Int(2)]),
        Some(PrimVal::Bool(true))
    );
}

#[test]
fn test_feq() {
    assert_eq!(
        PrimOp::FEq.apply(&[PrimVal::Float(3.0), PrimVal::Float(3.0)]),
        Some(PrimVal::Bool(true))
    );
}

#[test]
fn test_fne() {
    assert_eq!(
        PrimOp::FNe.apply(&[PrimVal::Float(3.0), PrimVal::Float(4.0)]),
        Some(PrimVal::Bool(true))
    );
}

#[test]
fn test_not_true() {
    assert_eq!(
        PrimOp::Not.apply(&[PrimVal::Bool(true)]),
        Some(PrimVal::Bool(false))
    );
}

#[test]
fn test_not_false() {
    assert_eq!(
        PrimOp::Not.apply(&[PrimVal::Bool(false)]),
        Some(PrimVal::Bool(true))
    );
}

#[test]
fn test_and() {
    assert_eq!(
        PrimOp::And.apply(&[PrimVal::Bool(true), PrimVal::Bool(true)]),
        Some(PrimVal::Bool(true))
    );
    assert_eq!(
        PrimOp::And.apply(&[PrimVal::Bool(true), PrimVal::Bool(false)]),
        Some(PrimVal::Bool(false))
    );
}

#[test]
fn test_or() {
    assert_eq!(
        PrimOp::Or.apply(&[PrimVal::Bool(false), PrimVal::Bool(true)]),
        Some(PrimVal::Bool(true))
    );
    assert_eq!(
        PrimOp::Or.apply(&[PrimVal::Bool(false), PrimVal::Bool(false)]),
        Some(PrimVal::Bool(false))
    );
}

#[test]
fn test_ord_c() {
    assert_eq!(
        PrimOp::Ord.apply(&[PrimVal::Char('c')]),
        Some(PrimVal::Int(99))
    );
}

#[test]
fn test_chr_42() {
    assert_eq!(
        PrimOp::Chr.apply(&[PrimVal::Int(42)]),
        Some(PrimVal::Char('*'))
    );
}

#[test]
fn test_type_mismatch_int_float() {
    assert_eq!(
        PrimOp::IAdd.apply(&[PrimVal::Int(1), PrimVal::Float(2.0)]),
        None
    );
}

#[test]
fn test_type_mismatch_int_bool() {
    assert_eq!(
        PrimOp::IEq.apply(&[PrimVal::Int(1), PrimVal::Bool(true)]),
        None
    );
}

#[test]
fn test_io_op_arity() {
    assert_eq!(IOOp::Print.arity(), 1);
    assert_eq!(IOOp::ReadLine.arity(), 1);
    assert_eq!(IOOp::OpenFile.arity(), 2);
    assert_eq!(IOOp::CloseFile.arity(), 2);
    assert_eq!(IOOp::FileWrite.arity(), 2);
}

#[test]
fn test_native_kind_all() {
    let kinds = NativeKind::all();
    assert_eq!(kinds.len(), 5);
    assert!(kinds.contains(&NativeKind::Int));
    assert!(kinds.contains(&NativeKind::Float));
    assert!(kinds.contains(&NativeKind::Bool));
    assert!(kinds.contains(&NativeKind::Char));
    assert!(kinds.contains(&NativeKind::Unit));
}

#[test]
fn test_prim_val_native_kind() {
    assert_eq!(PrimVal::Int(0).native_kind(), NativeKind::Int);
    assert_eq!(PrimVal::Float(0.0).native_kind(), NativeKind::Float);
    assert_eq!(PrimVal::Bool(false).native_kind(), NativeKind::Bool);
    assert_eq!(PrimVal::Char('a').native_kind(), NativeKind::Char);
    assert_eq!(PrimVal::Unit.native_kind(), NativeKind::Unit);
}
