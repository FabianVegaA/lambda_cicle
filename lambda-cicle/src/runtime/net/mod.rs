mod node;
mod port;
mod wire;

pub use node::Node;
pub use node::Agent;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    pub fn new(id: usize) -> NodeId {
        NodeId(id)
    }
}

impl Net {
    pub fn new() -> Net {
        Net {
            nodes: Vec::new(),
            wires: Vec::new(),
            free_ports: Vec::new(),
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
        self.wires.push(wire);
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
}

impl Default for Net {
    fn default() -> Self {
        Self::new()
    }
}
