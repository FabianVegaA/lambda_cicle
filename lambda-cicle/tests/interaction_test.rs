use lambda_cicle::runtime::net::{Agent, InteractionResult, Net, Node, PortIndex};
use lambda_cicle::runtime::primitives::{PrimOp, PrimVal};

fn create_prim_iadd_net() -> Net {
    let mut net = Net::new();

    let prim_node = Node::prim(PrimOp::IAdd);
    let prim_id = net.add_node(prim_node);

    let val1 = Node::prim_val(PrimVal::Int(2));
    let val1_id = net.add_node(val1);

    let val2 = Node::prim_val(PrimVal::Int(3));
    let val2_id = net.add_node(val2);

    net.connect(prim_id, PortIndex(1), val1_id, PortIndex(0));
    net.connect(prim_id, PortIndex(2), val2_id, PortIndex(0));

    net.add_free_port(prim_id, PortIndex(0));

    net
}

fn create_prim_ineg_net() -> Net {
    let mut net = Net::new();

    let prim_node = Node::prim(PrimOp::INeg);
    let prim_id = net.add_node(prim_node);

    let val = Node::prim_val(PrimVal::Int(5));
    let val_id = net.add_node(val);

    net.connect(prim_id, PortIndex(1), val_id, PortIndex(0));

    net.add_free_port(prim_id, PortIndex(0));

    net
}

fn create_primval_delta_net() -> Net {
    let mut net = Net::new();

    let val = Node::prim_val(PrimVal::Int(42));
    let val_id = net.add_node(val);

    let delta = Node::delta();
    let delta_id = net.add_node(delta);

    net.connect(val_id, PortIndex(1), delta_id, PortIndex(1));

    net.add_free_port(delta_id, PortIndex(0));
    net.add_free_port(delta_id, PortIndex(2));

    net
}

#[test]
fn test_prim_iadd_fires() {
    let mut net = create_prim_iadd_net();

    let mut found_result = false;
    for _ in 0..100 {
        let result = net.step();
        if matches!(result, InteractionResult::PrimEval) {
            found_result = true;
            break;
        }
        if matches!(result, InteractionResult::None) {
            break;
        }
    }

    assert!(found_result, "PrimEval should have fired");

    let has_primval = net
        .nodes()
        .iter()
        .any(|n| matches!(n.agent, Agent::PrimVal(PrimVal::Int(5))));
    assert!(has_primval, "Result should be PrimVal::Int(5)");
}

#[test]
fn test_prim_ineg_fires() {
    let mut net = create_prim_ineg_net();

    let mut found_result = false;
    for _ in 0..100 {
        let result = net.step();
        if matches!(result, InteractionResult::PrimEval) {
            found_result = true;
            break;
        }
        if matches!(result, InteractionResult::None) {
            break;
        }
    }

    assert!(found_result, "PrimEval should have fired");

    let has_primval = net
        .nodes()
        .iter()
        .any(|n| matches!(n.agent, Agent::PrimVal(PrimVal::Int(-5))));
    assert!(has_primval, "Result should be PrimVal::Int(-5)");
}

#[test]
fn test_primval_delta_duplicates() {
    let mut net = create_primval_delta_net();

    let initial_count = net.nodes().len();

    let mut found_result = false;
    for _ in 0..100 {
        let result = net.step();
        if matches!(result, InteractionResult::PrimValDup) {
            found_result = true;
            break;
        }
        if matches!(result, InteractionResult::None) {
            break;
        }
    }

    assert!(found_result, "PrimValDup should have fired");

    let new_count = net.nodes().len();
    assert_eq!(
        new_count,
        initial_count + 1,
        "Should have one more node after duplication"
    );

    let primval_count = net
        .nodes()
        .iter()
        .filter(|n| matches!(n.agent, Agent::PrimVal(PrimVal::Int(42))))
        .count();
    assert_eq!(primval_count, 2, "Should have two PrimVal::Int(42) nodes");
}

#[test]
fn test_unary_prim_ineg_evaluates() {
    let mut net = Net::new();
    let prim_node = Node::prim(PrimOp::INeg);
    let prim_id = net.add_node(prim_node);

    let val_node = Node::prim_val(PrimVal::Int(10));
    let val_id = net.add_node(val_node);

    net.connect(prim_id, PortIndex(1), val_id, PortIndex(0));
    net.add_free_port(prim_id, PortIndex(0));

    let mut fired = false;
    for _ in 0..10 {
        if matches!(net.step(), InteractionResult::PrimEval) {
            fired = true;
            break;
        }
    }

    assert!(fired, "Unary op INeg should fire");
}

#[test]
fn test_binary_prim_iadd_evaluates() {
    let mut net = Net::new();
    let prim_node = Node::prim(PrimOp::IAdd);
    let prim_id = net.add_node(prim_node);

    let val1 = Node::prim_val(PrimVal::Int(10));
    let val1_id = net.add_node(val1);

    let val2 = Node::prim_val(PrimVal::Int(3));
    let val2_id = net.add_node(val2);

    net.connect(prim_id, PortIndex(1), val1_id, PortIndex(0));
    net.connect(prim_id, PortIndex(2), val2_id, PortIndex(0));

    net.add_free_port(prim_id, PortIndex(0));

    let mut fired = false;
    for _ in 0..10 {
        if matches!(net.step(), InteractionResult::PrimEval) {
            fired = true;
            break;
        }
    }

    assert!(fired, "Binary op IAdd should fire");
}
