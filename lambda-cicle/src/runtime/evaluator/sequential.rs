use super::{EvalError, Evaluator};
use crate::core::ast::{Literal, Term};
use crate::runtime::net::{Agent, Net, NodeId, PortIndex};

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
            if net.is_stuck() {
                break;
            }

            net.step();
            steps += 1;
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

    for (id, node) in net.nodes().iter().enumerate() {
        if matches!(node.agent, Agent::Constructor(_) | Agent::Prim(_)) {
            let mut has_free_ports = false;
            for port_idx in 0..node.num_ports() {
                if net
                    .get_connected_port(NodeId(id), PortIndex(port_idx))
                    .is_none()
                {
                    has_free_ports = true;
                    break;
                }
            }
            if !has_free_ports && node.num_ports() == 0 {
                match &node.agent {
                    Agent::Constructor(name) => {
                        return Some(Term::Var(name.clone()));
                    }
                    Agent::Prim(_) => {
                        return Some(Term::NativeLiteral(Literal::Unit));
                    }
                    _ => {}
                }
            }
        }
    }

    None
}
