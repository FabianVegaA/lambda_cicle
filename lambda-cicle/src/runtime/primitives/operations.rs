use crate::core::ast::Literal;

#[derive(Debug, Clone, PartialEq)]
pub enum PrimOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Neg,
}

impl PrimOp {
    pub fn apply(&self, args: &[Literal]) -> Option<Literal> {
        match self {
            PrimOp::Eq => {
                let a = args.first()?;
                let b = args.get(1)?;
                Some(Literal::Bool(a == b))
            }
            PrimOp::Ne => {
                let a = args.first()?;
                let b = args.get(1)?;
                Some(Literal::Bool(a != b))
            }
            PrimOp::Lt => {
                if let (Literal::Int(x), Literal::Int(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Bool(x < y));
                }
                None
            }
            PrimOp::Gt => {
                if let (Literal::Int(x), Literal::Int(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Bool(x > y));
                }
                None
            }
            PrimOp::Le => {
                if let (Literal::Int(x), Literal::Int(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Bool(x <= y));
                }
                None
            }
            PrimOp::Ge => {
                if let (Literal::Int(x), Literal::Int(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Bool(x >= y));
                }
                None
            }
            PrimOp::Add => {
                if let (Literal::Int(x), Literal::Int(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Int(x + y));
                }
                if let (Literal::Float(x), Literal::Float(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Float(x + y));
                }
                None
            }
            PrimOp::Sub => {
                if let (Literal::Int(x), Literal::Int(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Int(x - y));
                }
                if let (Literal::Float(x), Literal::Float(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Float(x - y));
                }
                None
            }
            PrimOp::Mul => {
                if let (Literal::Int(x), Literal::Int(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Int(x * y));
                }
                if let (Literal::Float(x), Literal::Float(y)) = (args.first()?, args.get(1)?) {
                    return Some(Literal::Float(x * y));
                }
                None
            }
            PrimOp::Div => {
                if let (Literal::Int(x), Literal::Int(y)) = (args.first()?, args.get(1)?) {
                    if *y == 0 {
                        return None;
                    }
                    return Some(Literal::Int(x / y));
                }
                if let (Literal::Float(x), Literal::Float(y)) = (args.first()?, args.get(1)?) {
                    if *y == 0.0 {
                        return None;
                    }
                    return Some(Literal::Float(x / y));
                }
                None
            }
            PrimOp::Rem => {
                if let (Literal::Int(x), Literal::Int(y)) = (args.first()?, args.get(1)?) {
                    if *y == 0 {
                        return None;
                    }
                    return Some(Literal::Int(x % y));
                }
                if let (Literal::Float(x), Literal::Float(y)) = (args.first()?, args.get(1)?) {
                    if *y == 0.0 {
                        return None;
                    }
                    return Some(Literal::Float(x % y));
                }
                None
            }
            PrimOp::Neg => {
                let a = args.first()?;
                match a {
                    Literal::Int(x) => Some(Literal::Int(-x)),
                    Literal::Float(x) => Some(Literal::Float(-x)),
                    _ => None,
                }
            }
        }
    }
}
