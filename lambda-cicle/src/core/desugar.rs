use crate::core::ast::terms::Term;
use crate::core::ast::types::{MethodName, TraitName, Type};
use crate::traits::registry::{Registry, TraitMethodImpl};

/// Desugar trait method calls into direct primitive calls when possible.
///
/// This pass runs after type-checking and before translation to the interaction net.
/// It resolves trait method calls like `add 3 5` to `PrimCall { "prim_iadd", [3, 5] }`.
pub fn desugar_term(term: &Term, registry: &Registry) -> Term {
    desugar_recursive(term, registry)
}

fn desugar_recursive(term: &Term, registry: &Registry) -> Term {
    match term {
        Term::Var(_) | Term::NativeLiteral(_) => term.clone(),

        Term::Abs {
            var,
            multiplicity,
            annot,
            body,
        } => Term::Abs {
            var: var.clone(),
            multiplicity: multiplicity.clone(),
            annot: annot.clone(),
            body: Box::new(desugar_recursive(body, registry)),
        },

        Term::App { fun, arg } => {
            let (args, base) = collect_app_chain(term);

            // First, desugar all arguments so we can determine their types
            let desugared_args: Vec<Term> = args
                .iter()
                .map(|a| desugar_recursive(a, registry))
                .collect();

            // Try to resolve trait method call if base is a Var
            if let Term::Var(name) = &base {
                if let Some(impl_) = resolve_trait_method_call(name, &desugared_args, registry) {
                    let result = apply_resolution(impl_, desugared_args, registry);
                    return desugar_recursive(&result, registry);
                }
            }

            // Also check for TraitMethod base (existing handling)
            if let Term::TraitMethod {
                trait_name,
                method,
                arg: trait_arg,
            } = base
            {
                if let Some((Term::Var(var_name), _)) =
                    resolve_trait_method_impl(&trait_name.0, &method.0, &trait_arg, registry)
                {
                    if var_name.starts_with("prim_") {
                        let mut all_args = vec![desugar_recursive(&trait_arg, registry)];
                        all_args.extend(args.into_iter().map(|a| desugar_recursive(&a, registry)));
                        return Term::PrimCall {
                            prim_name: var_name,
                            args: all_args,
                        };
                    }
                }
            }

            // No resolution found, desugar normally
            let desugared_fun = desugar_recursive(fun, registry);
            let desugared_arg = desugar_recursive(arg, registry);
            Term::App {
                fun: Box::new(desugared_fun),
                arg: Box::new(desugared_arg),
            }
        }

        Term::Let {
            var,
            multiplicity,
            annot,
            value,
            body,
        } => Term::Let {
            var: var.clone(),
            multiplicity: multiplicity.clone(),
            annot: annot.clone(),
            value: Box::new(desugar_recursive(value, registry)),
            body: Box::new(desugar_recursive(body, registry)),
        },

        Term::Match { scrutinee, arms } => Term::Match {
            scrutinee: Box::new(desugar_recursive(scrutinee, registry)),
            arms: arms
                .iter()
                .map(|arm| crate::core::ast::terms::Arm {
                    pattern: arm.pattern.clone(),
                    body: Box::new(desugar_recursive(&arm.body, registry)),
                })
                .collect(),
        },

        Term::View { scrutinee, arms } => Term::View {
            scrutinee: Box::new(desugar_recursive(scrutinee, registry)),
            arms: arms
                .iter()
                .map(|arm| crate::core::ast::terms::Arm {
                    pattern: arm.pattern.clone(),
                    body: Box::new(desugar_recursive(&arm.body, registry)),
                })
                .collect(),
        },

        Term::TraitMethod {
            trait_name,
            method,
            arg,
        } => {
            let desugared_arg = desugar_recursive(arg, registry);

            if let Some((impl_term, _)) =
                resolve_trait_method_impl(&trait_name.0, &method.0, &desugared_arg, registry)
            {
                if let Term::Var(ref var_name) = impl_term {
                    if var_name.starts_with("prim_") {
                        return Term::PrimCall {
                            prim_name: var_name.clone(),
                            args: vec![desugared_arg],
                        };
                    }
                }

                return Term::App {
                    fun: Box::new(impl_term),
                    arg: Box::new(desugared_arg),
                };
            }

            Term::TraitMethod {
                trait_name: trait_name.clone(),
                method: method.clone(),
                arg: Box::new(desugared_arg),
            }
        }

        Term::Constructor(name, args) => Term::Constructor(
            name.clone(),
            args.iter()
                .map(|a| desugar_recursive(a, registry))
                .collect(),
        ),

        Term::PrimCall { prim_name, args } => Term::PrimCall {
            prim_name: prim_name.clone(),
            args: args
                .iter()
                .map(|a| desugar_recursive(a, registry))
                .collect(),
        },
    }
}

/// Resolve a trait method call by method name and argument types.
///
/// Given `Var("add")` applied to `[Int(3), Int(5)]`, this searches
/// the registry for a trait method named "add" that matches the argument types.
fn resolve_trait_method_call(
    method_name: &str,
    args: &[Term],
    registry: &Registry,
) -> Option<TraitMethodImpl> {
    let arg_types: Vec<Type> = args
        .iter()
        .filter_map(|a| get_type_of_term_with_registry(a, registry))
        .collect();

    if let Some((_trait_name, _for_type, method_term)) =
        registry.find_method_by_name(method_name, &arg_types)
    {
        if let Term::Var(var_name) = &method_term {
            if var_name.starts_with("prim_") {
                return Some(TraitMethodImpl::Primitive(var_name.clone()));
            }
        }

        return Some(TraitMethodImpl::Term(method_term.clone()));
    }

    None
}

/// Apply a resolved trait method implementation to arguments.
fn apply_resolution(impl_: TraitMethodImpl, args: Vec<Term>, registry: &Registry) -> Term {
    match impl_ {
        TraitMethodImpl::Primitive(prim_name) => {
            use crate::runtime::primitives::prim_name_to_op;
            if let Some(op) = prim_name_to_op(&prim_name) {
                let arity = op.arity();
                if args.len() == arity {
                    return Term::PrimCall { prim_name, args };
                } else if args.len() < arity {
                    // Partial application - create a lambda that will be applied later
                    // This lambda, when applied to remaining args, will create the PrimCall
                    let remaining_args: Vec<String> = (0..(arity - args.len()))
                        .map(|i| format!("_arg{}", i))
                        .collect();

                    // Build the body: prim_call prim_name [existing_args..., _arg0, _arg1, ...]
                    let mut prim_args: Vec<Term> = args.clone();
                    for arg_name in &remaining_args {
                        prim_args.push(Term::Var(arg_name.clone()));
                    }

                    let mut body: Term = Term::PrimCall {
                        prim_name: prim_name.clone(),
                        args: prim_args,
                    };

                    // Wrap in lambdas for remaining arguments
                    for arg_name in remaining_args.iter().rev() {
                        body = Term::Abs {
                            var: arg_name.clone(),
                            multiplicity: crate::core::ast::Multiplicity::One,
                            annot: crate::core::ast::Type::int(),
                            body: Box::new(body),
                        };
                    }

                    // Return the lambda WITHOUT calling desugar_recursive
                    // The outer App will handle the remaining arguments
                    return body;
                }
            }
            Term::PrimCall { prim_name, args }
        }
        TraitMethodImpl::Term(term) => {
            let mut result = term;
            for arg in args.into_iter().rev() {
                result = Term::App {
                    fun: Box::new(result),
                    arg: Box::new(arg),
                };
            }
            desugar_recursive(&result, registry)
        }
    }
}

/// Collect a chain of applications: ((f a) b) c → (f, [a, b, c])
fn collect_app_chain(term: &Term) -> (Vec<Term>, Term) {
    fn go(term: &Term, acc: &mut Vec<Term>) -> Term {
        match term {
            Term::App { fun, arg } => {
                let base = go(fun, acc);
                acc.push((**arg).clone());
                base
            }
            _ => term.clone(),
        }
    }

    let mut args = Vec::new();
    let base = go(term, &mut args);
    (args, base)
}

/// Resolve a trait method to its implementation term.
/// Returns None if the trait/method/implementation is not found.
fn resolve_trait_method_impl(
    trait_name: &str,
    method_name: &str,
    arg: &Term,
    registry: &Registry,
) -> Option<(Term, Type)> {
    use crate::core::ast::types::{MethodName, TraitName};

    let arg_type = get_type_of_term(arg)?;

    let impl_ = registry.get(&TraitName::new(trait_name), &arg_type)?;

    let method_term = impl_.get_method(&MethodName::new(method_name))?;

    Some((method_term.clone(), arg_type))
}

/// Get the type of a term (for desugaring purposes)
fn get_type_of_term(term: &Term) -> Option<Type> {
    match term {
        Term::NativeLiteral(lit) => Some(lit.ty()),
        Term::Var(_) => None,
        _ => term.get_type(),
    }
}

/// Get the type of a term using the registry for PrimCall terms
fn get_type_of_term_with_registry(term: &Term, registry: &Registry) -> Option<Type> {
    match term {
        Term::NativeLiteral(lit) => Some(lit.ty()),
        Term::Var(_) => None,
        Term::PrimCall { prim_name, args } => {
            let prim_type = prim_name_to_type(prim_name)?;
            let mut remaining = prim_type;
            for _arg in args {
                if let Type::Arrow(_, _, ret) = remaining {
                    remaining = *ret;
                } else {
                    return None;
                }
            }
            Some(remaining)
        }
        _ => term.get_type(),
    }
}

/// Map primitive names to their types
fn prim_name_to_type(prim_name: &str) -> Option<Type> {
    use crate::core::ast::types::Multiplicity;

    match prim_name {
        "prim_iadd" | "prim_isub" | "prim_imul" | "prim_irem" => Some(Type::arrow(
            Type::int(),
            Multiplicity::One,
            Type::arrow(Type::int(), Multiplicity::One, Type::int()),
        )),
        "prim_idiv" => {
            let division_by_zero = Type::inductive("DivisionByZero".to_string(), vec![]);
            Some(Type::arrow(
                Type::int(),
                Multiplicity::One,
                Type::arrow(
                    Type::int(),
                    Multiplicity::One,
                    Type::inductive("Result".to_string(), vec![Type::int(), division_by_zero]),
                ),
            ))
        }
        "prim_fadd" | "prim_fsub" | "prim_fmul" | "prim_fdiv" | "prim_frem" => Some(Type::arrow(
            Type::float(),
            Multiplicity::One,
            Type::arrow(Type::float(), Multiplicity::One, Type::float()),
        )),
        "prim_ieq" | "prim_igt" | "prim_ilt" | "prim_ige" | "prim_ile" => Some(Type::arrow(
            Type::int(),
            Multiplicity::One,
            Type::arrow(
                Type::int(),
                Multiplicity::One,
                Type::inductive("Bool".to_string(), vec![]),
            ),
        )),
        "prim_feq" | "prim_fgt" | "prim_flt" | "prim_fge" | "prim_fle" => Some(Type::arrow(
            Type::float(),
            Multiplicity::One,
            Type::arrow(
                Type::float(),
                Multiplicity::One,
                Type::inductive("Bool".to_string(), vec![]),
            ),
        )),
        _ => None,
    }
}
