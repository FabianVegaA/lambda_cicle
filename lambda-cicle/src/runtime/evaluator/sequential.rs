use super::{EvalError, Evaluator};
use crate::core::ast::{Literal, Term};
use crate::runtime::net::{Agent, InteractionResult, Net, NodeId, PortIndex};
use crate::runtime::primitives::PrimVal;
use crate::runtime::Node;

#[derive(Debug)]
pub enum ExtractionError {
    UnsupportedPrimitiveValue(Agent),
    NoResult,
}

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

    pub fn evaluate_with_debug(&self, net: &mut Net, debug_level: u8) -> Result<Term, EvalError> {
        wire_io_entry_point(net);

        let mut steps = 0;

        while steps < self.max_steps {
            let result = net.step();
            steps += 1;

            debug_log_step(steps, &result, net, debug_level);

            if result == InteractionResult::None {
                break;
            }
        }

        if steps >= self.max_steps {
            return Err(EvalError::NonTerminating);
        }

        extract_result(net).map_err(|err| EvalError::EvaluationError(format!("{:?}", err)))
    }
}

fn debug_log_step(step: usize, result: &InteractionResult, net: &Net, level: u8) {
    if level == 0 {
        return;
    }

    let interaction_name = match result {
        InteractionResult::BetaReduction => "β-reduction",
        InteractionResult::Duplication => "δ-duplication",
        InteractionResult::Erasure => "ε-erasure",
        InteractionResult::Commute => "δ-commute",
        InteractionResult::EraseBranch => "δ-erase-branch",
        InteractionResult::PrimEval => "prim-eval",
        InteractionResult::PrimValErase => "prim-val-erase",
        InteractionResult::PrimValDup => "prim-val-dup",
        InteractionResult::None => "none",
    };

    eprintln!("  {:>3}: {}", step, interaction_name);

    if level >= 2 {
        eprintln!(
            "       nodes: {}, wires: {}",
            net.nodes().len(),
            net.wires().len()
        );
    }

    if level >= 3 {
        eprintln!("{:#?}", net);
    }
}

impl Default for SequentialEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Evaluator for SequentialEvaluator {
    fn evaluate(&self, net: &mut Net) -> Result<Term, EvalError> {
        wire_io_entry_point(net);

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

        extract_result(net).map_err(|err| EvalError::EvaluationError(format!("{:?}", err)))
    }
}

fn wire_io_entry_point(net: &mut Net) {
    let Some((primio_id, _)) = net
        .nodes()
        .iter()
        .enumerate()
        .find(|(_, n)| matches!(n.agent, Agent::PrimIO(_)))
    else {
        return;
    };

    let primio_id = NodeId(primio_id);

    if net.get_wire_at_port(primio_id, PortIndex(1)).is_some() {
        return;
    }

    let token = Node::io_token();
    let token_id = net.add_node(token);
    net.connect(token_id, PortIndex(0), primio_id, PortIndex(1));
}

fn extract_result(net: &Net) -> Result<Term, ExtractionError> {
    if net.nodes().is_empty() && net.wires().is_empty() {
        return Ok(Term::NativeLiteral(Literal::Unit));
    }

    for (node_id, node) in net.nodes().iter().enumerate() {
        match &node.agent {
            Agent::PrimVal(PrimVal::Constructor(name, args)) => {
                let term_args: Vec<Term> = args
                    .iter()
                    .filter_map(|arg| prim_val_to_term(arg))
                    .collect();
                return Ok(Term::Constructor(name.clone(), term_args, None));
            }
            Agent::PrimVal(PrimVal::Int(n)) => {
                return Ok(Term::NativeLiteral(Literal::Int(*n)));
            }
            Agent::PrimVal(PrimVal::Float(f)) => {
                return Ok(Term::NativeLiteral(Literal::Float(*f)));
            }
            Agent::PrimVal(PrimVal::Char(c)) => {
                return Ok(Term::NativeLiteral(Literal::Char(*c)));
            }
            Agent::PrimVal(PrimVal::Bool(b)) => {
                // Bool primitives return Bool constructors (True/False)
                return Ok(Term::Constructor(
                    if *b {
                        "True".to_string()
                    } else {
                        "False".to_string()
                    },
                    vec![],
                    None,
                ));
            }
            Agent::PrimVal(PrimVal::Unit) => {
                return Ok(Term::NativeLiteral(Literal::Unit));
            }
            Agent::Constructor(name) => {
                if name == "()" || name == "Unit" {
                    return Ok(Term::NativeLiteral(Literal::Unit));
                }
                if let Ok(n) = name.parse::<i64>() {
                    return Ok(Term::NativeLiteral(Literal::Int(n)));
                }
            }
            Agent::Lambda => {
                // Lambda should have been reduced, but try to extract from body
                let node_id = NodeId(node_id);
                if let Some((body_node, _body_port)) = net.get_connected_port(node_id, PortIndex(2))
                {
                    // Try to recursively extract from the body
                    // Create a temporary subnet focused on the body
                    if let Some(body_node_ref) = net.get_node(body_node) {
                        match &body_node_ref.agent {
                            Agent::PrimVal(val) => {
                                if let Some(term) = prim_val_to_term(val) {
                                    return Ok(term);
                                }
                            }
                            Agent::Constructor(name) => {
                                return Ok(Term::Constructor(name.clone(), vec![], None));
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Err(ExtractionError::NoResult)
}

fn prim_val_to_term(val: &PrimVal) -> Option<Term> {
    match val {
        PrimVal::Int(n) => Some(Term::NativeLiteral(Literal::Int(*n))),
        PrimVal::Float(f) => Some(Term::NativeLiteral(Literal::Float(*f))),
        PrimVal::Char(c) => Some(Term::NativeLiteral(Literal::Char(*c))),
        PrimVal::Unit => Some(Term::NativeLiteral(Literal::Unit)),
        PrimVal::String(s) => Some(Term::NativeLiteral(Literal::Str(s.clone()))),
        PrimVal::Constructor(name, args) => {
            let term_args: Vec<Term> = args.iter().filter_map(prim_val_to_term).collect();
            Some(Term::Constructor(name.clone(), term_args, None))
        }
        _ => None,
    }
}
