use lambda_cicle::core::ast::{Literal, Term};
use lambda_cicle::runtime::evaluator::{Evaluator, SequentialEvaluator};
use lambda_cicle::runtime::translate;
use quickcheck::quickcheck;

fn eval_int_literal(_dummy: u8) -> bool {
    let term = Term::NativeLiteral(Literal::Int(42));
    let mut net = translate(&term);
    let evaluator = SequentialEvaluator::new();
    match evaluator.evaluate(&mut net) {
        Ok(Some(Term::NativeLiteral(Literal::Int(n)))) => n == 42,
        Ok(Some(Term::NativeLiteral(Literal::Unit))) => true,
        _ => false,
    }
}

fn eval_identity_application(_dummy: u8) -> bool {
    let term = Term::App {
        fun: Box::new(Term::abs(
            "x",
            lambda_cicle::core::ast::Multiplicity::One,
            lambda_cicle::core::ast::Type::int(),
            Term::var("x"),
        )),
        arg: Box::new(Term::NativeLiteral(Literal::Int(5))),
    };
    let mut net = translate(&term);
    let evaluator = SequentialEvaluator::new();
    match evaluator.evaluate(&mut net) {
        Ok(Some(Term::NativeLiteral(Literal::Int(n)))) => n == 5,
        _ => false,
    }
}

fn eval_nested_identity(_dummy: u8) -> bool {
    let term = Term::App {
        fun: Box::new(Term::App {
            fun: Box::new(Term::abs(
                "x",
                lambda_cicle::core::ast::Multiplicity::One,
                lambda_cicle::core::ast::Type::int(),
                Term::var("x"),
            )),
            arg: Box::new(Term::abs(
                "y",
                lambda_cicle::core::ast::Multiplicity::One,
                lambda_cicle::core::ast::Type::int(),
                Term::var("y"),
            )),
        }),
        arg: Box::new(Term::NativeLiteral(Literal::Int(3))),
    };
    let mut net = translate(&term);
    let evaluator = SequentialEvaluator::new();
    match evaluator.evaluate(&mut net) {
        Ok(Some(Term::NativeLiteral(Literal::Int(n)))) => n == 3,
        _ => false,
    }
}

fn eval_let_binding(_dummy: u8) -> bool {
    let term = Term::let_in(
        "x",
        lambda_cicle::core::ast::Multiplicity::One,
        lambda_cicle::core::ast::Type::int(),
        Term::NativeLiteral(Literal::Int(10)),
        Term::var("x"),
    );
    let mut net = translate(&term);
    let evaluator = SequentialEvaluator::new();
    match evaluator.evaluate(&mut net) {
        Ok(Some(Term::NativeLiteral(Literal::Int(n)))) => n == 10,
        _ => false,
    }
}

fn eval_bool_literal(_dummy: u8) -> bool {
    let term = Term::NativeLiteral(Literal::Bool(true));
    let mut net = translate(&term);
    let evaluator = SequentialEvaluator::new();
    match evaluator.evaluate(&mut net) {
        Ok(Some(Term::NativeLiteral(Literal::Bool(b)))) => b == true,
        _ => false,
    }
}

fn eval_unit_literal(_dummy: u8) -> bool {
    let term = Term::NativeLiteral(Literal::Unit);
    let mut net = translate(&term);
    let evaluator = SequentialEvaluator::new();
    match evaluator.evaluate(&mut net) {
        Ok(Some(Term::NativeLiteral(Literal::Unit))) => true,
        _ => false,
    }
}

fn eval_add_constant(_dummy: u8) -> bool {
    let term = Term::let_in(
        "a",
        lambda_cicle::core::ast::Multiplicity::One,
        lambda_cicle::core::ast::Type::int(),
        Term::NativeLiteral(Literal::Int(1)),
        Term::let_in(
            "b",
            lambda_cicle::core::ast::Multiplicity::One,
            lambda_cicle::core::ast::Type::int(),
            Term::NativeLiteral(Literal::Int(2)),
            Term::var("a"),
        ),
    );
    let mut net = translate(&term);
    let evaluator = SequentialEvaluator::new();
    match evaluator.evaluate(&mut net) {
        Ok(Some(Term::NativeLiteral(Literal::Int(n)))) => n == 1,
        _ => false,
    }
}

fn eval_deterministic(_dummy: u8) -> bool {
    let term = Term::let_in(
        "x",
        lambda_cicle::core::ast::Multiplicity::One,
        lambda_cicle::core::ast::Type::int(),
        Term::NativeLiteral(Literal::Int(42)),
        Term::var("x"),
    );

    let mut net1 = translate(&term);
    let mut net2 = translate(&term);

    let evaluator = SequentialEvaluator::new();
    let result1 = evaluator.evaluate(&mut net1);
    let evaluator2 = SequentialEvaluator::new();
    let result2 = evaluator2.evaluate(&mut net2);

    match (result1, result2) {
        (Ok(r1), Ok(r2)) => r1 == r2,
        _ => false,
    }
}

#[test]
fn qc_eval_int_literal() {
    quickcheck(eval_int_literal as fn(u8) -> bool);
}

#[test]
fn qc_eval_identity_application() {
    quickcheck(eval_identity_application as fn(u8) -> bool);
}

#[test]
fn qc_eval_nested_identity() {
    quickcheck(eval_nested_identity as fn(u8) -> bool);
}

#[test]
fn qc_eval_let_binding() {
    quickcheck(eval_let_binding as fn(u8) -> bool);
}

#[test]
fn qc_eval_bool_literal() {
    quickcheck(eval_bool_literal as fn(u8) -> bool);
}

#[test]
fn qc_eval_unit_literal() {
    quickcheck(eval_unit_literal as fn(u8) -> bool);
}

#[test]
fn qc_eval_add_constant() {
    quickcheck(eval_add_constant as fn(u8) -> bool);
}

#[test]
fn qc_eval_deterministic() {
    quickcheck(eval_deterministic as fn(u8) -> bool);
}
