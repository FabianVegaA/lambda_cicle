mod parallel;
mod sequential;

pub use parallel::ParallelEvaluator;
pub use sequential::SequentialEvaluator;

use crate::core::ast::Term;
use crate::runtime::net::{Agent, Net, NodeId, PortIndex};

pub trait Evaluator {
    fn evaluate(&self, net: &mut Net) -> Result<Term, EvalError>;
}

#[derive(Debug)]
pub enum EvalError {
    NonTerminating,
    Stuck(String),
    TypeError(String),
    S5PrimeViolation(String),
    EvaluationError(String),
}

pub fn verify_s5_prime(net: &Net) -> Result<(), EvalError> {
    let mut stack: Vec<NodeId> = Vec::new();

    for (delta_id, node) in net.nodes().iter().enumerate() {
        if !matches!(node.agent, Agent::Delta) {
            continue;
        }

        let delta_id = NodeId::new(delta_id);

        let output1 = net.get_connected_port(delta_id, PortIndex(1));
        let output2 = net.get_connected_port(delta_id, PortIndex(2));

        let Some((target1, _)) = output1 else {
            continue;
        };
        let Some((target2, _)) = output2 else {
            continue;
        };

        let subgraph1 = reachable_nodes(net, target1, &mut stack);
        let subgraph2 = reachable_nodes(net, target2, &mut stack);

        let roots1 = find_root_deltas(net, &subgraph1);
        let roots2 = find_root_deltas(net, &subgraph2);

        for r1 in &roots1 {
            if roots2.contains(r1) {
                return Err(EvalError::S5PrimeViolation(format!(
                    "δ at {:?} shares root with both branches",
                    delta_id
                )));
            }
        }
    }

    Ok(())
}

fn reachable_nodes(
    net: &Net,
    start: NodeId,
    stack: &mut Vec<NodeId>,
) -> std::collections::HashSet<NodeId> {
    let mut result = std::collections::HashSet::new();
    stack.push(start);

    while let Some(node_id) = stack.pop() {
        if result.contains(&node_id) {
            continue;
        }
        result.insert(node_id);

        let Some(node) = net.get_node(node_id) else {
            continue;
        };

        for port_idx in 0..node.num_ports() {
            if let Some((connected_node, _)) = net.get_connected_port(node_id, PortIndex(port_idx))
            {
                if !result.contains(&connected_node) {
                    stack.push(connected_node);
                }
            }
        }
    }

    result
}

fn find_root_deltas(
    net: &Net,
    subgraph: &std::collections::HashSet<NodeId>,
) -> std::collections::HashSet<NodeId> {
    let mut roots = std::collections::HashSet::new();

    for &node_id in subgraph {
        let Some(node) = net.get_node(node_id) else {
            continue;
        };

        if !matches!(node.agent, Agent::Delta) {
            continue;
        }

        let input = net.get_connected_port(node_id, PortIndex(0));
        let has_input_from_subgraph = input.map(|(n, _)| subgraph.contains(&n)).unwrap_or(false);

        if !has_input_from_subgraph {
            roots.insert(node_id);
        }
    }

    roots
}
