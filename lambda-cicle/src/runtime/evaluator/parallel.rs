use super::{EvalError, Evaluator};
use crate::core::ast::Term;
use crate::runtime::net::Net;

pub struct ParallelEvaluator;

impl ParallelEvaluator {
    pub fn new() -> ParallelEvaluator {
        ParallelEvaluator
    }
}

impl Default for ParallelEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for ParallelEvaluator {
    fn evaluate(&self, _net: &mut Net) -> Result<Option<Term>, EvalError> {
        Ok(None)
    }
}
