use lambda_cicle::core::typecheck::context::ConstructorInfo;
use lambda_cicle::modules::loader::inject_prelude;
use lambda_cicle::{
    build_registry_from_decls, desugar_trait_methods, parse_program, translate, type_check,
    BorrowChecker, Decl, Evaluator, Multiplicity, SequentialEvaluator, Term, TypeContext,
};

fn eval_expr(source: &str) -> Result<Term, String> {
    let mut decls = parse_program(source).map_err(|e| format!("Parse error: {:?}", e))?;

    inject_prelude(&mut decls).map_err(|e| format!("Prelude injection error: {:?}", e))?;

    let mut ctx = TypeContext::new();

    for decl in &decls {
        if let Decl::TypeDecl {
            name: type_name,
            constructors,
            ..
        } = decl
        {
            for constructor in constructors {
                let info = ConstructorInfo {
                    type_name: type_name.clone(),
                    field_types: constructor.args.clone(),
                    result_type: lambda_cicle::Type::inductive(type_name.clone(), vec![]),
                };
                ctx = ctx.register_constructor(constructor.name.clone(), info);

                let constructor_type = if constructor.args.is_empty() {
                    lambda_cicle::Type::inductive(type_name.clone(), vec![])
                } else {
                    let result_type = lambda_cicle::Type::inductive(type_name.clone(), vec![]);
                    constructor
                        .args
                        .iter()
                        .rev()
                        .fold(result_type, |acc, arg_ty| {
                            lambda_cicle::Type::arrow(arg_ty.clone(), Multiplicity::Omega, acc)
                        })
                };
                ctx = ctx.extend(
                    constructor.name.clone(),
                    Multiplicity::Omega,
                    constructor_type,
                );
            }
        }
    }

    for decl in &decls {
        if let Decl::ValDecl { name, ty, .. } = decl {
            ctx = ctx.extend(name.clone(), Multiplicity::Omega, ty.clone());
        }
    }

    let registry = build_registry_from_decls(&decls);

    let term = decls
        .iter()
        .find_map(|d| {
            if let lambda_cicle::core::ast::Decl::ValDecl { name, term, .. } = d {
                if name == "main" {
                    return Some((**term).clone());
                }
            }
            None
        })
        .ok_or_else(|| "No main expression found".to_string())?;

    let (_ty, _) =
        type_check(&term, &ctx, &registry).map_err(|e| format!("Type error: {:?}", e))?;

    let desugared_term = desugar_trait_methods(&term, &registry);

    let mut borrow_checker = BorrowChecker::new();
    borrow_checker
        .check(&desugared_term)
        .map_err(|e| format!("Borrow check error: {:?}", e))?;

    let mut net = translate(&desugared_term);
    let evaluator = SequentialEvaluator::new();
    evaluator
        .evaluate(&mut net)
        .map_err(|e| format!("Eval error: {:?}", e))?
        .ok_or_else(|| "Evaluation returned None".to_string())
}

fn extract_int(result: Term) -> i64 {
    match result {
        Term::NativeLiteral(lambda_cicle::core::ast::Literal::Int(n)) => n,
        _ => panic!("Expected Int literal, got {:?}", result),
    }
}

fn extract_float(result: Term) -> f64 {
    match result {
        Term::NativeLiteral(lambda_cicle::core::ast::Literal::Float(n)) => n,
        _ => panic!("Expected Float literal, got {:?}", result),
    }
}

fn extract_bool(result: Term) -> bool {
    match result {
        Term::NativeLiteral(lambda_cicle::core::ast::Literal::Bool(b)) => b,
        _ => panic!("Expected Bool literal, got {:?}", result),
    }
}

fn extract_char(result: Term) -> char {
    match result {
        Term::NativeLiteral(lambda_cicle::core::ast::Literal::Char(c)) => c,
        _ => panic!("Expected Char literal, got {:?}", result),
    }
}

mod integer_arithmetic {
    use super::*;

    #[test]
    fn test_iadd_e2e() {
        let result = eval_expr("val main : Int = add 3 5").unwrap();
        assert_eq!(extract_int(result), 8);
    }

    #[test]
    fn test_isub_e2e() {
        let result = eval_expr("val main : Int = sub 10 3").unwrap();
        assert_eq!(extract_int(result), 7);
    }

    #[test]
    fn test_imul_e2e() {
        let result = eval_expr("val main : Int = mul 6 7").unwrap();
        assert_eq!(extract_int(result), 42);
    }

    #[test]
    fn test_idiv_e2e() {
        let result = eval_expr("val main : Result Int DivisionByZero = div 10 3").unwrap();
        match result {
            Term::Constructor(name, args) if name == "Ok" => {
                assert_eq!(extract_int(args[0].clone()), 3);
            }
            _ => panic!("Expected Ok(3), got {:?}", result),
        }
    }

    #[test]
    fn test_idiv_by_zero() {
        let result = eval_expr("val main : Result Int DivisionByZero = div 1 0").unwrap();
        match result {
            Term::Constructor(name, _) if name == "Err" => {}
            _ => panic!("Expected Err(DivisionByZero), got {:?}", result),
        }
    }

    #[test]
    fn test_irem_e2e() {
        let result = eval_expr("val main : Result Int DivisionByZero = rem 10 3").unwrap();
        match result {
            Term::Constructor(name, args) if name == "Ok" => {
                assert_eq!(extract_int(args[0].clone()), 1);
            }
            _ => panic!("Expected Ok(1), got {:?}", result),
        }
    }

    #[test]
    fn test_ineg_e2e() {
        let result = eval_expr("val main : Int = neg 5").unwrap();
        assert_eq!(extract_int(result), -5);
    }

    #[test]
    fn test_ihash_e2e() {
        let result = eval_expr("val main : Int = hash 42").unwrap();
        let n = extract_int(result);
        assert!(n >= 0);
    }

    #[test]
    fn test_ieq_equal_e2e() {
        let result = eval_expr("val main : Bool = eq 5 5").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_ieq_not_equal_e2e() {
        let result = eval_expr("val main : Bool = eq 5 6").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_ilt_less_e2e() {
        let result = eval_expr("val main : Bool = lt 3 5").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_ilt_not_less_e2e() {
        let result = eval_expr("val main : Bool = lt 5 3").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_igt_greater_e2e() {
        let result = eval_expr("val main : Bool = gt 5 3").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_igt_not_greater_e2e() {
        let result = eval_expr("val main : Bool = gt 3 5").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_ile_less_equal_e2e() {
        let result = eval_expr("val main : Bool = lte 3 5").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_ile_equal_e2e() {
        let result = eval_expr("val main : Bool = lte 5 5").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_ige_greater_equal_e2e() {
        let result = eval_expr("val main : Bool = gte 5 5").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_ige_greater_e2e() {
        let result = eval_expr("val main : Bool = gte 6 5").unwrap();
        assert!(extract_bool(result));
    }
}

mod float_arithmetic {
    use super::*;

    #[test]
    fn test_fadd_e2e() {
        let result = eval_expr("val main : Float = add 1.5 2.5").unwrap();
        assert!((extract_float(result) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_fsub_e2e() {
        let result = eval_expr("val main : Float = sub 5.0 3.0").unwrap();
        assert!((extract_float(result) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_fmul_e2e() {
        let result = eval_expr("val main : Float = mul 3.0 4.0").unwrap();
        assert!((extract_float(result) - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_fdiv_e2e() {
        let result = eval_expr("val main : Float = div 10.0 2.0").unwrap();
        assert!((extract_float(result) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_frem_e2e() {
        let result = eval_expr("val main : Float = rem 10.0 3.0").unwrap();
        assert!((extract_float(result) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_fneg_e2e() {
        let result = eval_expr("val main : Float = neg 5.0").unwrap();
        assert!((extract_float(result) - (-5.0)).abs() < 1e-10);
    }

    #[test]
    fn test_feq_equal_e2e() {
        let result = eval_expr("val main : Bool = eq 3.0 3.0").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_feq_not_equal_e2e() {
        let result = eval_expr("val main : Bool = eq 3.0 4.0").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_fne_e2e() {
        let result = eval_expr("val main : Bool = ne 3.0 4.0").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_flt_less_e2e() {
        let result = eval_expr("val main : Bool = lt 2.0 3.0").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_fgt_greater_e2e() {
        let result = eval_expr("val main : Bool = gt 3.0 2.0").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_fle_less_equal_e2e() {
        let result = eval_expr("val main : Bool = lte 2.0 3.0").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_fge_greater_equal_e2e() {
        let result = eval_expr("val main : Bool = gte 3.0 3.0").unwrap();
        assert!(extract_bool(result));
    }
}

mod boolean_operations {
    use super::*;

    #[test]
    fn test_not_true_e2e() {
        let result = eval_expr("val main : Bool = not True").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_not_false_e2e() {
        let result = eval_expr("val main : Bool = not False").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_and_both_true_e2e() {
        let result = eval_expr("val main : Bool = and True True").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_and_first_false_e2e() {
        let result = eval_expr("val main : Bool = and False True").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_and_second_false_e2e() {
        let result = eval_expr("val main : Bool = and True False").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_and_both_false_e2e() {
        let result = eval_expr("val main : Bool = and False False").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_or_both_false_e2e() {
        let result = eval_expr("val main : Bool = or False False").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_or_first_true_e2e() {
        let result = eval_expr("val main : Bool = or True False").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_or_second_true_e2e() {
        let result = eval_expr("val main : Bool = or False True").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_or_both_true_e2e() {
        let result = eval_expr("val main : Bool = or True True").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_beq_equal_e2e() {
        let result = eval_expr("val main : Bool = eq True True").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_beq_not_equal_e2e() {
        let result = eval_expr("val main : Bool = eq True False").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_bhash_true_e2e() {
        let result = eval_expr("val main : Int = hash True").unwrap();
        let n = extract_int(result);
        assert!(n >= 0);
    }
}

mod char_operations {
    use super::*;

    #[test]
    fn test_ceq_equal_e2e() {
        let result = eval_expr("val main : Bool = eq 'a' 'a'").unwrap();
        assert!(extract_bool(result));
    }

    #[test]
    fn test_ceq_not_equal_e2e() {
        let result = eval_expr("val main : Bool = eq 'a' 'b'").unwrap();
        assert!(!extract_bool(result));
    }

    #[test]
    fn test_chash_e2e() {
        let result = eval_expr("val main : Int = hash 'a'").unwrap();
        let n = extract_int(result);
        assert!(n >= 0);
    }

    #[test]
    fn test_ord_c_e2e() {
        let result = eval_expr("val main : Ordering = compare 'c' 'a'").unwrap();
        match result {
            Term::Constructor(name, _) if name == "GT" => {}
            _ => panic!("Expected GT, got {:?}", result),
        }
    }
}

#[test]
fn test_prim_call_translation() {
    use lambda_cicle::{Term, translate};
    use lambda_cicle::core::ast::Literal;
    use lambda_cicle::runtime::evaluator::{Evaluator, SequentialEvaluator};
    
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
