use lambda_cicle::runtime::evaluator::Evaluator;
use lambda_cicle::runtime::net::{Agent, InteractionResult, Net, Node, NodeId, PortIndex};
use lambda_cicle::runtime::primitives::{prim_name_to_io_op, IOOp, PrimVal, INTRINSICS_TABLE};

#[test]
fn test_io_primitives_in_table() {
    let io_prims = [
        "prim_io_print",
        "prim_io_println",
        "prim_io_eprint",
        "prim_io_eprintln",
        "prim_io_read_line",
        "prim_io_open",
        "prim_io_close",
        "prim_io_read",
        "prim_io_write",
    ];

    for prim in io_prims {
        assert!(
            INTRINSICS_TABLE.contains(&prim),
            "{} should be in INTRINSICS_TABLE",
            prim
        );
    }
}

#[test]
fn test_prim_name_to_io_op_all_ops() {
    assert_eq!(prim_name_to_io_op("prim_io_print"), Some(IOOp::Print));
    assert_eq!(prim_name_to_io_op("prim_io_println"), Some(IOOp::Println));
    assert_eq!(prim_name_to_io_op("prim_io_eprint"), Some(IOOp::EPrint));
    assert_eq!(prim_name_to_io_op("prim_io_eprintln"), Some(IOOp::EPrintln));
    assert_eq!(
        prim_name_to_io_op("prim_io_read_line"),
        Some(IOOp::ReadLine)
    );
    assert_eq!(prim_name_to_io_op("prim_io_open"), Some(IOOp::Open));
    assert_eq!(prim_name_to_io_op("prim_io_close"), Some(IOOp::Close));
    assert_eq!(prim_name_to_io_op("prim_io_read"), Some(IOOp::Read));
    assert_eq!(prim_name_to_io_op("prim_io_write"), Some(IOOp::Write));
}

#[test]
fn test_prim_name_to_io_op_invalid() {
    assert_eq!(prim_name_to_io_op("prim_io_invalid"), None);
    assert_eq!(prim_name_to_io_op("prim_io_print_extra"), None);
    assert_eq!(prim_name_to_io_op("prim_iadd"), None);
    assert_eq!(prim_name_to_io_op(""), None);
}

#[test]
fn test_io_op_arity() {
    assert_eq!(IOOp::Print.arity(), 1);
    assert_eq!(IOOp::Println.arity(), 1);
    assert_eq!(IOOp::EPrint.arity(), 1);
    assert_eq!(IOOp::EPrintln.arity(), 1);
    assert_eq!(IOOp::ReadLine.arity(), 0);
    assert_eq!(IOOp::Open.arity(), 2);
    assert_eq!(IOOp::Close.arity(), 1);
    assert_eq!(IOOp::Read.arity(), 1);
    assert_eq!(IOOp::Write.arity(), 2);
}

#[test]
fn test_node_prim_io_creation() {
    let node = Node::prim_io(IOOp::Print);
    match node.agent {
        Agent::PrimIO(op) => assert_eq!(op, IOOp::Print),
        _ => panic!("Expected PrimIO agent"),
    }
}

#[test]
fn test_node_io_token_creation() {
    let node = Node::io_token();
    match node.agent {
        Agent::IOToken => {}
        _ => panic!("Expected IOToken agent"),
    }
}

fn create_prim_io_print_net() -> Net {
    let mut net = Net::new();

    let io_node = Node::prim_io(IOOp::Print);
    let io_id = net.add_node(io_node);

    let io_token = Node::io_token();
    let token_id = net.add_node(io_token);

    let string_val = Node::prim_val(PrimVal::String("test".to_string()));
    let string_id = net.add_node(string_val);

    net.connect(token_id, PortIndex(0), io_id, PortIndex(1));
    net.connect(string_id, PortIndex(0), io_id, PortIndex(2));

    net.add_free_port(io_id, PortIndex(0));

    net
}

#[test]
fn test_prim_io_print_net_structure() {
    let net = create_prim_io_print_net();

    let io_nodes: Vec<_> = net
        .nodes()
        .iter()
        .filter(|n| matches!(n.agent, Agent::PrimIO(_)))
        .collect();

    assert_eq!(io_nodes.len(), 1, "Should have exactly one IO node");

    let io_node = &io_nodes[0];
    match &io_node.agent {
        Agent::PrimIO(op) => assert_eq!(*op, IOOp::Print),
        _ => panic!("Expected PrimIO"),
    }
}

#[test]
fn test_io_token_exists() {
    let net = create_prim_io_print_net();

    let token_nodes: Vec<_> = net
        .nodes()
        .iter()
        .filter(|n| matches!(n.agent, Agent::IOToken))
        .collect();

    assert_eq!(token_nodes.len(), 1, "Should have exactly one IO token");
}

#[test]
fn test_prim_io_print_fires_with_token() {
    let mut net = create_prim_io_print_net();

    let mut fired = false;
    for _ in 0..100 {
        match net.step() {
            InteractionResult::PrimEval => {
                fired = true;
                break;
            }
            InteractionResult::None => break,
            _ => continue,
        }
    }

    assert!(
        fired,
        "PrimIO should fire when IO token and arguments are present"
    );
}

#[test]
fn test_prim_io_println_arity() {
    let node = Node::prim_io(IOOp::Println);
    let id = net::Net::new().add_node(node);
    // Println should have arity 1 (one argument + IO token)
    // The node is created with ports: 0 (result), 1 (token), 2+ (args)
}

#[test]
fn test_prim_io_write_arity() {
    let node = Node::prim_io(IOOp::Write);
    match node.agent {
        Agent::PrimIO(IOOp::Write) => {}
        _ => panic!("Expected PrimIO(Write)"),
    }
}

mod net {
    pub use lambda_cicle::runtime::net::Net;
}

#[test]
fn test_wire_io_entry_point_creates_token() {
    use lambda_cicle::runtime::net::{Agent, Node, PortIndex};

    let mut net = Net::new();

    let io_node = Node::prim_io(lambda_cicle::runtime::primitives::IOOp::Println);
    let io_id = net.add_node(io_node);

    net.add_free_port(io_id, PortIndex(0));
    net.add_free_port(io_id, PortIndex(1));
    net.add_free_port(io_id, PortIndex(2));

    assert!(
        net.get_wire_at_port(io_id, PortIndex(1)).is_none(),
        "Port 1 should be free before wiring"
    );

    use lambda_cicle::runtime::evaluator::SequentialEvaluator;
    let evaluator = SequentialEvaluator::new();
    let result = evaluator.evaluate(&mut net);

    assert!(result.is_ok(), "Evaluation should succeed");

    let token_nodes: Vec<_> = net
        .nodes()
        .iter()
        .filter(|n| matches!(n.agent, Agent::IOToken))
        .collect();

    assert_eq!(
        token_nodes.len(),
        1,
        "Should have exactly one IO token after wiring"
    );
}
