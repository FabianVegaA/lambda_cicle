use crate::runtime::net::{Agent, Net, NodeId, PortIndex};

pub fn net_to_dot(net: &Net) -> String {
    let mut output = String::new();

    output.push_str("digraph lambda_cicle {\n");
    output.push_str("    rankdir=LR;\n");
    output.push_str("    node [shape=box];\n\n");

    for (id, node) in net.nodes().iter().enumerate() {
        let label = format_node_label(&node.agent);
        let color = format_node_color(&node.agent);
        output.push_str(&format!(
            "    {} [label=\"{}\" style=filled fillcolor={}];\n",
            id, label, color
        ));
    }

    output.push_str("\n");

    for (id, node) in net.nodes().iter().enumerate() {
        for port_idx in 0..node.num_ports() {
            if let Some((target_node, _target_port)) =
                net.get_connected_port(NodeId(id), PortIndex(port_idx))
            {
                output.push_str(&format!(
                    "    {} -> {} [label=\"{}\"];\n",
                    id, target_node.0, port_idx
                ));
            }
        }
    }

    output.push_str("}\n");

    output
}

fn format_node_label(agent: &Agent) -> String {
    match agent {
        Agent::Lambda => "λ".to_string(),
        Agent::App => "@".to_string(),
        Agent::Delta => "δ".to_string(),
        Agent::Epsilon => "ε".to_string(),
        Agent::Constructor(name) => name.clone(),
        Agent::Prim(op) => format!("{:?}", op),
    }
}

fn format_node_color(agent: &Agent) -> String {
    match agent {
        Agent::Lambda => "lightblue".to_string(),
        Agent::App => "lightcoral".to_string(),
        Agent::Delta => "lightgreen".to_string(),
        Agent::Epsilon => "gold".to_string(),
        Agent::Constructor(_) => "lightgray".to_string(),
        Agent::Prim(_) => "lightyellow".to_string(),
    }
}

pub fn write_dot_file(net: &Net, path: &std::path::Path) -> std::io::Result<()> {
    let dot = net_to_dot(net);
    std::fs::write(path, dot)
}
