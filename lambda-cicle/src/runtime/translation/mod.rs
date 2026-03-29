use crate::core::ast::types::Multiplicity;
use crate::core::ast::{Arm, Literal, Pattern, Term};
use crate::runtime::net::{Agent, Net, Node, NodeId, Port, PortIndex, Wire};
use crate::runtime::primitives::PrimOp;

pub struct NetBuilder {
    net: Net,
}

impl NetBuilder {
    pub fn new() -> NetBuilder {
        NetBuilder { net: Net::new() }
    }

    pub fn build(mut self, term: &Term) -> Net {
        self.translate_term(term);
        self.net
    }

    fn translate_term(&mut self, term: &Term) -> NodeId {
        match term {
            Term::Var(name) => self.translate_var(name),
            Term::Abs {
                var: _,
                multiplicity,
                annot: _,
                body,
            } => self.translate_abs(multiplicity, body),
            Term::App { fun, arg } => self.translate_app(fun, arg),
            Term::Let {
                var: _,
                multiplicity: _,
                annot: _,
                value,
                body,
            } => {
                let value_id = self.translate_term(value);
                let body_id = self.translate_term(body);
                self.connect_ports(value_id, 1, body_id, 0);
                body_id
            }
            Term::Match { scrutinee, arms } => self.translate_match(scrutinee, arms),
            Term::View { scrutinee, arms } => self.translate_view(scrutinee, arms),
            Term::Constructor(name, args) => self.translate_constructor(name, args),
            Term::NativeLiteral(lit) => self.translate_literal(lit),
            Term::BinaryOp { op, left, right } => self.translate_binop(op, left, right),
            Term::UnaryOp { op, arg } => self.translate_unaryop(op, arg),
            Term::TraitMethod {
                trait_name: _,
                method: _,
                arg,
            } => self.translate_term(arg),
        }
    }

    fn translate_var(&mut self, name: &str) -> NodeId {
        let node = Node::new(Agent::Constructor(name.to_string()), 0);
        self.net.add_node(node)
    }

    fn translate_abs(&mut self, mult: &Multiplicity, body: &Term) -> NodeId {
        let lambda = Node::lambda();
        let lambda_id = self.net.add_node(lambda);

        let body_id = self.translate_term(body);

        self.connect_ports(lambda_id, 2, body_id, 0);

        match mult {
            Multiplicity::One => lambda_id,
            Multiplicity::Omega => {
                let delta = Node::delta();
                let delta_id = self.net.add_node(delta);
                self.connect_ports(lambda_id, 1, delta_id, 1);
                delta_id
            }
            Multiplicity::Zero => {
                let epsilon = Node::epsilon();
                let epsilon_id = self.net.add_node(epsilon);
                self.connect_ports(lambda_id, 1, epsilon_id, 0);
                self.net.add_free_port(lambda_id, PortIndex(1));
                lambda_id
            }
            Multiplicity::Borrow => lambda_id,
        }
    }

    fn translate_app(&mut self, fun: &Term, arg: &Term) -> NodeId {
        let fun_id = self.translate_term(fun);
        let arg_id = self.translate_term(arg);

        let app = Node::app();
        let app_id = self.net.add_node(app);

        self.connect_ports(fun_id, 1, app_id, 0);
        self.connect_ports(arg_id, 0, app_id, 1);

        app_id
    }

    fn translate_match(&mut self, scrutinee: &Term, arms: &[Arm]) -> NodeId {
        let scrutinee_id = self.translate_term(scrutinee);

        let app = Node::app();
        let app_id = self.net.add_node(app);

        self.connect_ports(scrutinee_id, 0, app_id, 0);

        for arm in arms {
            let pattern_id = self.translate_pattern(&arm.pattern, Multiplicity::One);
            let body_id = self.translate_term(&arm.body);
            self.connect_ports(pattern_id, 0, app_id, 1);
            self.connect_ports(body_id, 0, app_id, 2);
        }

        app_id
    }

    fn translate_view(&mut self, scrutinee: &Term, arms: &[Arm]) -> NodeId {
        let scrutinee_id = self.translate_term(scrutinee);

        let app = Node::app();
        let app_id = self.net.add_node(app);

        self.connect_ports(scrutinee_id, 0, app_id, 0);

        for arm in arms {
            let pattern_id = self.translate_pattern(&arm.pattern, Multiplicity::Borrow);
            let body_id = self.translate_term(&arm.body);
            self.connect_ports(pattern_id, 0, app_id, 1);
            self.connect_ports(body_id, 0, app_id, 2);
        }

        app_id
    }

    fn translate_pattern(&mut self, pattern: &Pattern, mult: Multiplicity) -> NodeId {
        match pattern {
            Pattern::Wildcard => {
                let epsilon = Node::epsilon();
                self.net.add_node(epsilon)
            }
            Pattern::Var(name) => self.translate_var(name),
            Pattern::Constructor(name, args) => {
                let node = Node::constructor(name.clone(), args.len());
                let node_id = self.net.add_node(node);

                for (i, arg) in args.iter().enumerate() {
                    let arg_id = self.translate_pattern(arg, mult.clone());
                    self.connect_ports(arg_id, 0, node_id, i);
                }

                node_id
            }
        }
    }

    fn translate_constructor(&mut self, name: &str, args: &[Term]) -> NodeId {
        let node = Node::constructor(name.to_string(), args.len());
        let node_id = self.net.add_node(node);

        for (i, arg) in args.iter().enumerate() {
            let arg_id = self.translate_term(arg);
            self.connect_ports(arg_id, 0, node_id, i);
        }

        node_id
    }

    fn translate_literal(&mut self, lit: &Literal) -> NodeId {
        let node = Node::constructor(format!("{:?}", lit), 0);
        self.net.add_node(node)
    }

    fn translate_binop(
        &mut self,
        op: &crate::core::ast::BinOp,
        left: &Term,
        right: &Term,
    ) -> NodeId {
        let prim_op = PrimOp::from_ast_op(op);
        let node = Node::prim(prim_op);
        let node_id = self.net.add_node(node);

        let left_id = self.translate_term(left);
        let right_id = self.translate_term(right);

        self.connect_ports(left_id, 0, node_id, 0);
        self.connect_ports(right_id, 0, node_id, 1);

        node_id
    }

    fn translate_unaryop(&mut self, op: &crate::core::ast::UnOp, arg: &Term) -> NodeId {
        let prim_op = PrimOp::from_unary_op(op);
        let node = Node::prim(prim_op);
        let node_id = self.net.add_node(node);

        let arg_id = self.translate_term(arg);
        self.connect_ports(arg_id, 0, node_id, 0);

        node_id
    }

    fn connect_ports(
        &mut self,
        source_node: NodeId,
        source_port: usize,
        target_node: NodeId,
        target_port: usize,
    ) {
        let source = Port::new(source_node, PortIndex(source_port));
        let target = Port::new(target_node, PortIndex(target_port));
        let wire = Wire::new(source, target);
        self.net.add_wire(wire);
    }
}

impl Default for NetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn translate(term: &Term) -> Net {
    let builder = NetBuilder::new();
    builder.build(term)
}
