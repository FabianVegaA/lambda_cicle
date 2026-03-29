use super::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortIndex(pub usize);

impl PortIndex {
    pub fn new(index: usize) -> PortIndex {
        PortIndex(index)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Port {
    pub node: NodeId,
    pub index: PortIndex,
}

impl Port {
    pub fn new(node: NodeId, index: PortIndex) -> Port {
        Port { node, index }
    }
}
