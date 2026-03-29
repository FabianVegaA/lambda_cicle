use super::{Agent, InteractionResult};
use crate::runtime::net::{Net, NodeId, PortIndex};

pub struct DeltaAgent;

impl Agent for DeltaAgent {
    fn interact(
        &self,
        _net: &mut Net,
        _node: NodeId,
        _other_port: PortIndex,
    ) -> Option<InteractionResult> {
        None
    }
}
