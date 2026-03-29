use lambda_cicle::core::ast::{Literal, Term, Type};
use lambda_cicle::core::typecheck::type_check_with_borrow_check;
use quickcheck::{quickcheck, Arbitrary, Gen};

fn type_int_literal(_dummy: u8) -> bool {
    let term = Term::NativeLiteral(Literal::Int(42));
    match type_check_with_borrow_check(&term) {
        Ok(ty) => ty == Type::int(),
        Err(_) => false,
    }
}

fn type_bool_literal(_dummy: u8) -> bool {
    let term = Term::NativeLiteral(Literal::Bool(true));
    match type_check_with_borrow_check(&term) {
        Ok(ty) => ty == Type::bool(),
        Err(_) => false,
    }
}

fn type_unit_literal(_dummy: u8) -> bool {
    let term = Term::NativeLiteral(Literal::Unit);
    match type_check_with_borrow_check(&term) {
        Ok(ty) => ty == Type::unit(),
        Err(_) => false,
    }
}

fn type_variable() -> bool {
    let term = Term::var("x");
    match type_check_with_borrow_check(&term) {
        Err(_) => true,
        Ok(_) => false,
    }
}

fn type_lambda_int_to_int(_dummy: u8) -> bool {
    let term = Term::abs(
        "x",
        lambda_cicle::core::ast::Multiplicity::One,
        Type::int(),
        Term::var("x"),
    );
    match type_check_with_borrow_check(&term) {
        Ok(ty) => matches!(ty, Type::Arrow(_, arg, ret) 
            if *arg == Type::int() && *ret == Type::int()),
        Err(_) => false,
    }
}

fn type_lambda_application(_dummy: u8) -> bool {
    let term = Term::App {
        fun: Box::new(Term::abs(
            "x",
            lambda_cicle::core::ast::Multiplicity::One,
            Type::int(),
            Term::var("x"),
        )),
        arg: Box::new(Term::NativeLiteral(Literal::Int(5))),
    };
    match type_check_with_borrow_check(&term) {
        Ok(ty) => ty == Type::int(),
        Err(_) => false,
    }
}

fn type_lambda_wrong_arg_type(_dummy: u8) -> bool {
    let term = Term::App {
        fun: Box::new(Term::abs(
            "x",
            lambda_cicle::core::ast::Multiplicity::One,
            Type::int(),
            Term::var("x"),
        )),
        arg: Box::new(Term::NativeLiteral(Literal::Bool(true))),
    };
    match type_check_with_borrow_check(&term) {
        Err(_) => true,
        Ok(_) => false,
    }
}

fn type_let_binding(_dummy: u8) -> bool {
    let term = Term::let_in(
        "x",
        lambda_cicle::core::ast::Multiplicity::One,
        Type::int(),
        Term::NativeLiteral(Literal::Int(10)),
        Term::var("x"),
    );
    match type_check_with_borrow_check(&term) {
        Ok(ty) => ty == Type::int(),
        Err(_) => false,
    }
}

fn type_nested_let(_dummy: u8) -> bool {
    let term = Term::let_in(
        "x",
        lambda_cicle::core::ast::Multiplicity::One,
        Type::int(),
        Term::NativeLiteral(Literal::Int(1)),
        Term::let_in(
            "y",
            lambda_cicle::core::ast::Multiplicity::One,
            Type::int(),
            Term::NativeLiteral(Literal::Int(2)),
            Term::var("x"),
        ),
    );
    match type_check_with_borrow_check(&term) {
        Ok(ty) => ty == Type::int(),
        Err(_) => false,
    }
}

fn type_arrow_reflexive(_dummy: u8) -> bool {
    let term = Term::abs(
        "x",
        lambda_cicle::core::ast::Multiplicity::One,
        Type::int(),
        Term::var("x"),
    );
    match type_check_with_borrow_check(&term) {
        Ok(ty) => {
            ty == Type::arrow(
                Type::int(),
                lambda_cicle::core::ast::Multiplicity::One,
                Type::int(),
            )
        }
        Err(_) => false,
    }
}

#[test]
fn qc_type_int_literal() {
    quickcheck(type_int_literal as fn(u8) -> bool);
}

#[test]
fn qc_type_bool_literal() {
    quickcheck(type_bool_literal as fn(u8) -> bool);
}

#[test]
fn qc_type_unit_literal() {
    quickcheck(type_unit_literal as fn(u8) -> bool);
}

#[test]
fn qc_type_variable() {
    quickcheck(type_variable as fn() -> bool);
}

#[test]
fn qc_type_lambda_int_to_int() {
    quickcheck(type_lambda_int_to_int as fn(u8) -> bool);
}

#[test]
fn qc_type_lambda_application() {
    quickcheck(type_lambda_application as fn(u8) -> bool);
}

#[test]
fn qc_type_lambda_wrong_arg_type() {
    quickcheck(type_lambda_wrong_arg_type as fn(u8) -> bool);
}

#[test]
fn qc_type_let_binding() {
    quickcheck(type_let_binding as fn(u8) -> bool);
}

#[test]
fn qc_type_nested_let() {
    quickcheck(type_nested_let as fn(u8) -> bool);
}

#[test]
fn qc_type_arrow_reflexive() {
    quickcheck(type_arrow_reflexive as fn(u8) -> bool);
}
