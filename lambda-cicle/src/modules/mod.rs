pub mod export;
pub mod linker;
pub mod loader;
pub mod serializer;

pub use export::Exports;
pub use linker::link;
pub use loader::{
    compile_module, elaborate_declarations, inject_prelude, load_module, parse_module_file,
};
pub use serializer::{deserialize_module, get_export_hash, serialize_module};

use crate::runtime::net::Net;
use crate::traits::Implementation;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub exports: Exports,
    pub impls: Vec<Implementation>,
    pub net: Net,
}

#[derive(Debug, Clone, Default)]
pub struct ModuleError {
    pub message: String,
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module error: {}", self.message)
    }
}

impl std::error::Error for ModuleError {}

impl From<crate::core::parser::ParseError> for ModuleError {
    fn from(e: crate::core::parser::ParseError) -> Self {
        ModuleError {
            message: format!("Parse error: {}", e),
        }
    }
}

impl From<crate::core::typecheck::TypeError> for ModuleError {
    fn from(e: crate::core::typecheck::TypeError) -> Self {
        ModuleError {
            message: format!("Type error: {}", e),
        }
    }
}

impl From<crate::traits::TraitError> for ModuleError {
    fn from(e: crate::traits::TraitError) -> Self {
        ModuleError {
            message: format!("Trait error: {}", e),
        }
    }
}

impl From<crate::runtime::evaluator::EvalError> for ModuleError {
    fn from(e: crate::runtime::evaluator::EvalError) -> Self {
        ModuleError {
            message: format!("Evaluation error: {:?}", e),
        }
    }
}

impl From<std::io::Error> for ModuleError {
    fn from(e: std::io::Error) -> Self {
        ModuleError {
            message: format!("IO error: {}", e),
        }
    }
}
