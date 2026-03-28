use crate::core::ast::types::Multiplicity;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    Wildcard,
    Var(String),
    Constructor(String, Vec<Pattern>),
}

impl Pattern {
    pub fn wildcard() -> Pattern {
        Pattern::Wildcard
    }

    pub fn var(name: impl Into<String>) -> Pattern {
        Pattern::Var(name.into())
    }

    pub fn constructor(name: impl Into<String>, args: Vec<Pattern>) -> Pattern {
        Pattern::Constructor(name.into(), args)
    }

    pub fn is_wildcard(&self) -> bool {
        matches!(self, Pattern::Wildcard)
    }

    pub fn bindings(&self) -> Vec<(String, Multiplicity)> {
        match self {
            Pattern::Wildcard => vec![],
            Pattern::Var(name) => vec![(name.clone(), Multiplicity::One)],
            Pattern::Constructor(_, args) => args.iter().flat_map(|p| p.bindings()).collect(),
        }
    }

    pub fn multiplicity_of(&self, var: &str) -> Option<Multiplicity> {
        match self {
            Pattern::Wildcard => None,
            Pattern::Var(name) => {
                if name == var {
                    Some(Multiplicity::One)
                } else {
                    None
                }
            }
            Pattern::Constructor(_, args) => {
                for arg in args {
                    if let Some(m) = arg.multiplicity_of(var) {
                        return Some(m);
                    }
                }
                None
            }
        }
    }

    pub fn binding_multiplicity(&self) -> Multiplicity {
        match self {
            Pattern::Wildcard => Multiplicity::Zero,
            Pattern::Var(_) => Multiplicity::One,
            Pattern::Constructor(_, _) => Multiplicity::One,
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Var(name) => write!(f, "{}", name),
            Pattern::Constructor(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(
                        f,
                        "{} ({})",
                        name,
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
