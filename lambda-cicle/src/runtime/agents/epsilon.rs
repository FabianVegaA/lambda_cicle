use super::{Agent, InteractionResult};
use crate::runtime::net::{Net, NodeId, PortIndex};

pub struct EpsilonAgent;

impl Agent for EpsilonAgent {
    fn interact(
        &self,
        _net: &mut Net,
        _node: NodeId,
        _other_port: PortIndex,
    ) -> Option<InteractionResult> {
        None
    }
}
