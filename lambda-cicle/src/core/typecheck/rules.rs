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

            let scaled_ctx = ctx.scale(q).map_err(|_e| TypeError::BorrowContextMix)?;
            let (val_ty, ctx1) = type_check(value, &scaled_ctx)?;

            if val_ty != *annot {
                return Err(TypeError::TypeMismatch {
                    expected: annot.clone(),
                    found: val_ty,
                });
            }

            let body_ctx = ctx1.extend(var.clone(), multiplicity.clone(), annot.clone());
            let (body_ty, ctx2) = type_check(body, &body_ctx)?;

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
        Term::BinaryOp { op, left, right } => {
            let (left_ty, ctx1) = type_check(left, ctx)?;
            let (right_ty, ctx2) = type_check(right, &ctx1)?;

            if left_ty != right_ty {
                return Err(TypeError::TypeMismatch {
                    expected: left_ty,
                    found: right_ty,
                });
            }

            let result_ty = match op {
                crate::core::ast::BinOp::Add
                | crate::core::ast::BinOp::Sub
                | crate::core::ast::BinOp::Mul
                | crate::core::ast::BinOp::Div
                | crate::core::ast::BinOp::Mod => left_ty,
                crate::core::ast::BinOp::Eq
                | crate::core::ast::BinOp::Ne
                | crate::core::ast::BinOp::Lt
                | crate::core::ast::BinOp::Gt
                | crate::core::ast::BinOp::Le
                | crate::core::ast::BinOp::Ge
                | crate::core::ast::BinOp::And
                | crate::core::ast::BinOp::Or => Type::bool(),
            };

            let final_ctx = ctx1.add(&ctx2).map_err(|_e| TypeError::BorrowContextMix)?;

            Ok((result_ty, final_ctx))
        }
        Term::UnaryOp { op: _, arg } => {
            let (arg_ty, final_ctx) = type_check(arg, ctx)?;
            Ok((arg_ty, final_ctx))
        }
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

pub fn type_check_with_borrow_check(term: &Term) -> Result<Type, TypeError> {
    let ctx = TypeContext::new();
    let (ty, _) = type_check(term, &ctx)?;

    let checker = BorrowChecker::new();
    checker.check(term)?;

    Ok(ty)
}
