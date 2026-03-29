pub mod bench;
pub mod dot;
pub mod repl;
pub mod trace;

pub use bench::run_benchmark;
pub use dot::net_to_dot;
pub use repl::run_repl;
pub use trace::TraceDebugger;

use crate::core::parser::ParseError;
use crate::core::typecheck::TypeError;
use crate::runtime::evaluator::{Evaluator, SequentialEvaluator};
use crate::{parse, translate, type_check_with_borrow_check};

#[derive(Debug)]
pub enum ToolError {
    Parse(ParseError),
    Typecheck(TypeError),
    Eval(crate::runtime::evaluator::EvalError),
    Io(std::io::Error),
}

impl From<ParseError> for ToolError {
    fn from(e: ParseError) -> Self {
        ToolError::Parse(e)
    }
}

impl From<TypeError> for ToolError {
    fn from(e: TypeError) -> Self {
        ToolError::Typecheck(e)
    }
}

impl From<crate::runtime::evaluator::EvalError> for ToolError {
    fn from(e: crate::runtime::evaluator::EvalError) -> Self {
        ToolError::Eval(e)
    }
}

impl From<std::io::Error> for ToolError {
    fn from(e: std::io::Error) -> Self {
        ToolError::Io(e)
    }
}

pub fn compile_source(
    source: &str,
) -> Result<
    (
        crate::core::ast::Term,
        crate::core::ast::Type,
        crate::runtime::net::Net,
    ),
    ToolError,
> {
    let term = parse(source)?;
    let ty = type_check_with_borrow_check(&term)?;
    let net = translate(&term);
    Ok((term, ty, net))
}

pub fn run_source(source: &str) -> Result<Option<crate::core::ast::Term>, ToolError> {
    let (_, _, mut net) = compile_source(source)?;
    let evaluator = SequentialEvaluator::new();
    evaluator.evaluate(&mut net).map_err(ToolError::from)
}
