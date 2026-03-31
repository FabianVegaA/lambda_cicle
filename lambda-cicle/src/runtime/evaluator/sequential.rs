use super::{EvalError, Evaluator};
use crate::core::ast::{Literal, Term};
use crate::runtime::net::{Agent, InteractionResult, Net, NodeId, PortIndex};
use crate::runtime::primitives::PrimVal;

pub struct SequentialEvaluator {
    max_steps: usize,
}

impl SequentialEvaluator {
    pub fn new() -> SequentialEvaluator {
        SequentialEvaluator {
            max_steps: 10_000_000,
        }
    }

    pub fn with_max_steps(max_steps: usize) -> SequentialEvaluator {
        SequentialEvaluator { max_steps }
    }
}

impl Default for SequentialEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for SequentialEvaluator {
    fn evaluate(&self, net: &mut Net) -> Result<Option<Term>, EvalError> {
        let mut steps = 0;

        while steps < self.max_steps {
            let result = net.step();
            steps += 1;

            if result == InteractionResult::None {
                break;
            }
        }

        if steps >= self.max_steps {
            return Err(EvalError::NonTerminating);
        }

        let result = extract_result(net);
        Ok(result)
    }
}

fn extract_result(net: &Net) -> Option<Term> {
    if net.nodes().is_empty() && net.wires().is_empty() {
        return Some(Term::NativeLiteral(Literal::Unit));
    }

    for (node_id, node) in net.nodes().iter().enumerate() {
        match &node.agent {
            Agent::PrimVal(PrimVal::Int(n)) => {
                return Some(Term::NativeLiteral(Literal::Int(*n)));
            }
            Agent::PrimVal(PrimVal::Float(f)) => {
                return Some(Term::NativeLiteral(Literal::Float(*f)));
            }
            Agent::PrimVal(PrimVal::Bool(b)) => {
                return Some(Term::NativeLiteral(Literal::Bool(*b)));
            }
            Agent::PrimVal(PrimVal::Char(c)) => {
                return Some(Term::NativeLiteral(Literal::Char(*c)));
            }
            Agent::PrimVal(PrimVal::Unit) => {
                return Some(Term::NativeLiteral(Literal::Unit));
            }
            Agent::Constructor(name) => {
                if let Ok(n) = name.parse::<i64>() {
                    return Some(Term::NativeLiteral(Literal::Int(n)));
                }
                if let Ok(b) = name.parse::<bool>() {
                    return Some(Term::NativeLiteral(Literal::Bool(b)));
                }
                if name == "()" || name == "Unit" {
                    return Some(Term::NativeLiteral(Literal::Unit));
                }
            }
            _ => {}
        }
    }

    None
}
