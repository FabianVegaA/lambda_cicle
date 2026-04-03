use crate::runtime::primitives::{IOOp, PrimVal};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimOp {
    IAdd,
    ISub,
    IMul,
    IDiv,
    IRem,
    INeg,
    IHash,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FRem,
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
    BNot,
    And,
    BAnd,
    Or,
    BOr,
    Chr,
    Ord,
    CEq,
    COrd,
    CHash,
    BEq,
    BHash,
    IntToString,
    FloatToString,
    CharToString,
}

impl PrimOp {
    pub fn arity(&self) -> usize {
        match self {
            PrimOp::INeg
            | PrimOp::FNeg
            | PrimOp::Not
            | PrimOp::BNot
            | PrimOp::Chr
            | PrimOp::Ord
            | PrimOp::IHash
            | PrimOp::CEq
            | PrimOp::COrd
            | PrimOp::CHash
            | PrimOp::BEq
            | PrimOp::BHash
            | PrimOp::IntToString
            | PrimOp::FloatToString
            | PrimOp::CharToString => 1,
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
                        return Some(PrimVal::Constructor(
                            "Err".to_string(),
                            vec![PrimVal::Constructor("DivisionByZero".to_string(), vec![])],
                        ));
                    }
                    return Some(PrimVal::Constructor(
                        "Ok".to_string(),
                        vec![PrimVal::Int(x / y)],
                    ));
                }
                None
            }
            PrimOp::IRem => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Int(x), PrimVal::Int(y)) = (a, b) {
                    if *y == 0 {
                        return Some(PrimVal::Constructor(
                            "Err".to_string(),
                            vec![PrimVal::Constructor("DivisionByZero".to_string(), vec![])],
                        ));
                    }
                    return Some(PrimVal::Constructor(
                        "Ok".to_string(),
                        vec![PrimVal::Int(x % y)],
                    ));
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
                        return Some(PrimVal::Constructor(
                            "Err".to_string(),
                            vec![PrimVal::Constructor("DivisionByZero".to_string(), vec![])],
                        ));
                    }
                    return Some(PrimVal::Constructor(
                        "Ok".to_string(),
                        vec![PrimVal::Float(x / y)],
                    ));
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
            PrimOp::IntToString => {
                let a = args.first()?;
                if let PrimVal::Int(n) = a {
                    return Some(PrimVal::String(format!("{}", n)));
                }
                None
            }
            PrimOp::FloatToString => {
                let a = args.first()?;
                if let PrimVal::Float(f) = a {
                    return Some(PrimVal::String(format!("{}", f)));
                }
                None
            }
            PrimOp::CharToString => {
                let a = args.first()?;
                if let PrimVal::Char(c) = a {
                    return Some(PrimVal::String(c.to_string()));
                }
                None
            }
            PrimOp::IHash => {
                let a = args.first()?;
                if let PrimVal::Int(x) = a {
                    return Some(PrimVal::Int(*x));
                }
                None
            }
            PrimOp::FRem => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Float(x), PrimVal::Float(y)) = (a, b) {
                    if *y == 0.0 {
                        return Some(PrimVal::Constructor(
                            "Err".to_string(),
                            vec![PrimVal::Constructor("DivisionByZero".to_string(), vec![])],
                        ));
                    }
                    return Some(PrimVal::Constructor(
                        "Ok".to_string(),
                        vec![PrimVal::Float(x % y)],
                    ));
                }
                None
            }
            PrimOp::BNot => {
                let a = args.first()?;
                if let PrimVal::Bool(b) = a {
                    return Some(PrimVal::Bool(!*b));
                }
                None
            }
            PrimOp::BAnd => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Bool(x), PrimVal::Bool(y)) = (a, b) {
                    return Some(PrimVal::Bool(*x && *y));
                }
                None
            }
            PrimOp::BOr => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Bool(x), PrimVal::Bool(y)) = (a, b) {
                    return Some(PrimVal::Bool(*x || *y));
                }
                None
            }
            PrimOp::CEq => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Char(x), PrimVal::Char(y)) = (a, b) {
                    return Some(PrimVal::Bool(x == y));
                }
                None
            }
            PrimOp::COrd => {
                let a = args.first()?;
                if let PrimVal::Char(c) = a {
                    let ord = if (*c as i64) < 0 {
                        -1
                    } else if (*c as i64) > 0 {
                        1
                    } else {
                        0
                    };
                    return Some(PrimVal::Int(ord));
                }
                None
            }
            PrimOp::CHash => {
                let a = args.first()?;
                if let PrimVal::Char(c) = a {
                    return Some(PrimVal::Int(*c as i64));
                }
                None
            }
            PrimOp::BEq => {
                let a = args.first()?;
                let b = args.get(1)?;
                if let (PrimVal::Bool(x), PrimVal::Bool(y)) = (a, b) {
                    return Some(PrimVal::Bool(x == y));
                }
                None
            }
            PrimOp::BHash => {
                let a = args.first()?;
                if let PrimVal::Bool(b) = a {
                    return Some(PrimVal::Int(if *b { 1 } else { 0 }));
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
        "prim_ihash" => Some(PrimOp::IHash),
        "prim_fadd" => Some(PrimOp::FAdd),
        "prim_fsub" => Some(PrimOp::FSub),
        "prim_fmul" => Some(PrimOp::FMul),
        "prim_fdiv" => Some(PrimOp::FDiv),
        "prim_frem" => Some(PrimOp::FRem),
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
        "prim_bnot" => Some(PrimOp::BNot),
        "prim_band" => Some(PrimOp::BAnd),
        "prim_bor" => Some(PrimOp::BOr),
        "prim_chr" => Some(PrimOp::Chr),
        "prim_ord" => Some(PrimOp::Ord),
        "prim_ceq" => Some(PrimOp::CEq),
        "prim_cord" => Some(PrimOp::COrd),
        "prim_chash" => Some(PrimOp::CHash),
        "prim_beq" => Some(PrimOp::BEq),
        "prim_bhash" => Some(PrimOp::BHash),
        "prim_int_to_string" => Some(PrimOp::IntToString),
        "prim_float_to_string" => Some(PrimOp::FloatToString),
        "prim_char_to_string" => Some(PrimOp::CharToString),
        _ => None,
    }
}

pub fn prim_name_to_io_op(name: &str) -> Option<IOOp> {
    match name {
        "prim_io_print" => Some(IOOp::Print),
        "prim_io_println" => Some(IOOp::Println),
        "prim_io_eprint" => Some(IOOp::EPrint),
        "prim_io_eprintln" => Some(IOOp::EPrintln),
        "prim_io_read_line" => Some(IOOp::ReadLine),
        "prim_io_open" => Some(IOOp::Open),
        "prim_io_close" => Some(IOOp::Close),
        "prim_io_read" => Some(IOOp::Read),
        "prim_io_write" => Some(IOOp::Write),
        _ => None,
    }
}
