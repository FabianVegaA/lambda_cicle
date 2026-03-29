mod sequential;
mod parallel;

pub use sequential::SequentialEvaluator;
pub use parallel::ParallelEvaluator;

use crate::runtime::net::Net;
use crate::core::ast::Term;

pub trait Evaluator {
    fn evaluate(&self, net: &mut Net) -> Result<Option<Term>, EvalError>;
}

#[derive(Debug)]
pub enum EvalError {
    NonTerminating,
    Stuck(String),
    TypeError(String),
}
