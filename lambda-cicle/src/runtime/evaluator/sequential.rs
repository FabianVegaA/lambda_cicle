use super::{EvalError, Evaluator};
use crate::core::ast::Term;
use crate::runtime::net::Net;

pub struct SequentialEvaluator;

impl SequentialEvaluator {
    pub fn new() -> SequentialEvaluator {
        SequentialEvaluator
    }
}

impl Default for SequentialEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for SequentialEvaluator {
    fn evaluate(&self, _net: &mut Net) -> Result<Option<Term>, EvalError> {
        Ok(None)
    }
}
