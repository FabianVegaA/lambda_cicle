use crate::core::ast::Literal;

#[derive(Debug, Clone, PartialEq)]
pub enum PrimOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
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
        }
    }
}
