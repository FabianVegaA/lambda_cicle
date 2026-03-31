use crate::runtime::net::Agent;
use crate::runtime::net::{InteractionResult, Net, NodeId, PortIndex};

pub struct TraceDebugger {
    show_steps: bool,
    highlight_epsilon: bool,
    max_steps: usize,
}

impl TraceDebugger {
    pub fn new() -> Self {
        TraceDebugger {
            show_steps: true,
            highlight_epsilon: true,
            max_steps: 1000,
        }
    }

    pub fn with_max_steps(mut self, max_steps: usize) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub fn trace(&self, net: &mut Net) -> Trace {
        let mut trace = Trace { steps: Vec::new() };
        let mut steps = 0;

        while steps < self.max_steps {
            if net.is_stuck() {
                break;
            }

            let active_nodes = self.get_active_nodes(net);
            let epsilon_events = self.find_epsilon_events(net);

            let result = net.step();

            let trace_step = TraceStep {
                step_number: steps,
                interaction: result.clone(),
                active_nodes,
                epsilon_events,
            };

            trace.steps.push(trace_step);
            steps += 1;

            if matches!(result, InteractionResult::None) {
                break;
            }
        }

        trace
    }

    pub fn print_trace(&self, trace: &Trace) {
        println!("Reduction trace ({} steps):\n", trace.steps.len());

        for step in &trace.steps {
            if self.show_steps {
                print!("Step {:4}: ", step.step_number);
            }

            match &step.interaction {
                InteractionResult::BetaReduction => {
                    println!("β-reduction (λ ⋈ @)");
                }
                InteractionResult::Duplication => {
                    println!("duplication (λ ⋈ δ)");
                }
                InteractionResult::Erasure => {
                    println!("erasure (λ ⋈ ε)");
                }
                InteractionResult::Commute => {
                    println!("commute (δ ⋈ δ)");
                }
                InteractionResult::EraseBranch => {
                    println!("erase branch (δ ⋈ ε)");
                }
                InteractionResult::PrimEval => {
                    println!("prim-eval (Prim ⋈ PrimVal)");
                }
                InteractionResult::PrimValErase => {
                    println!("prim-val-erase (PrimVal ⋈ ε)");
                }
                InteractionResult::PrimValDup => {
                    println!("prim-val-dup (PrimVal ⋈ δ)");
                }
                InteractionResult::None => {
                    println!("stuck");
                }
            }

            if self.highlight_epsilon && !step.epsilon_events.is_empty() {
                println!("  ⚠ ε-firing events: {:?}", step.epsilon_events);
            }
        }
    }

    fn get_active_nodes(&self, net: &Net) -> Vec<NodeId> {
        net.nodes()
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                matches!(
                    node.agent,
                    Agent::Lambda | Agent::App | Agent::Delta | Agent::Prim(_) | Agent::PrimIO(_)
                )
            })
            .map(|(i, _)| NodeId(i))
            .collect()
    }

    fn find_epsilon_events(&self, net: &Net) -> Vec<EpsilonEvent> {
        let mut events = Vec::new();

        for (id, node) in net.nodes().iter().enumerate() {
            if matches!(node.agent, Agent::Epsilon) {
                if let Some((target, port)) = net.get_connected_port(NodeId(id), PortIndex(0)) {
                    events.push(EpsilonEvent {
                        epsilon_node: NodeId(id),
                        target,
                        target_port: port,
                    });
                }
            }
        }

        events
    }
}

impl Default for TraceDebugger {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Trace {
    pub steps: Vec<TraceStep>,
}

pub struct TraceStep {
    pub step_number: usize,
    pub interaction: InteractionResult,
    pub active_nodes: Vec<NodeId>,
    pub epsilon_events: Vec<EpsilonEvent>,
}

#[derive(Debug)]
pub struct EpsilonEvent {
    pub epsilon_node: NodeId,
    pub target: NodeId,
    pub target_port: PortIndex,
}
