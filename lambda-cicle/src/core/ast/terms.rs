use crate::core::ast::types::{MethodName, Multiplicity, TraitName, Type};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Unit,
}

impl Literal {
    pub fn ty(&self) -> Type {
        match self {
            Literal::Int(_) => Type::int(),
            Literal::Float(_) => Type::float(),
            Literal::Bool(_) => Type::bool(),
            Literal::Char(_) => Type::char(),
            Literal::Unit => Type::unit(),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Int(n) => write!(f, "{}", n),
            Literal::Float(n) => write!(f, "{}", n),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Char(c) => write!(f, "'{}'", c),
            Literal::Unit => write!(f, "()"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BinOp {
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
}

impl BinOp {
    pub fn from_symbol(s: &str) -> Option<BinOp> {
        match s {
            "+" => Some(BinOp::Add),
            "-" => Some(BinOp::Sub),
            "*" => Some(BinOp::Mul),
            "/" => Some(BinOp::Div),
            "%" => Some(BinOp::Mod),
            "==" => Some(BinOp::Eq),
            "!=" => Some(BinOp::Ne),
            "<" => Some(BinOp::Lt),
            ">" => Some(BinOp::Gt),
            "<=" => Some(BinOp::Le),
            ">=" => Some(BinOp::Ge),
            "&&" => Some(BinOp::And),
            "||" => Some(BinOp::Or),
            _ => None,
        }
    }

    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::Or => 1,
            BinOp::And => 2,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => 3,
            BinOp::Add | BinOp::Sub => 4,
            BinOp::Mul | BinOp::Div | BinOp::Mod => 5,
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnOp {
    Neg,
    Not,
}

impl UnOp {
    pub fn from_symbol(s: &str) -> Option<UnOp> {
        match s {
            "-" => Some(UnOp::Neg),
            "!" => Some(UnOp::Not),
            _ => None,
        }
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub pattern: super::Pattern,
    pub body: Box<Term>,
}

impl Arm {
    pub fn new(pattern: super::Pattern, body: Term) -> Arm {
        Arm {
            pattern,
            body: Box::new(body),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(String),
    Abs {
        var: String,
        multiplicity: Multiplicity,
        annot: Type,
        body: Box<Term>,
    },
    App {
        fun: Box<Term>,
        arg: Box<Term>,
    },
    Let {
        var: String,
        multiplicity: Multiplicity,
        annot: Type,
        value: Box<Term>,
        body: Box<Term>,
    },
    Match {
        scrutinee: Box<Term>,
        arms: Vec<Arm>,
    },
    View {
        scrutinee: Box<Term>,
        arms: Vec<Arm>,
    },
    TraitMethod {
        trait_name: TraitName,
        method: MethodName,
        arg: Box<Term>,
    },
    Constructor(String, Vec<Term>),
    NativeLiteral(Literal),
    BinaryOp {
        op: BinOp,
        left: Box<Term>,
        right: Box<Term>,
    },
    UnaryOp {
        op: UnOp,
        arg: Box<Term>,
    },
}

impl Term {
    pub fn var(name: impl Into<String>) -> Term {
        Term::Var(name.into())
    }

    pub fn abs(var: impl Into<String>, mult: Multiplicity, annot: Type, body: Term) -> Term {
        Term::Abs {
            var: var.into(),
            multiplicity: mult,
            annot,
            body: Box::new(body),
        }
    }

    pub fn app(fun: Term, arg: Term) -> Term {
        Term::App {
            fun: Box::new(fun),
            arg: Box::new(arg),
        }
    }

    pub fn let_in(
        var: impl Into<String>,
        mult: Multiplicity,
        annot: Type,
        value: Term,
        body: Term,
    ) -> Term {
        Term::Let {
            var: var.into(),
            multiplicity: mult,
            annot,
            value: Box::new(value),
            body: Box::new(body),
        }
    }

    pub fn match_on(scrutinee: Term, arms: Vec<Arm>) -> Term {
        Term::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        }
    }

    pub fn view_on(scrutinee: Term, arms: Vec<Arm>) -> Term {
        Term::View {
            scrutinee: Box::new(scrutinee),
            arms,
        }
    }

    pub fn constructor(name: impl Into<String>, args: Vec<Term>) -> Term {
        Term::Constructor(name.into(), args)
    }

    pub fn literal(lit: Literal) -> Term {
        Term::NativeLiteral(lit)
    }

    pub fn binary(op: BinOp, left: Term, right: Term) -> Term {
        Term::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    pub fn unary(op: UnOp, arg: Term) -> Term {
        Term::UnaryOp {
            op,
            arg: Box::new(arg),
        }
    }

    pub fn trait_method(trait_name: TraitName, method: MethodName, arg: Term) -> Term {
        Term::TraitMethod {
            trait_name,
            method,
            arg: Box::new(arg),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Var(name) => write!(f, "{}", name),
            Term::Abs {
                var,
                multiplicity,
                annot,
                body,
            } => {
                write!(f, "λ{}:{}:{}. {}", var, multiplicity, annot, body)
            }
            Term::App { fun, arg } => {
                write!(f, "({} {})", fun, arg)
            }
            Term::Let {
                var,
                multiplicity,
                annot,
                value,
                body,
            } => {
                write!(
                    f,
                    "let {}:{}{} = {} in {}",
                    var, multiplicity, annot, value, body
                )
            }
            Term::Match { scrutinee, arms } => {
                write!(
                    f,
                    "match {} with {{ {} }}",
                    scrutinee,
                    arms.iter()
                        .map(|a| format!("{} -> {}", a.pattern, a.body))
                        .collect::<Vec<_>>()
                        .join(" | ")
                )
            }
            Term::View { scrutinee, arms } => {
                write!(
                    f,
                    "view {} with {{ {} }}",
                    scrutinee,
                    arms.iter()
                        .map(|a| format!("{} -> {}", a.pattern, a.body))
                        .collect::<Vec<_>>()
                        .join(" | ")
                )
            }
            Term::TraitMethod {
                trait_name,
                method,
                arg,
            } => {
                write!(f, "({}.{} {})", trait_name, method, arg)
            }
            Term::Constructor(name, args) => {
                write!(
                    f,
                    "{}({})",
                    name,
                    args.iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Term::NativeLiteral(lit) => write!(f, "{}", lit),
            Term::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }
            Term::UnaryOp { op, arg } => {
                write!(f, "({}{})", op, arg)
            }
        }
    }
}
