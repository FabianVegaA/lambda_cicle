use super::{Agent, InteractionResult};
use crate::runtime::net::{Net, NodeId, PortIndex};

pub struct AppAgent;

impl Agent for AppAgent {
    fn interact(
        &self,
        _net: &mut Net,
        _node: NodeId,
        _other_port: PortIndex,
    ) -> Option<InteractionResult> {
        None
    }
}
