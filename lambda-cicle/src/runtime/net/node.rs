use super::PortIndex;
use crate::runtime::primitives::PrimOp;

#[derive(Debug, Clone)]
pub struct Node {
    pub agent: Agent,
    pub ports: Vec<PortIndex>,
}

impl Node {
    pub fn new(agent: Agent, num_ports: usize) -> Node {
        Node {
            agent,
            ports: vec![PortIndex(0); num_ports],
        }
    }

    pub fn lambda() -> Node {
        Node::new(Agent::Lambda, 3)
    }

    pub fn app() -> Node {
        Node::new(Agent::App, 3)
    }

    pub fn delta() -> Node {
        Node::new(Agent::Delta, 3)
    }

    pub fn epsilon() -> Node {
        Node::new(Agent::Epsilon, 1)
    }

    pub fn constructor(name: String, arity: usize) -> Node {
        Node::new(Agent::Constructor(name), arity)
    }

    pub fn prim(op: PrimOp) -> Node {
        Node::new(Agent::Prim(op), 3)
    }

    pub fn num_ports(&self) -> usize {
        self.ports.len()
    }

    pub fn set_port(&mut self, index: usize, port: PortIndex) {
        if index < self.ports.len() {
            self.ports[index] = port;
        }
    }

    pub fn get_port(&self, index: usize) -> Option<PortIndex> {
        self.ports.get(index).copied()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Agent {
    Lambda,
    App,
    Delta,
    Epsilon,
    Constructor(String),
    Prim(PrimOp),
}

impl Agent {
    pub fn arity(&self) -> usize {
        match self {
            Agent::Lambda => 3,
            Agent::App => 3,
            Agent::Delta => 3,
            Agent::Epsilon => 1,
            Agent::Constructor(_) => 0,
            Agent::Prim(_) => 3,
        }
    }

    pub fn is_constructor(&self) -> bool {
        matches!(self, Agent::Constructor(_))
    }
}
