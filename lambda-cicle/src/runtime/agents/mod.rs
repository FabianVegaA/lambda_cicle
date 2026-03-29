mod lambda;
mod app;
mod delta;
mod epsilon;

pub use lambda::LambdaAgent;
pub use app::AppAgent;
pub use delta::DeltaAgent;
pub use epsilon::EpsilonAgent;

use crate::runtime::net::{Net, NodeId, PortIndex};

pub trait Agent {
    fn interact(&self, net: &mut Net, node: NodeId, other_port: PortIndex) -> Option<InteractionResult>;
}

pub struct InteractionResult {
    pub new_nodes: Vec<NodeId>,
    pub removed_nodes: Vec<NodeId>,
}
