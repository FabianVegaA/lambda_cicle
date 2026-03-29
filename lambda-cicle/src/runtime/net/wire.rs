use super::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireId(pub usize);

impl WireId {
    pub fn new(id: usize) -> WireId {
        WireId(id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Wire {
    pub source: Port,
    pub target: Port,
}

impl Wire {
    pub fn new(source: Port, target: Port) -> Wire {
        Wire { source, target }
    }
}
