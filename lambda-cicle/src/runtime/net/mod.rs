mod node;
mod port;
mod wire;

pub use node::Agent;
pub use node::Node;
pub use port::Port;
pub use port::PortIndex;
pub use wire::Wire;
pub use wire::WireId;

use hashbrown::HashMap;

#[derive(Debug, Clone)]
pub struct Net {
    nodes: Vec<Node>,
    wires: Vec<Wire>,
    free_ports: Vec<(NodeId, PortIndex)>,
    port_to_wire: HashMap<(NodeId, PortIndex), WireId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    pub fn new(id: usize) -> NodeId {
        NodeId(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionResult {
    BetaReduction,
    Duplication,
    Erasure,
    Commute,
    EraseBranch,
    None,
}

impl Net {
    pub fn new() -> Net {
        Net {
            nodes: Vec::new(),
            wires: Vec::new(),
            free_ports: Vec::new(),
            port_to_wire: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.0)
    }

    pub fn add_wire(&mut self, wire: Wire) -> WireId {
        let id = WireId(self.wires.len());
        self.wires.push(wire.clone());

        let source = wire.source;
        let target = wire.target;
        self.port_to_wire.insert((source.node, source.index), id);
        self.port_to_wire.insert((target.node, target.index), id);

        id
    }

    pub fn get_wire(&self, id: WireId) -> Option<&Wire> {
        self.wires.get(id.0)
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn wires(&self) -> &[Wire] {
        &self.wires
    }

    pub fn add_free_port(&mut self, node_id: NodeId, port: PortIndex) {
        self.free_ports.push((node_id, port));
    }

    pub fn take_free_port(&mut self) -> Option<(NodeId, PortIndex)> {
        self.free_ports.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.wires.is_empty()
    }

    pub fn get_wire_at_port(&self, node_id: NodeId, port: PortIndex) -> Option<WireId> {
        self.port_to_wire.get(&(node_id, port)).copied()
    }

    pub fn get_connected_port(
        &self,
        node_id: NodeId,
        port: PortIndex,
    ) -> Option<(NodeId, PortIndex)> {
        let wire_id = self.get_wire_at_port(node_id, port)?;
        let wire = self.get_wire(wire_id)?;

        let source = &wire.source;
        let target = &wire.target;

        if source.node == node_id && source.index == port {
            Some((target.node, target.index))
        } else if target.node == node_id && target.index == port {
            Some((source.node, source.index))
        } else {
            None
        }
    }

    pub fn disconnect_port(&mut self, node_id: NodeId, port: PortIndex) -> Option<WireId> {
        let wire_id = self.port_to_wire.remove(&(node_id, port))?;

        let wire = &self.wires[wire_id.0];
        let source = &wire.source;
        let target = &wire.target;

        if source.node == node_id && source.index == port {
            self.port_to_wire.remove(&(target.node, target.index));
        } else {
            self.port_to_wire.remove(&(source.node, source.index));
        }

        Some(wire_id)
    }

    pub fn connect(
        &mut self,
        source_node: NodeId,
        source_port: PortIndex,
        target_node: NodeId,
        target_port: PortIndex,
    ) {
        let wire = Wire::new(
            Port::new(source_node, source_port),
            Port::new(target_node, target_port),
        );
        self.add_wire(wire);
    }

    pub fn step(&mut self) -> InteractionResult {
        if let Some(result) = self.try_beta_reduction() {
            return result;
        }
        if let Some(result) = self.try_duplication() {
            return result;
        }
        if let Some(result) = self.try_erasure() {
            return result;
        }
        if let Some(result) = self.try_commute() {
            return result;
        }
        if let Some(result) = self.try_erase_branch() {
            return result;
        }
        InteractionResult::None
    }

    fn try_beta_reduction(&mut self) -> Option<InteractionResult> {
        for (lambda_id, node) in self.nodes.iter().enumerate() {
            if !matches!(node.agent, Agent::Lambda) {
                continue;
            }

            let lambda_id = NodeId(lambda_id);

            let lambda_port_1 = self.get_connected_port(lambda_id, PortIndex(1))?;
            let (app_id, app_port_0_idx) = lambda_port_1;

            if app_port_0_idx != PortIndex(0) {
                continue;
            }

            let app_node = self.get_node(app_id)?;
            if !matches!(app_node.agent, Agent::App) {
                continue;
            }

            let (arg_node, arg_port) = self.get_connected_port(app_id, PortIndex(1))?;

            self.disconnect_port(lambda_id, PortIndex(1))?;
            self.disconnect_port(app_id, PortIndex(1))?;

            self.connect(arg_node, arg_port, lambda_id, PortIndex(2));
            self.connect(lambda_id, PortIndex(0), app_id, PortIndex(2));

            return Some(InteractionResult::BetaReduction);
        }
        None
    }

    fn try_duplication(&mut self) -> Option<InteractionResult> {
        for (delta_id, node) in self.nodes.iter().enumerate() {
            if !matches!(node.agent, Agent::Delta) {
                continue;
            }

            let delta_id = NodeId(delta_id);

            let delta_port_1 = self.get_connected_port(delta_id, PortIndex(1))?;
            let (lambda_id, lambda_port_idx) = delta_port_1;

            if lambda_port_idx != PortIndex(1) {
                continue;
            }

            let lambda_node = self.get_node(lambda_id)?;
            if !matches!(lambda_node.agent, Agent::Lambda) {
                continue;
            }

            let lambda_body = self.get_connected_port(lambda_id, PortIndex(2))?;

            let new_lambda = Node::lambda();
            let new_lambda_id = self.add_node(new_lambda);

            self.connect(new_lambda_id, PortIndex(2), lambda_body.0, lambda_body.1);
            self.connect(delta_id, PortIndex(2), new_lambda_id, PortIndex(1));

            return Some(InteractionResult::Duplication);
        }
        None
    }

    fn try_erasure(&mut self) -> Option<InteractionResult> {
        for (epsilon_id, node) in self.nodes.iter().enumerate() {
            if !matches!(node.agent, Agent::Epsilon) {
                continue;
            }

            let epsilon_id = NodeId(epsilon_id);

            let epsilon_port_0 = self.get_connected_port(epsilon_id, PortIndex(0))?;
            let (lambda_id, lambda_port_idx) = epsilon_port_0;

            if lambda_port_idx != PortIndex(1) {
                continue;
            }

            let lambda_node = self.get_node(lambda_id)?;
            if !matches!(lambda_node.agent, Agent::Lambda) {
                continue;
            }

            self.disconnect_port(epsilon_id, PortIndex(0))?;
            self.disconnect_port(lambda_id, PortIndex(1))?;

            if let Some((body_node, body_port)) = self.get_connected_port(lambda_id, PortIndex(2)) {
                self.disconnect_port(lambda_id, PortIndex(2));
                self.add_free_port(body_node, body_port);
            }

            self.add_free_port(lambda_id, PortIndex(1));
            self.add_free_port(lambda_id, PortIndex(2));

            return Some(InteractionResult::Erasure);
        }
        None
    }

    fn try_commute(&mut self) -> Option<InteractionResult> {
        for (delta1_id, node) in self.nodes.iter().enumerate() {
            if !matches!(node.agent, Agent::Delta) {
                continue;
            }

            let delta1_id = NodeId(delta1_id);

            let port_0 = self.get_connected_port(delta1_id, PortIndex(0))?;
            let (delta2_id, delta2_port_idx) = port_0;

            if delta2_port_idx != PortIndex(0) {
                continue;
            }

            let delta2_node = self.get_node(delta2_id)?;
            if !matches!(delta2_node.agent, Agent::Delta) {
                continue;
            }

            let delta1_port_1 = self.get_connected_port(delta1_id, PortIndex(1));
            let delta1_port_2 = self.get_connected_port(delta1_id, PortIndex(2));
            let delta2_port_1 = self.get_connected_port(delta2_id, PortIndex(1));
            let delta2_port_2 = self.get_connected_port(delta2_id, PortIndex(2));

            self.disconnect_port(delta1_id, PortIndex(0))?;
            self.disconnect_port(delta2_id, PortIndex(0))?;

            if let Some((n, p)) = delta2_port_1 {
                self.disconnect_port(delta2_id, PortIndex(1))?;
                self.connect(delta1_id, PortIndex(0), n, p);
            }
            if let Some((n, p)) = delta2_port_2 {
                self.disconnect_port(delta2_id, PortIndex(2))?;
                self.connect(delta1_id, PortIndex(1), n, p);
            }

            if let Some((n, p)) = delta1_port_1 {
                self.disconnect_port(delta1_id, PortIndex(1))?;
                self.connect(delta2_id, PortIndex(0), n, p);
            }
            if let Some((n, p)) = delta1_port_2 {
                self.disconnect_port(delta1_id, PortIndex(2))?;
                self.connect(delta2_id, PortIndex(1), n, p);
            }

            return Some(InteractionResult::Commute);
        }
        None
    }

    fn try_erase_branch(&mut self) -> Option<InteractionResult> {
        for (delta_id, node) in self.nodes.iter().enumerate() {
            if !matches!(node.agent, Agent::Delta) {
                continue;
            }

            let delta_id = NodeId(delta_id);

            for port_idx in [PortIndex(1), PortIndex(2)] {
                let port = self.get_connected_port(delta_id, port_idx)?;
                let (other_id, other_port_idx) = port;

                let other_node = self.get_node(other_id)?;
                if matches!(other_node.agent, Agent::Epsilon) {
                    self.disconnect_port(delta_id, port_idx)?;
                    self.disconnect_port(other_id, other_port_idx)?;
                    self.add_free_port(delta_id, port_idx);
                    return Some(InteractionResult::EraseBranch);
                }
            }
        }
        None
    }

    pub fn is_stuck(&self) -> bool {
        for node in &self.nodes {
            match &node.agent {
                Agent::Lambda | Agent::App | Agent::Delta => {
                    return false;
                }
                _ => {}
            }
        }
        true
    }
}

impl Default for Net {
    fn default() -> Self {
        Self::new()
    }
}
