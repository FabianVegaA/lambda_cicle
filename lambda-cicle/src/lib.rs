pub mod core;
pub mod modules;
pub mod runtime;
pub mod tools;
pub mod traits;

pub use core::ast::{Decl, Pattern, Term, Type, Visibility};
pub use core::parser::{parse, parse_program, ParseError};
pub use core::typecheck::{type_check_with_borrow_check, TypeError};
pub use modules::{link, load_module, Exports, Module, ModuleError};
pub use runtime::evaluator::{Evaluator, ParallelEvaluator, SequentialEvaluator};
pub use runtime::translate;
pub use tools::bench::run_benchmark;
pub use traits::{check_coherence, resolve_method, Implementation, Registry, TraitError};

#[derive(Debug)]
pub enum PipelineError {
    Parse(ParseError),
    Typecheck(TypeError),
    Eval(runtime::evaluator::EvalError),
}

pub fn run_sequential(source: &str) -> Result<Option<Term>, PipelineError> {
    let term = parse(source).map_err(PipelineError::Parse)?;
    let _ty = type_check_with_borrow_check(&term).map_err(PipelineError::Typecheck)?;
    let mut net = translate(&term);
    let evaluator = SequentialEvaluator::new();
    evaluator.evaluate(&mut net).map_err(PipelineError::Eval)
}

pub fn run_parallel(source: &str) -> Result<Option<Term>, PipelineError> {
    let term = parse(source).map_err(PipelineError::Parse)?;
    let _ty = type_check_with_borrow_check(&term).map_err(PipelineError::Typecheck)?;
    let mut net = translate(&term);
    let evaluator = ParallelEvaluator::new();
    evaluator.evaluate(&mut net).map_err(PipelineError::Eval)
}
