use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NativeKind {
    Int,
    Float,
    Bool,
    Char,
    Unit,
}

impl fmt::Display for NativeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NativeKind::Int => write!(f, "Int"),
            NativeKind::Float => write!(f, "Float"),
            NativeKind::Bool => write!(f, "Bool"),
            NativeKind::Char => write!(f, "Char"),
            NativeKind::Unit => write!(f, "Unit"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Multiplicity {
    Zero,
    One,
    Omega,
    Borrow,
}

impl Multiplicity {
    pub fn from_symbol(s: &str) -> Option<Multiplicity> {
        match s {
            "0" => Some(Multiplicity::Zero),
            "1" => Some(Multiplicity::One),
            "ω" | "omega" => Some(Multiplicity::Omega),
            "&" => Some(Multiplicity::Borrow),
            _ => None,
        }
    }

    pub fn is_quantity(&self) -> bool {
        matches!(
            self,
            Multiplicity::Zero | Multiplicity::One | Multiplicity::Omega
        )
    }

    pub fn is_borrow(&self) -> bool {
        matches!(self, Multiplicity::Borrow)
    }
}

impl fmt::Display for Multiplicity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Multiplicity::Zero => write!(f, "0"),
            Multiplicity::One => write!(f, "1"),
            Multiplicity::Omega => write!(f, "ω"),
            Multiplicity::Borrow => write!(f, "&"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraitName(pub String);

impl TraitName {
    pub fn new(name: impl Into<String>) -> TraitName {
        TraitName(name.into())
    }
}

impl fmt::Display for TraitName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeName(pub String);

impl TypeName {
    pub fn new(name: impl Into<String>) -> TypeName {
        TypeName(name.into())
    }
}

impl fmt::Display for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MethodName(pub String);

impl MethodName {
    pub fn new(name: impl Into<String>) -> MethodName {
        MethodName(name.into())
    }
}

impl fmt::Display for MethodName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    Native(NativeKind),
    Arrow(Multiplicity, Box<Type>, Box<Type>),
    Forall(String, Box<Type>),
    TraitConstraint(TraitName, Box<Type>),
    Inductive(TypeName, Vec<Type>),
    Borrow(Box<Type>),
    Product(Box<Type>, Box<Type>),
    Sum(Box<Type>, Box<Type>),
    Var(String),
    App(Box<Type>, Vec<Type>),
}

impl Type {
    pub fn arrow(arg: Type, mult: Multiplicity, ret: Type) -> Type {
        Type::Arrow(mult, Box::new(arg), Box::new(ret))
    }

    pub fn for_all(name: impl Into<String>, ty: Type) -> Type {
        Type::Forall(name.into(), Box::new(ty))
    }

    pub fn trait_constraint(trait_name: TraitName, ty: Type) -> Type {
        Type::TraitConstraint(trait_name, Box::new(ty))
    }

    pub fn inductive(name: impl Into<String>, params: Vec<Type>) -> Type {
        Type::Inductive(TypeName::new(name), params)
    }

    pub fn borrow(ty: Type) -> Type {
        Type::Borrow(Box::new(ty))
    }

    pub fn product(left: Type, right: Type) -> Type {
        Type::Product(Box::new(left), Box::new(right))
    }

    pub fn sum(left: Type, right: Type) -> Type {
        Type::Sum(Box::new(left), Box::new(right))
    }

    pub fn native(kind: NativeKind) -> Type {
        Type::Native(kind)
    }

    pub fn unit() -> Type {
        Type::Native(NativeKind::Unit)
    }

    pub fn bool() -> Type {
        Type::Native(NativeKind::Bool)
    }

    pub fn int() -> Type {
        Type::Native(NativeKind::Int)
    }

    pub fn float() -> Type {
        Type::Native(NativeKind::Float)
    }

    pub fn char() -> Type {
        Type::Native(NativeKind::Char)
    }

    pub fn app(ty: Type, args: Vec<Type>) -> Type {
        Type::App(Box::new(ty), args)
    }

    /// Check if a type contains type variables (is polymorphic)
    pub fn is_polymorphic(&self) -> bool {
        match self {
            Type::Var(_) => true,
            Type::Forall(_, _) => true,
            Type::Arrow(_, arg, ret) => arg.is_polymorphic() || ret.is_polymorphic(),
            Type::TraitConstraint(_, ty) => ty.is_polymorphic(),
            Type::Inductive(_, params) => params.iter().any(|p| p.is_polymorphic()),
            Type::Borrow(ty) => ty.is_polymorphic(),
            Type::Product(left, right) => left.is_polymorphic() || right.is_polymorphic(),
            Type::Sum(left, right) => left.is_polymorphic() || right.is_polymorphic(),
            Type::App(ty, args) => ty.is_polymorphic() || args.iter().any(|a| a.is_polymorphic()),
            Type::Native(_) => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Native(kind) => write!(f, "{}", kind),
            Type::Arrow(mult, arg, ret) => {
                write!(f, "({} →^{} {})", arg, mult, ret)
            }
            Type::Forall(name, ty) => {
                write!(f, "∀{}. {}", name, ty)
            }
            Type::TraitConstraint(trait_name, ty) => {
                write!(f, "{} {}", trait_name, ty)
            }
            Type::Inductive(name, params) => {
                if params.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(
                        f,
                        "{} {}",
                        name,
                        params
                            .iter()
                            .map(|p| p.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                }
            }
            Type::Borrow(ty) => {
                write!(f, "&{}", ty)
            }
            Type::Product(left, right) => {
                write!(f, "({}, {})", left, right)
            }
            Type::Sum(left, right) => {
                write!(f, "{} + {}", left, right)
            }
            Type::Var(name) => {
                write!(f, "{}", name)
            }
            Type::App(ty, args) => {
                if args.is_empty() {
                    write!(f, "{}", ty)
                } else {
                    write!(
                        f,
                        "{}<{}>",
                        ty,
                        args.iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
        }
    }
}
