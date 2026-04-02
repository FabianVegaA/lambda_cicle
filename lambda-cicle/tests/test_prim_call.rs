use lambda_cicle::{Term, translate, Evaluator, SequentialEvaluator};
use lambda_cicle::core::ast::Literal;

#[test]
fn test_prim_call_direct() {
    // Create: prim_iadd(3, 5)
    let term = Term::PrimCall {
        prim_name: "prim_iadd".to_string(),
        args: vec![
            Term::NativeLiteral(Literal::Int(3)),
            Term::NativeLiteral(Literal::Int(5)),
        ],
    };
    
    let mut net = translate(&term);
    let evaluator = SequentialEvaluator::new();
    let result = evaluator.evaluate(&mut net).unwrap();
    
    assert_eq!(result, Some(Term::NativeLiteral(Literal::Int(8))));
}
