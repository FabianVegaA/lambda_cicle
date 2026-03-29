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
        }
    }
}
