pub mod coherence;
pub mod registry;
pub mod resolution;

pub use coherence::{check_coherence, check_orphan_rule, DefiningModules};
pub use registry::{Implementation, Registry};
pub use resolution::resolve_method;

use crate::core::ast::{MethodName, TraitName, Type};

#[derive(Debug, Clone)]
pub enum TraitError {
    TraitNotFound(TraitName, Type),
    DuplicateImpl(TraitName, Type),
    CoherenceViolation(String),
    MethodNotFound(TraitName, MethodName),
    OrphanImpl {
        impl_module: String,
        trait_name: TraitName,
        type_name: String,
    },
}

impl std::fmt::Display for TraitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraitError::TraitNotFound(t, ty) => write!(f, "Trait {} not found for type {}", t, ty),
            TraitError::DuplicateImpl(t, ty) => {
                write!(f, "Duplicate implementation of {} for {}", t, ty)
            }
            TraitError::CoherenceViolation(msg) => write!(f, "Coherence violation: {}", msg),
            TraitError::MethodNotFound(t, m) => write!(f, "Method {} not found in trait {}", m, t),
            TraitError::OrphanImpl {
                impl_module,
                trait_name,
                type_name,
            } => write!(
                f,
                "Orphan impl: impl {} {} is in module '{}', but neither defines trait '{}' nor type '{}'",
                trait_name, type_name, impl_module, trait_name, type_name
            ),
        }
    }
}

impl std::error::Error for TraitError {}
