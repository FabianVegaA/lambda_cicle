use crate::core::ast::Literal;

#[derive(Debug, Clone, PartialEq)]
pub enum PrimOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Not,
    Neg,
}

impl PrimOp {
    pub fn from_ast_op(op: &crate::core::ast::BinOp) -> PrimOp {
        match op {
            crate::core::ast::BinOp::Add => PrimOp::Add,
            crate::core::ast::BinOp::Sub => PrimOp::Sub,
            crate::core::ast::BinOp::Mul => PrimOp::Mul,
            crate::core::ast::BinOp::Div => PrimOp::Div,
            crate::core::ast::BinOp::Mod => PrimOp::Mod,
            crate::core::ast::BinOp::Eq => PrimOp::Eq,
            crate::core::ast::BinOp::Ne => PrimOp::Ne,
            crate::core::ast::BinOp::Lt => PrimOp::Lt,
            crate::core::ast::BinOp::Gt => PrimOp::Gt,
            crate::core::ast::BinOp::Le => PrimOp::Le,
            crate::core::ast::BinOp::Ge => PrimOp::Ge,
            crate::core::ast::BinOp::And => PrimOp::And,
            crate::core::ast::BinOp::Or => PrimOp::Or,
        }
    }

    pub fn from_unary_op(op: &crate::core::ast::UnOp) -> PrimOp {
        match op {
            crate::core::ast::UnOp::Neg => PrimOp::Neg,
            crate::core::ast::UnOp::Not => PrimOp::Not,
        }
    }

    pub fn apply(&self, args: &[Literal]) -> Option<Literal> {
        match self {
            PrimOp::Add => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Int(x), Literal::Int(y)) = (a, b) {
                    return Some(Literal::Int(x + y));
                }
                None
            }
            PrimOp::Sub => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Int(x), Literal::Int(y)) = (a, b) {
                    return Some(Literal::Int(x - y));
                }
                None
            }
            PrimOp::Mul => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Int(x), Literal::Int(y)) = (a, b) {
                    return Some(Literal::Int(x * y));
                }
                None
            }
            PrimOp::Div => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Int(x), Literal::Int(y)) = (a, b) {
                    if *y != 0 {
                        return Some(Literal::Int(x / y));
                    }
                }
                None
            }
            PrimOp::Mod => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Int(x), Literal::Int(y)) = (a, b) {
                    if *y != 0 {
                        return Some(Literal::Int(x % y));
                    }
                }
                None
            }
            PrimOp::Eq => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                Some(Literal::Bool(a == b))
            }
            PrimOp::Ne => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                Some(Literal::Bool(a != b))
            }
            PrimOp::Lt => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Int(x), Literal::Int(y)) = (a, b) {
                    return Some(Literal::Bool(x < y));
                }
                None
            }
            PrimOp::Gt => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Int(x), Literal::Int(y)) = (a, b) {
                    return Some(Literal::Bool(x > y));
                }
                None
            }
            PrimOp::Le => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Int(x), Literal::Int(y)) = (a, b) {
                    return Some(Literal::Bool(x <= y));
                }
                None
            }
            PrimOp::Ge => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Int(x), Literal::Int(y)) = (a, b) {
                    return Some(Literal::Bool(x >= y));
                }
                None
            }
            PrimOp::And => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Bool(x), Literal::Bool(y)) = (a, b) {
                    return Some(Literal::Bool(*x && *y));
                }
                None
            }
            PrimOp::Or => {
                let a = args.get(0)?;
                let b = args.get(1)?;
                if let (Literal::Bool(x), Literal::Bool(y)) = (a, b) {
                    return Some(Literal::Bool(*x || *y));
                }
                None
            }
            PrimOp::Not => {
                let a = args.get(0)?;
                if let Literal::Bool(x) = a {
                    return Some(Literal::Bool(!x));
                }
                None
            }
            PrimOp::Neg => {
                let a = args.get(0)?;
                if let Literal::Int(x) = a {
                    return Some(Literal::Int(-x));
                }
                None
            }
        }
    }
}
