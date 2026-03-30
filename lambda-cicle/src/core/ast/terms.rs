use crate::core::ast::types::{MethodName, Multiplicity, TraitName, Type};
use crate::runtime::PrimOp;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Unit,
    Prim(PrimOp),
}

impl Literal {
    pub fn ty(&self) -> Type {
        let division_by_zero = Type::inductive("DivisionByZero".to_string(), vec![]);
        match self {
            Literal::Int(_) => Type::int(),
            Literal::Float(_) => Type::float(),
            Literal::Bool(_) => Type::bool(),
            Literal::Char(_) => Type::char(),
            Literal::Unit => Type::unit(),
            Literal::Prim(op) => match op {
                PrimOp::Neg => Type::arrow(Type::int(), Multiplicity::One, Type::int()),
                PrimOp::Add | PrimOp::Sub | PrimOp::Mul => Type::arrow(
                    Type::int(),
                    Multiplicity::One,
                    Type::arrow(Type::int(), Multiplicity::One, Type::int()),
                ),
                PrimOp::Div | PrimOp::Rem => Type::arrow(
                    Type::int(),
                    Multiplicity::One,
                    Type::arrow(
                        Type::int(),
                        Multiplicity::One,
                        Type::inductive("Result".to_string(), vec![Type::int(), division_by_zero]),
                    ),
                ),
                PrimOp::Eq | PrimOp::Ne | PrimOp::Lt | PrimOp::Gt | PrimOp::Le | PrimOp::Ge => {
                    Type::arrow(
                        Type::int(),
                        Multiplicity::One,
                        Type::arrow(Type::int(), Multiplicity::One, Type::bool()),
                    )
                }
            },
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
            Literal::Prim(op) => write!(f, "prim({:?})", op),
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
pub enum Visibility {
    Private,
    Public,
}

impl serde::Serialize for Visibility {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Visibility::Private => serializer.serialize_str("private"),
            Visibility::Public => serializer.serialize_str("public"),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Visibility {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Unexpected;
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "private" => Ok(Visibility::Private),
            "public" => Ok(Visibility::Public),
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Str(&s),
                &"private or public",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UseMode {
    Qualified,
    Selective(Vec<String>),
    Unqualified,
    Aliased(String),
}

#[allow(dead_code)]
impl serde::Serialize for UseMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("UseMode")
    }
}

#[allow(dead_code)]
impl<'de> serde::Deserialize<'de> for UseMode {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(UseMode::Qualified)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    TypeDecl {
        visibility: Visibility,
        name: String,
        params: Vec<String>,
        ty: Type,
        transparent: bool,
    },
    ValDecl {
        visibility: Visibility,
        name: String,
        ty: Type,
        term: Box<Term>,
    },
    TraitDecl {
        visibility: Visibility,
        name: String,
        params: Vec<String>,
        methods: Vec<MethodSig>,
    },
    ImplDecl {
        ty: Type,
        trait_name: TraitName,
        methods: Vec<MethodDef>,
    },
    UseDecl {
        path: Vec<String>,
        mode: UseMode,
    },
    NoPrelude,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodSig {
    pub name: MethodName,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MethodDef {
    pub name: MethodName,
    pub term: Box<Term>,
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

    pub fn get_type(&self) -> Option<Type> {
        match self {
            Term::Var(_) => None,
            Term::Abs { annot, body, .. } => Some(Type::arrow(
                annot.clone(),
                Multiplicity::One,
                body.get_type().unwrap_or(Type::unit()),
            )),
            Term::App { fun, .. } => fun.get_type().and_then(|t| {
                if let Type::Arrow(_, _, ret) = t {
                    Some(*ret)
                } else {
                    None
                }
            }),
            Term::Let { annot, body, .. } => {
                let mut ty = annot.clone();
                if let Some(body_type) = body.get_type() {
                    ty = body_type;
                }
                Some(ty)
            }
            Term::Match { arms, .. } => arms.first().and_then(|a| a.body.get_type()),
            Term::View { arms, .. } => arms.first().and_then(|a| a.body.get_type()),
            Term::TraitMethod { .. } => None,
            Term::Constructor(_, _) => None,
            Term::NativeLiteral(lit) => Some(lit.ty()),
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
