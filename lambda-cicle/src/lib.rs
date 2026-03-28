pub mod core;

pub use core::ast::{Pattern, Term, Type};
pub use core::parser::parse;
pub use core::typecheck::type_check_with_borrow_check;
