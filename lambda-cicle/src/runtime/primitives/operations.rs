use crate::core::ast::Literal;
use crate::runtime::primitives::{IOOp, NativeKind, PrimVal};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimOp {
    IAdd,
    ISub,
    IMul,
    IDiv,
    IRem,
    INeg,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FNeg,
    IEq,
    IFEq,
    IGt,
    IGe,
    ILt,
    ILe,
    FEq,
    FNe,
    FGt,
    FGe,
    FLt,
    FLe,
    Not,
    And,
    Or,
    Chr,
    Ord,
}

impl PrimOp {
    pub fn arity(&self) -> usize {
        match self {
            PrimOp::INeg | PrimOp::FNeg | PrimOp::Not | PrimOp::Chr | PrimOp::Ord => 1,
            _ => 2,
        }
    }

    pub fn apply(&self, args: &[PrimVal]) -> Option<PrimVal> {
        match self {
            PrimOp::IEq => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    return Some(PrimVal::Bool(x == y));
                }
                None
            }
            PrimOp::IFEq => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    return Some(PrimVal::Int(if x == y { 1 } else { 0 }));
                }
                None
            }
            PrimOp::IGt => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    return Some(PrimVal::Bool(x > y));
                }
                None
            }
            PrimOp::IGe => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    return Some(PrimVal::Bool(x >= y));
                }
                None
            }
            PrimOp::ILt => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    return Some(PrimVal::Bool(x < y));
                }
                None
            }
            PrimOp::ILe => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    return Some(PrimVal::Bool(x <= y));
                }
                None
            }
            PrimOp::IAdd => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    return x.checked_add(*y).map(PrimVal::Int);
                }
                None
            }
            PrimOp::ISub => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    return x.checked_sub(*y).map(PrimVal::Int);
                }
                None
            }
            PrimOp::IMul => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    return x.checked_mul(*y).map(PrimVal::Int);
                }
                None
            }
            PrimOp::IDiv => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    if *y == 0 {
                        return None;
                    }
                    return Some(PrimVal::Int(x / y));
                }
                None
            }
            PrimOp::IRem => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    if *y == 0 {
                        return None;
                    }
                    return Some(PrimVal::Int(x % y));
                }
                None
            }
            PrimOp::INeg => {
                let a = args.first()?;
                if let PrimVal::Int(x) = a {
                    return Some(PrimVal::Int(-x));
                }
                None
            }
            PrimOp::FEq => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    return Some(PrimVal::Bool(x == y));
                }
                None
            }
            PrimOp::FNe => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    return Some(PrimVal::Bool(x != y));
                }
                None
            }
            PrimOp::FGt => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    return Some(PrimVal::Bool(x > y));
                }
                None
            }
            PrimOp::FGe => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    return Some(PrimVal::Bool(x >= y));
                }
                None
            }
            PrimOp::FLt => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    return Some(PrimVal::Bool(x < y));
                }
                None
            }
            PrimOp::FLe => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    return Some(PrimVal::Bool(x <= y));
                }
                None
            }
            PrimOp::FAdd => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    return Some(PrimVal::Float(x + y));
                }
                None
            }
            PrimOp::FSub => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    return Some(PrimVal::Float(x - y));
                }
                None
            }
            PrimOp::FMul => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    return Some(PrimVal::Float(x * y));
                }
                None
            }
            PrimOp::FDiv => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    if *y == 0.0 {
                        return None;
                    }
                    return Some(PrimVal::Float(x / y));
                }
                None
            }
            PrimOp::FNeg => {
                let a = args.first()?;
                if let PrimVal::Float(x) = a {
                    return Some(PrimVal::Float(-x));
                }
                None
            }
            PrimOp::Not => {
                let a = args.first()?;
                if let PrimVal::Bool(b) = a {
                    return Some(PrimVal::Bool(!b));
                }
                None
            }
            PrimOp::And => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Bool(x), PrimVal::Bool(y)) = (a, b) {
                    return Some(PrimVal::Bool(*x && *y));
                }
                None
            }
            PrimOp::Or => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Bool(x), PrimVal::Bool(y)) = (a, b) {
                    return Some(PrimVal::Bool(*x || *y));
                }
                None
            }
            PrimOp::Chr => {
                let a = args.first()?;
                if let PrimVal::Int(n) = a {
                    if let Some(c) = char::from_u32(*n as u32) {
                        return Some(PrimVal::Char(c));
                    }
                }
                None
            }
            PrimOp::Ord => {
                let a = args.first()?;
                if let PrimVal::Char(c) = a {
                    return Some(PrimVal::Int(*c as i64));
                }
                None
            }
        }
    }
}

pub fn prim_name_to_op(name: &str) -> Option<PrimOp> {
    match name {
        "prim_iadd" => Some(PrimOp::IAdd),
        "prim_isub" => Some(PrimOp::ISub),
        "prim_imul" => Some(PrimOp::IMul),
        "prim_idiv" => Some(PrimOp::IDiv),
        "prim_irem" => Some(PrimOp::IRem),
        "prim_ineg" => Some(PrimOp::INeg),
        "prim_fadd" => Some(PrimOp::FAdd),
        "prim_fsub" => Some(PrimOp::FSub),
        "prim_fmul" => Some(PrimOp::FMul),
        "prim_fdiv" => Some(PrimOp::FDiv),
        "prim_fneg" => Some(PrimOp::FNeg),
        "prim_ieq" => Some(PrimOp::IEq),
        "prim_ifeq" => Some(PrimOp::IFEq),
        "prim_igt" => Some(PrimOp::IGt),
        "prim_ige" => Some(PrimOp::IGe),
        "prim_ilt" => Some(PrimOp::ILt),
        "prim_ile" => Some(PrimOp::ILe),
        "prim_feq" => Some(PrimOp::FEq),
        "prim_fne" => Some(PrimOp::FNe),
        "prim_fgt" => Some(PrimOp::FGt),
        "prim_fge" => Some(PrimOp::FGe),
        "prim_flt" => Some(PrimOp::FLt),
        "prim_fle" => Some(PrimOp::FLe),
        "prim_not" => Some(PrimOp::Not),
        "prim_and" => Some(PrimOp::And),
        "prim_or" => Some(PrimOp::Or),
        "prim_chr" => Some(PrimOp::Chr),
        "prim_ord" => Some(PrimOp::Ord),
        _ => None,
    }
}
