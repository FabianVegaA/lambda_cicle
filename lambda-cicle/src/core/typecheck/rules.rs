use crate::core::ast::types::{Multiplicity, Type};
use crate::core::ast::{Pattern, Term};
use crate::core::borrow::BorrowChecker;
use crate::core::multiplicity::semiring::Quantity;
use crate::core::typecheck::{TypeContext, TypeError};

pub fn type_check(term: &Term, ctx: &TypeContext) -> Result<(Type, TypeContext), TypeError> {
    match term {
        Term::Var(name) => match ctx.get(name) {
            Some((mult, ty)) => {
                let result_ty = match mult {
                    Multiplicity::One | Multiplicity::Omega => ty.clone(),
                    Multiplicity::Borrow => Type::borrow(ty.clone()),
                    Multiplicity::Zero => ty.clone(),
                };
                Ok((result_ty, ctx.clone()))
            }
            None => Err(TypeError::UnknownVariable(name.clone())),
        },
        Term::Abs {
            var,
            multiplicity,
            annot,
            body,
        } => {
            let body_ctx = ctx.extend(var.clone(), multiplicity.clone(), annot.clone());
            let (body_ty, _) = type_check(body, &body_ctx)?;
            let result_ty = Type::arrow(annot.clone(), multiplicity.clone(), body_ty);
            Ok((result_ty, ctx.clone()))
        }
        Term::App { fun, arg } => {
            let (fun_ty, ctx1) = type_check(fun, ctx)?;
            let (arg_ty, ctx2) = type_check(arg, &ctx1)?;

            match fun_ty {
                Type::Arrow(mult, arg_ty_expected, ret_ty) => {
                    if arg_ty != *arg_ty_expected {
                        return Err(TypeError::TypeMismatch {
                            expected: *arg_ty_expected,
                            found: arg_ty,
                        });
                    }
                    match mult {
                        Multiplicity::One | Multiplicity::Omega | Multiplicity::Zero => {
                            let combined_ctx =
                                ctx1.add(&ctx2).map_err(|_e| TypeError::BorrowContextMix)?;
                            Ok((*ret_ty, combined_ctx))
                        }
                        Multiplicity::Borrow => Err(TypeError::MultiplicityMismatch {
                            expected: Multiplicity::One,
                            found: mult,
                        }),
                    }
                }
                _ => Err(TypeError::InvalidApplication(format!(
                    "Expected function type, found {}",
                    fun_ty
                ))),
            }
        }
        Term::Let {
            var,
            multiplicity,
            annot,
            value,
            body,
        } => {
            let q = match multiplicity {
                Multiplicity::Zero => Quantity::Zero,
                Multiplicity::One => Quantity::One,
                Multiplicity::Omega => Quantity::Omega,
                Multiplicity::Borrow => {
                    return Err(TypeError::MultiplicityMismatch {
                        expected: Multiplicity::One,
                        found: multiplicity.clone(),
                    });
                }
            };

            // Lean4 spec: HasType (Context.scale q Γ₁) e₁ τ₁
            // Scale the outer context by q for checking the value
            let scaled_ctx = ctx.scale(q).map_err(|_e| TypeError::BorrowContextMix)?;
            let (val_ty, ctx1) = type_check(value, &scaled_ctx)?;

            // If annotation is a type variable, don't require equality - allow inference
            // Otherwise, check that value type matches annotation
            let needs_check = !matches!(annot, Type::Var(_));
            if needs_check && val_ty != *annot {
                return Err(TypeError::TypeMismatch {
                    expected: annot.clone(),
                    found: val_ty,
                });
            }

            // Lean4 spec: HasType (Context.add Γ₂ {name:=x, mult:=q, type:=τ₁}) e₂ τ₂
            // Extend ctx1 (context after value) with the binding
            let body_ctx = ctx1.extend(var.clone(), multiplicity.clone(), annot.clone());
            let (body_ty, ctx2) = type_check(body, &body_ctx)?;

            // Lean4 spec: HasType (Context.addCtx Γ₁ Γ₂) (Term.letTerm ...) τ₂
            // Final context is outer context (Γ₁) + context after body (Γ₂)
            let final_ctx = ctx.add(&ctx2).map_err(|_e| TypeError::BorrowContextMix)?;

            Ok((body_ty, final_ctx))
        }
        Term::Match { scrutinee, arms } => {
            let (_, ctx1) = type_check(scrutinee, ctx)?;

            let mut arm_types = Vec::new();
            let mut final_ctx = ctx1.clone();

            for arm in arms {
                let arm_ctx = extend_with_pattern(&ctx1, &arm.pattern)?;
                let (arm_ty, arm_ctx_after) = type_check(&arm.body, &arm_ctx)?;
                arm_types.push(arm_ty);

                final_ctx = final_ctx
                    .add(&arm_ctx_after)
                    .map_err(|_e| TypeError::BorrowContextMix)?;
            }

            if arm_types.is_empty() {
                return Err(TypeError::NonExhaustivePattern);
            }

            let first = &arm_types[0];
            for ty in &arm_types[1..] {
                if ty != first {
                    return Err(TypeError::TypeMismatch {
                        expected: first.clone(),
                        found: ty.clone(),
                    });
                }
            }

            Ok((first.clone(), final_ctx))
        }
        Term::View { scrutinee, arms } => {
            let (_, ctx1) = type_check(scrutinee, ctx)?;

            let mut arm_types = Vec::new();
            let mut final_ctx = ctx1.clone();

            for arm in arms {
                let arm_ctx = extend_with_pattern_as_borrow(&ctx1, &arm.pattern)?;
                let (arm_ty, arm_ctx_after) = type_check(&arm.body, &arm_ctx)?;
                arm_types.push(arm_ty);

                final_ctx = final_ctx
                    .add(&arm_ctx_after)
                    .map_err(|_e| TypeError::BorrowContextMix)?;
            }

            if arm_types.is_empty() {
                return Err(TypeError::NonExhaustivePattern);
            }

            let first = &arm_types[0];
            for ty in &arm_types[1..] {
                if ty != first {
                    return Err(TypeError::TypeMismatch {
                        expected: first.clone(),
                        found: ty.clone(),
                    });
                }
            }

            Ok((first.clone(), final_ctx))
        }
        Term::TraitMethod {
            trait_name,
            method: _,
            arg,
        } => {
            let (arg_ty, _) = type_check(arg, ctx)?;
            Err(TypeError::TraitNotFound(trait_name.clone(), arg_ty))
        }
        Term::Constructor(name, args) => {
            let mut ctx = ctx.clone();
            for arg in args {
                let (_, new_ctx) = type_check(arg, &ctx)?;
                ctx = new_ctx;
            }
            Err(TypeError::InvalidApplication(format!(
                "Unknown constructor: {}",
                name
            )))
        }
        Term::NativeLiteral(lit) => Ok((lit.ty(), ctx.clone())),
    }
}

fn extend_with_pattern(ctx: &TypeContext, pattern: &Pattern) -> Result<TypeContext, TypeError> {
    match pattern {
        Pattern::Wildcard => Ok(ctx.clone()),
        Pattern::Var(name) => Ok(ctx.extend(name.clone(), Multiplicity::One, Type::unit())),
        Pattern::Constructor(_name, args) => {
            let mut new_ctx = ctx.clone();
            for arg in args {
                new_ctx = extend_with_pattern(&new_ctx, arg)?;
            }
            Ok(new_ctx)
        }
    }
}

fn extend_with_pattern_as_borrow(
    ctx: &TypeContext,
    pattern: &Pattern,
) -> Result<TypeContext, TypeError> {
    match pattern {
        Pattern::Wildcard => Ok(ctx.clone()),
        Pattern::Var(name) => Ok(ctx.extend(name.clone(), Multiplicity::Borrow, Type::unit())),
        Pattern::Constructor(_name, args) => {
            let mut new_ctx = ctx.clone();
            for arg in args {
                new_ctx = extend_with_pattern_as_borrow(&new_ctx, arg)?;
            }
            Ok(new_ctx)
        }
    }
}

impl From<Term> for Type {
    fn from(term: Term) -> Type {
        match term {
            Term::NativeLiteral(lit) => lit.ty(),
            _ => Type::unit(),
        }
    }
}

pub fn check_strict_positivity(ty: &Type) -> Result<(), TypeError> {
    match ty {
        Type::Inductive(name, params) => {
            if let Some(first_param) = params.first() {
                check_positive_occurrence(first_param, &name.0)?;
            }
            Ok(())
        }
        Type::Arrow(_, arg, ret) => {
            check_strict_positivity(arg)?;
            check_strict_positivity(ret)?;
            Ok(())
        }
        Type::Forall(_, ty) => check_strict_positivity(ty),
        Type::TraitConstraint(_, ty) => check_strict_positivity(ty),
        Type::Borrow(ty) => check_strict_positivity(ty),
        Type::Product(left, right) => {
            check_strict_positivity(left)?;
            check_strict_positivity(right)?;
            Ok(())
        }
        Type::Sum(left, right) => {
            check_strict_positivity(left)?;
            check_strict_positivity(right)?;
            Ok(())
        }
        Type::Native(_) => Ok(()),
        Type::Var(_) => Ok(()),
    }
}

fn check_positive_occurrence(ty: &Type, type_param: &str) -> Result<(), TypeError> {
    match ty {
        Type::Arrow(_, arg, ret) => {
            check_negative_occurrence(arg, type_param)?;
            check_positive_occurrence(ret, type_param)?;
            Ok(())
        }
        Type::Forall(_, ty) => check_positive_occurrence(ty, type_param),
        Type::TraitConstraint(_, ty) => check_positive_occurrence(ty, type_param),
        Type::Borrow(ty) => check_positive_occurrence(ty, type_param),
        Type::Product(left, right) => {
            check_positive_occurrence(left, type_param)?;
            check_positive_occurrence(right, type_param)
        }
        Type::Sum(left, right) => {
            check_positive_occurrence(left, type_param)?;
            check_positive_occurrence(right, type_param)
        }
        Type::Inductive(name, params) => {
            for param in params {
                if let Type::Inductive(n, _) = param {
                    if n.0 == type_param {
                        return Err(TypeError::StrictPositivityViolation(name.clone()));
                    }
                }
                check_positive_occurrence(param, type_param)?;
            }
            Ok(())
        }
        Type::Native(_) => Ok(()),
        Type::Var(_) => Ok(()),
    }
}

fn check_negative_occurrence(ty: &Type, type_param: &str) -> Result<(), TypeError> {
    match ty {
        Type::Arrow(_, arg, ret) => {
            check_negative_occurrence(arg, type_param)?;
            check_negative_occurrence(ret, type_param)?;
            Ok(())
        }
        Type::Forall(_, ty) => check_negative_occurrence(ty, type_param),
        Type::TraitConstraint(_, ty) => check_negative_occurrence(ty, type_param),
        Type::Borrow(ty) => check_negative_occurrence(ty, type_param),
        Type::Product(left, right) => {
            check_negative_occurrence(left, type_param)?;
            check_negative_occurrence(right, type_param)
        }
        Type::Sum(left, right) => {
            check_negative_occurrence(left, type_param)?;
            check_negative_occurrence(right, type_param)
        }
        Type::Inductive(_, params) => {
            for param in params {
                check_negative_occurrence(param, type_param)?;
            }
            Ok(())
        }
        Type::Native(_) => Ok(()),
        Type::Var(_) => Ok(()),
    }
}

pub fn type_check_with_borrow_check(term: &Term) -> Result<Type, TypeError> {
    let ctx = TypeContext::new();
    let (ty, _) = type_check(term, &ctx)?;

    let mut checker = BorrowChecker::new();
    checker.check(term)?;

    Ok(ty)
}
