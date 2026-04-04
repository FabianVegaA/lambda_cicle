use lambda_cicle::runtime::primitives::{prim_name_to_op, PrimOp, PrimVal};

#[test]
fn test_prim_int_to_string() {
    let result = PrimOp::IntToString.apply(&[PrimVal::Int(42)]);
    assert!(result.is_some(), "IntToString should return Some");
    if let Some(PrimVal::String(s)) = result {
        assert_eq!(s, "42", "IntToString(42) should be \"42\"");
    } else {
        panic!("Expected String variant");
    }
}

#[test]
fn test_prim_int_to_string_negative() {
    let result = PrimOp::IntToString.apply(&[PrimVal::Int(-100)]);
    assert!(result.is_some(), "IntToString should return Some");
    if let Some(PrimVal::String(s)) = result {
        assert_eq!(s, "-100", "IntToString(-100) should be \"-100\"");
    } else {
        panic!("Expected String variant");
    }
}

#[test]
fn test_prim_float_to_string() {
    let result = PrimOp::FloatToString.apply(&[PrimVal::Float(3.14)]);
    assert!(result.is_some(), "FloatToString should return Some");
    if let Some(PrimVal::String(s)) = result {
        assert!(
            s.contains("3.14"),
            "FloatToString(3.14) should contain \"3.14\""
        );
    } else {
        panic!("Expected String variant");
    }
}

#[test]
fn test_prim_char_to_string() {
    let result = PrimOp::CharToString.apply(&[PrimVal::Char('a')]);
    assert!(result.is_some(), "CharToString should return Some");
    if let Some(PrimVal::String(s)) = result {
        assert_eq!(s, "a", "CharToString('a') should be \"a\"");
    } else {
        panic!("Expected String variant");
    }
}

#[test]
fn test_prim_int_to_string_zero() {
    let result = PrimOp::IntToString.apply(&[PrimVal::Int(0)]);
    assert!(result.is_some(), "IntToString(0) should return Some");
    if let Some(PrimVal::String(s)) = result {
        assert_eq!(s, "0", "IntToString(0) should be \"0\"");
    }
}

#[test]
fn test_prim_float_to_string_large() {
    let result = PrimOp::FloatToString.apply(&[PrimVal::Float(1e10)]);
    assert!(
        result.is_some(),
        "FloatToString should handle large numbers"
    );
}

#[test]
fn test_prim_float_to_string_negative() {
    let result = PrimOp::FloatToString.apply(&[PrimVal::Float(-2.5)]);
    assert!(
        result.is_some(),
        "FloatToString should handle negative numbers"
    );
}

#[test]
fn test_int_to_string_type_mismatch() {
    let result = PrimOp::IntToString.apply(&[PrimVal::Float(3.14)]);
    assert_eq!(result, None, "IntToString with Float should return None");
}

#[test]
fn test_char_to_string_type_mismatch() {
    let result = PrimOp::CharToString.apply(&[PrimVal::Int(65)]);
    assert_eq!(result, None, "CharToString with Int should return None");
}

#[test]
fn test_prim_name_to_op_int_to_string() {
    assert_eq!(
        prim_name_to_op("prim_int_to_string"),
        Some(PrimOp::IntToString)
    );
}

#[test]
fn test_prim_name_to_op_float_to_string() {
    assert_eq!(
        prim_name_to_op("prim_float_to_string"),
        Some(PrimOp::FloatToString)
    );
}

#[test]
fn test_prim_name_to_op_char_to_string() {
    assert_eq!(
        prim_name_to_op("prim_char_to_string"),
        Some(PrimOp::CharToString)
    );
}

// NOTE: String conversion intrinsics are not in design doc §16.3
// Commenting out until spec is updated or these are removed
// #[test]
// fn test_intrinsics_table_has_string_conversions() {
//     assert!(
//         INTRINSICS_TABLE.contains(&"prim_int_to_string"),
//         "INTRINSICS_TABLE should contain prim_int_to_string"
//     );
//     assert!(
//         INTRINSICS_TABLE.contains(&"prim_float_to_string"),
//         "INTRINSICS_TABLE should contain prim_float_to_string"
//     );
//     assert!(
//         INTRINSICS_TABLE.contains(&"prim_char_to_string"),
//         "INTRINSICS_TABLE should contain prim_char_to_string"
//     );
// }

#[test]
fn test_int_to_string_arity() {
    assert_eq!(
        PrimOp::IntToString.arity(),
        1,
        "IntToString should have arity 1"
    );
}

#[test]
fn test_float_to_string_arity() {
    assert_eq!(
        PrimOp::FloatToString.arity(),
        1,
        "FloatToString should have arity 1"
    );
}

#[test]
fn test_char_to_string_arity() {
    assert_eq!(
        PrimOp::CharToString.arity(),
        1,
        "CharToString should have arity 1"
    );
}

#[test]
fn test_string_primval_display() {
    let val = PrimVal::String("hello".to_string());
    let s = format!("{:?}", val);
    assert!(
        s.contains("hello"),
        "String PrimVal should display correctly"
    );
}
