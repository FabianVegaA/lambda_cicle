use crate::core::ast::types::{Multiplicity, Type};
use crate::core::ast::{Pattern, Term};
use crate::core::borrow::BorrowChecker;
use crate::core::multiplicity::semiring::Quantity;
use crate::core::typecheck::{TypeContext, TypeError};

pub fn type_check(term: &Term, ctx: &TypeContext) -> Result<(Type, TypeContext, Term), TypeError> {
    match term {
        Term::Var(name) => match ctx.get(name) {
            Some((mult, ty)) => {
                let result_ty = match mult {
                    Multiplicity::One | Multiplicity::Omega => ty.clone(),
                    Multiplicity::Borrow => Type::borrow(ty.clone()),
                    Multiplicity::Zero => ty.clone(),
                };
                Ok((result_ty, ctx.clone(), term.clone()))
            }
            None => {
                // Check if this might be a trait method call
                let possible_traits = [
                    "Add", "Sub", "Mul", "Div", "Rem", "Neg", "Eq", "Ord", "Hash", "Clone", "Copy",
                ];
                let is_possible_trait_method = possible_traits
                    .iter()
                    .any(|t| name.to_lowercase().contains(&t.to_lowercase()));

                if is_possible_trait_method {
                    Err(TypeError::UnknownVariable(format!(
                        "'{}' - trait method not found. Make sure the type has an implementation of the required trait.",
                        name
                    )))
                } else {
                    Err(TypeError::UnknownVariable(name.clone()))
                }
            }
        },
        Term::Abs {
            var,
            multiplicity,
            annot,
            body,
        } => {
            let body_ctx = ctx.extend(var.clone(), multiplicity.clone(), annot.clone());
            let (body_ty, _, _) = type_check(body, &body_ctx)?;
            let result_ty = Type::arrow(annot.clone(), multiplicity.clone(), body_ty);
            Ok((result_ty, ctx.clone(), term.clone()))
        }
        Term::App { fun, arg } => {
            let (fun_ty, ctx1, _) = type_check(fun, ctx)?;
            let (arg_ty, ctx2, _) = type_check(arg, &ctx1)?;

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
                            Ok((*ret_ty, combined_ctx, term.clone()))
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
            let (val_ty, ctx1, _) = type_check(value, &scaled_ctx)?;

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
            let (body_ty, ctx2, _) = type_check(body, &body_ctx)?;

            // Lean4 spec: HasType (Context.addCtx Γ₁ Γ₂) (Term.letTerm ...) τ₂
            // Final context is outer context (Γ₁) + context after body (Γ₂)
            let final_ctx = ctx.add(&ctx2).map_err(|_e| TypeError::BorrowContextMix)?;

            Ok((body_ty, final_ctx, term.clone()))
        }
        Term::Match { scrutinee, arms } => {
            let (_, ctx1, _) = type_check(scrutinee, ctx)?;

            let mut arm_types = Vec::new();
            let mut final_ctx = ctx1.clone();

            for arm in arms {
                let arm_ctx = extend_with_pattern(&ctx1, &arm.pattern)?;
                let (arm_ty, arm_ctx_after, _) = type_check(&arm.body, &arm_ctx)?;
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

            Ok((first.clone(), final_ctx, term.clone()))
        }
        Term::View { scrutinee, arms } => {
            let (_, ctx1, _) = type_check(scrutinee, ctx)?;

            let mut arm_types = Vec::new();
            let mut final_ctx = ctx1.clone();

            for arm in arms {
                let arm_ctx = extend_with_pattern_as_borrow(&ctx1, &arm.pattern)?;
                let (arm_ty, arm_ctx_after, _) = type_check(&arm.body, &arm_ctx)?;
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

            Ok((first.clone(), final_ctx, term.clone()))
        }
        Term::TraitMethod {
            trait_name,
            method: _,
            arg,
        } => {
            let (arg_ty, _, _) = type_check(arg, ctx)?;
            Err(TypeError::TraitNotFound(trait_name.clone(), arg_ty))
        }
        Term::Constructor(name, args, ty) => {
            let constructor_info = ctx.get_constructor(name).ok_or_else(|| {
                TypeError::InvalidApplication(format!(
                    "Unknown constructor: {} for type {:?}",
                    name, ty
                ))
            })?;

            if args.len() != constructor_info.field_types.len() {
                return Err(TypeError::InvalidApplication(format!(
                    "Constructor {} expects {} arguments, got {}",
                    name,
                    constructor_info.field_types.len(),
                    args.len()
                )));
            }

            let mut ctx = ctx.clone();
            for (arg, expected_ty) in args.iter().zip(&constructor_info.field_types) {
                let (arg_ty, new_ctx, _) = type_check(arg, &ctx)?;
                if arg_ty != *expected_ty {
                    return Err(TypeError::InvalidApplication(format!(
                        "Type mismatch in constructor {}: expected {:?}, got {:?}",
                        name, expected_ty, arg_ty
                    )));
                }
                ctx = new_ctx;
            }

            let mut typed_args = Vec::new();
            let mut ctx = ctx.clone();
            for (arg, expected_ty) in args.iter().zip(&constructor_info.field_types) {
                let (arg_ty, new_ctx, typed_arg) = type_check(arg, &ctx)?;
                if arg_ty != *expected_ty {
                    return Err(TypeError::InvalidApplication(format!(
                        "Type mismatch in constructor {}: expected {:?}, got {:?}",
                        name, expected_ty, arg_ty
                    )));
                }
                typed_args.push(typed_arg);
                ctx = new_ctx;
            }

            let typed_term = Term::Constructor(
                name.clone(),
                typed_args,
                Some(constructor_info.result_type.clone()),
            );

            Ok((constructor_info.result_type.clone(), ctx, typed_term))
        }
        Term::NativeLiteral(lit) => Ok((lit.ty(), ctx.clone(), term.clone())),
        Term::PrimCall { prim_name, args } => {
            let prim_ty = ctx
                .get(prim_name)
                .ok_or_else(|| TypeError::UnknownVariable(prim_name.clone()))?
                .1
                .clone();

            let mut current_ctx = ctx.clone();
            let mut remaining_ty = prim_ty;

            for arg in args {
                let (arg_ty, new_ctx, _) = type_check(arg, &current_ctx)?;
                current_ctx = new_ctx;

                match remaining_ty {
                    Type::Arrow(_, expected_arg_ty, ret) => {
                        if arg_ty != *expected_arg_ty {
                            return Err(TypeError::TypeMismatch {
                                expected: *expected_arg_ty.clone(),
                                found: arg_ty,
                            });
                        }
                        remaining_ty = (*ret).clone();
                    }
                    _ => {
                        return Err(TypeError::InvalidApplication(format!(
                            "Primitive {} expects more arguments",
                            prim_name
                        )));
                    }
                }
            }

            Ok((remaining_ty, current_ctx, term.clone()))
        }
    }
}

fn extend_with_pattern(ctx: &TypeContext, pattern: &Pattern) -> Result<TypeContext, TypeError> {
    match pattern {
        Pattern::Wildcard => Ok(ctx.clone()),
        Pattern::Var(name) => Ok(ctx.extend(name.clone(), Multiplicity::One, Type::unit())),
        Pattern::Constructor(name, args) => {
            let constructor_info = ctx.get_constructor(name).ok_or_else(|| {
                TypeError::InvalidApplication(format!("Unknown constructor in pattern: {}", name))
            })?;

            if args.len() != constructor_info.field_types.len() {
                return Err(TypeError::InvalidApplication(format!(
                    "Pattern constructor {} expects {} arguments, got {}",
                    name,
                    constructor_info.field_types.len(),
                    args.len()
                )));
            }

            let mut new_ctx = ctx.clone();
            for (arg, field_ty) in args.iter().zip(&constructor_info.field_types) {
                new_ctx = extend_with_pattern_with_type(&new_ctx, arg, field_ty)?;
            }
            Ok(new_ctx)
        }
    }
}

fn extend_with_pattern_with_type(
    ctx: &TypeContext,
    pattern: &Pattern,
    expected_type: &Type,
) -> Result<TypeContext, TypeError> {
    match pattern {
        Pattern::Wildcard => Ok(ctx.clone()),
        Pattern::Var(name) => {
            Ok(ctx.extend(name.clone(), Multiplicity::One, expected_type.clone()))
        }
        Pattern::Constructor(name, args) => {
            let constructor_info = ctx.get_constructor(name).ok_or_else(|| {
                TypeError::InvalidApplication(format!("Unknown constructor in pattern: {}", name))
            })?;

            if args.len() != constructor_info.field_types.len() {
                return Err(TypeError::InvalidApplication(format!(
                    "Pattern constructor {} expects {} arguments, got {}",
                    name,
                    constructor_info.field_types.len(),
                    args.len()
                )));
            }

            let mut new_ctx = ctx.clone();
            for (arg, field_ty) in args.iter().zip(&constructor_info.field_types) {
                new_ctx = extend_with_pattern_with_type(&new_ctx, arg, field_ty)?;
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
        Pattern::Constructor(_name, _args) => Err(TypeError::InvalidApplication(
            "Cannot borrow from constructor pattern".to_string(),
        )),
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
        Type::App(_, args) => {
            for arg in args {
                check_strict_positivity(arg)?;
            }
            Ok(())
        }
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
        Type::App(_, args) => {
            for arg in args {
                check_positive_occurrence(arg, type_param)?;
            }
            Ok(())
        }
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
        Type::App(_, args) => {
            for arg in args {
                check_negative_occurrence(arg, type_param)?;
            }
            Ok(())
        }
    }
}

pub fn type_check_with_borrow_check(term: &Term) -> Result<Type, TypeError> {
    let ctx = TypeContext::new();
    let (ty, _, _) = type_check(term, &ctx)?;

    let mut checker = BorrowChecker::new();
    checker.check(term)?;

    Ok(ty)
}
