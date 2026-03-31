use crate::core::ast::types::{Multiplicity, TraitName, Type, TypeName};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Linearity violation: variable {0} used with multiplicity {1}")]
    LinearityViolation(String, Multiplicity),

    #[error("Cannot mix borrow context with quantity context")]
    BorrowContextMix,

    #[error("Multiplicity mismatch: expected {expected}, found {found}")]
    MultiplicityMismatch {
        expected: Multiplicity,
        found: Multiplicity,
    },

    #[error("Ownership escape: {0}")]
    OwnershipEscape(String),

    #[error("Trait not found: {0} for type {1}")]
    TraitNotFound(TraitName, Type),

    #[error("Duplicate implementation of trait {0} for type {1}")]
    DuplicateImpl(TraitName, Type),

    #[error("Non-exhaustive pattern: missing cases")]
    NonExhaustivePattern,

    #[error("Strict positivity violation in inductive type {0}")]
    StrictPositivityViolation(TypeName),

    #[error("Cannot use borrow in match arm: {0}")]
    BorrowInMatchArm(String),

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: Type, found: Type },

    #[error("Unknown variable: {0}")]
    UnknownVariable(String),

    #[error("Invalid application: {0}")]
    InvalidApplication(String),

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("Unknown primitive: {0}")]
    UnknownPrimitive(String),
}
