use super::TraitError;
use crate::core::ast::types::Type;
use crate::core::ast::{MethodName, Term, TraitName};
use std::collections::HashMap;

/// Result of resolving a trait method call.
/// Can be either a primitive operation or a term (lambda).
#[derive(Debug, Clone)]
pub enum TraitMethodImpl {
    Primitive(String), // e.g., "prim_iadd"
    Term(Term),        // e.g., \x. \y. match ... for Ord.compare
}

#[derive(Debug, Clone)]
pub struct Implementation {
    pub trait_name: TraitName,
    pub for_type: Type,
    pub methods: HashMap<MethodName, Term>,
    pub method_types: HashMap<MethodName, Type>,
    pub supertraits: Vec<TraitName>,
}

impl Implementation {
    pub fn new(trait_name: TraitName, for_type: Type) -> Self {
        Implementation {
            trait_name,
            for_type,
            methods: HashMap::new(),
            method_types: HashMap::new(),
            supertraits: Vec::new(),
        }
    }

    pub fn add_method(mut self, name: MethodName, term: Term) -> Self {
        self.methods.insert(name, term);
        self
    }

    pub fn with_method_type(mut self, name: MethodName, ty: Type) -> Self {
        self.method_types.insert(name, ty);
        self
    }

    pub fn with_supertraits(mut self, supertraits: Vec<TraitName>) -> Self {
        self.supertraits = supertraits;
        self
    }

    pub fn get_method(&self, name: &MethodName) -> Option<&Term> {
        self.methods.get(name)
    }

    pub fn get_method_type(&self, name: &MethodName) -> Option<&Type> {
        self.method_types.get(name)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Registry {
    impls: HashMap<(TraitName, Type), Implementation>,
    trait_decls: HashMap<TraitName, Vec<TraitName>>,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            impls: HashMap::new(),
            trait_decls: HashMap::new(),
        }
    }

    pub fn declare_trait(
        &mut self,
        name: TraitName,
        supertraits: Vec<TraitName>,
    ) -> Result<(), TraitError> {
        self.trait_decls.insert(name, supertraits);
        Ok(())
    }

    pub fn insert(&mut self, implementation: Implementation) -> Result<(), TraitError> {
        let key = (
            implementation.trait_name.clone(),
            implementation.for_type.clone(),
        );

        if self.impls.contains_key(&key) {
            return Err(TraitError::DuplicateImpl(
                implementation.trait_name,
                implementation.for_type,
            ));
        }

        self.impls.insert(key, implementation);
        Ok(())
    }

    pub fn get(&self, trait_name: &TraitName, ty: &Type) -> Option<&Implementation> {
        self.impls.get(&(trait_name.clone(), ty.clone()))
    }

    pub fn get_trait_declaration(&self, trait_name: &TraitName) -> Option<&Vec<TraitName>> {
        self.trait_decls.get(trait_name)
    }

    pub fn all_impls(&self) -> impl Iterator<Item = &Implementation> {
        self.impls.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TraitName, &Type, &Implementation)> {
        self.impls.iter().map(|(k, v)| (&k.0, &k.1, v))
    }

    /// Find a method by name across all implementations.
    ///
    /// Searches through all impls to find one that:
    /// 1. Has a method with the given name
    /// 2. The method's type matches the argument types (via unification)
    /// 3. After applying the substitution, the resolved method type is consistent with for_type
    ///
    /// Returns (trait_name, for_type, method_term) if found.
    pub fn find_method_by_name(
        &self,
        method_name: &str,
        arg_types: &[Type],
    ) -> Option<(TraitName, Type, Term)> {
        for ((trait_name, for_type), impl_) in &self.impls {
            if let Some(method_term) = impl_.methods.get(&MethodName::new(method_name)) {
                if let Some(method_ty) = impl_.method_types.get(&MethodName::new(method_name)) {
                    if let Some(subst) = type_matches(method_ty, arg_types) {
                        // Substitute the type variables in for_type
                        let resolved_for_type = apply_subst(for_type, &subst);

                        // The resolved for_type should be a concrete type (Native or Inductive)
                        // If it's still a type variable, the unification was invalid
                        if !matches!(resolved_for_type, Type::Var(_)) {
                            return Some((
                                trait_name.clone(),
                                for_type.clone(),
                                method_term.clone(),
                            ));
                        }
                    }
                }
            }
        }
        None
    }
}

/// Apply a type substitution to a type
fn apply_subst(ty: &Type, subst: &TypeSubst) -> Type {
    match ty {
        Type::Var(v) => subst.get(v).cloned().unwrap_or_else(|| ty.clone()),
        Type::Arrow(m, a, b) => {
            Type::arrow(apply_subst(a, subst), m.clone(), apply_subst(b, subst))
        }
        Type::Forall(v, inner) => Type::Forall(v.clone(), inner.clone()),
        Type::TraitConstraint(t, inner) => {
            Type::trait_constraint(t.clone(), apply_subst(inner, subst))
        }
        Type::Inductive(n, params) => Type::inductive(
            n.0.clone(),
            params.iter().map(|p| apply_subst(p, subst)).collect(),
        ),
        Type::Borrow(inner) => Type::Borrow(Box::new(apply_subst(inner, subst))),
        Type::Product(a, b) => Type::Product(
            Box::new(apply_subst(a, subst)),
            Box::new(apply_subst(b, subst)),
        ),
        Type::Sum(a, b) => Type::Sum(
            Box::new(apply_subst(a, subst)),
            Box::new(apply_subst(b, subst)),
        ),
        Type::App(ty, args) => Type::App(
            Box::new(apply_subst(ty, subst)),
            args.iter().map(|a| apply_subst(a, subst)).collect(),
        ),
        Type::Native(_) => ty.clone(),
    }
}

impl Default for Implementation {
    fn default() -> Self {
        Implementation::new(TraitName::new(""), Type::unit())
    }
}

/// Check if a method type matches the provided argument types.
/// Returns the type substitution if matching succeeds.
fn type_matches(method_ty: &Type, arg_types: &[Type]) -> Option<TypeSubst> {
    let mut subst = TypeSubst::new();
    if try_match_types(method_ty, arg_types, &mut subst) {
        Some(subst)
    } else {
        None
    }
}

#[derive(Default, Debug)]
struct TypeSubst {
    vars: HashMap<String, Type>,
}

impl TypeSubst {
    fn new() -> Self {
        TypeSubst {
            vars: HashMap::new(),
        }
    }

    fn insert(&mut self, var: String, ty: Type) -> bool {
        // If variable already has a binding, check consistency
        if let Some(existing) = self.vars.get(&var) {
            existing == &ty
        } else {
            self.vars.insert(var, ty);
            true
        }
    }

    fn get(&self, var: &str) -> Option<&Type> {
        self.vars.get(var)
    }
}

/// Try to match a method type with argument types, populating the substitution.
fn try_match_types(method_ty: &Type, arg_types: &[Type], subst: &mut TypeSubst) -> bool {
    match (method_ty, arg_types) {
        (Type::Arrow(_, param_ty, ret_ty), [first, rest @ ..]) => {
            if !match_type_with_subst(param_ty, first, subst) {
                return false;
            }
            try_match_types(ret_ty, rest, subst)
        }
        (ty, []) => {
            // No more args to match, check if type is fully concrete or all vars substituted
            is_concrete_or_vars_resolved(ty, subst)
        }
        _ => false,
    }
}

/// Match a parameter type with an argument type, updating the substitution.
fn match_type_with_subst(param_ty: &Type, arg_ty: &Type, subst: &mut TypeSubst) -> bool {
    match (param_ty, arg_ty) {
        (Type::Var(v), _) => {
            // If param is a type variable, add to substitution or check consistency
            if let Some(existing) = subst.get(v) {
                existing == arg_ty
            } else {
                subst.insert(v.clone(), arg_ty.clone());
                true
            }
        }
        (Type::Arrow(m1, a1, r1), Type::Arrow(m2, a2, r2)) => {
            m1 == m2 && match_type_with_subst(a1, a2, subst) && match_type_with_subst(r1, r2, subst)
        }
        (Type::TraitConstraint(t1, inner1), Type::TraitConstraint(t2, inner2)) => {
            t1 == t2 && match_type_with_subst(inner1, inner2, subst)
        }
        (Type::Inductive(n1, params1), Type::Inductive(n2, params2)) => {
            n1 == n2
                && params1.len() == params2.len()
                && params1
                    .iter()
                    .zip(params2.iter())
                    .all(|(p1, p2)| match_type_with_subst(p1, p2, subst))
        }
        (Type::Borrow(inner1), Type::Borrow(inner2)) => {
            match_type_with_subst(inner1, inner2, subst)
        }
        (Type::Product(a1, b1), Type::Product(a2, b2)) => {
            match_type_with_subst(a1, a2, subst) && match_type_with_subst(b1, b2, subst)
        }
        (Type::Sum(a1, b1), Type::Sum(a2, b2)) => {
            match_type_with_subst(a1, a2, subst) && match_type_with_subst(b1, b2, subst)
        }
        (Type::App(ty1, args1), Type::App(ty2, args2)) => {
            ty1 == ty2
                && args1.len() == args2.len()
                && args1
                    .iter()
                    .zip(args2.iter())
                    .all(|(a1, a2)| match_type_with_subst(a1, a2, subst))
        }
        _ => param_ty == arg_ty,
    }
}

/// Check if a type is concrete or all its variables are resolved in the substitution
fn is_concrete_or_vars_resolved(ty: &Type, subst: &TypeSubst) -> bool {
    match ty {
        Type::Var(v) => subst.get(v).is_some(),
        Type::Forall(v, inner) => {
            let resolved = subst.get(v);
            if resolved.is_some() {
                is_concrete_or_vars_resolved(inner, subst)
            } else {
                false
            }
        }
        Type::Arrow(_, a, b) => {
            is_concrete_or_vars_resolved(a, subst) && is_concrete_or_vars_resolved(b, subst)
        }
        Type::TraitConstraint(_, inner) => is_concrete_or_vars_resolved(inner, subst),
        Type::Inductive(_, params) => params
            .iter()
            .all(|p| is_concrete_or_vars_resolved(p, subst)),
        Type::Borrow(inner) => is_concrete_or_vars_resolved(inner, subst),
        Type::Product(a, b) => {
            is_concrete_or_vars_resolved(a, subst) && is_concrete_or_vars_resolved(b, subst)
        }
        Type::Sum(a, b) => {
            is_concrete_or_vars_resolved(a, subst) && is_concrete_or_vars_resolved(b, subst)
        }
        Type::App(ty, args) => {
            is_concrete_or_vars_resolved(ty, subst)
                && args.iter().all(|a| is_concrete_or_vars_resolved(a, subst))
        }
        Type::Native(_) => true,
    }
}

/// Check if an argument type matches a parameter type (with unification)
#[allow(dead_code)]
fn matches_concrete(arg_ty: &Type, param_ty: &Type) -> bool {
    // If param is a type variable, it matches anything
    if matches!(param_ty, Type::Var(_)) {
        return true;
    }
    // Otherwise, types must be equal
    arg_ty == param_ty
}

/// Build a trait registry from a list of declarations.
///
/// This extracts all `ImplDecl` and `TraitDecl` declarations and builds
/// a Registry that can be used for trait method resolution.
pub fn build_registry_from_decls(decls: &[crate::core::ast::Decl]) -> Registry {
    use crate::core::ast::Decl;

    let mut registry = Registry::new();

    // First pass: declare all traits (to handle supertraits)
    for decl in decls {
        if let Decl::TraitDecl {
            name, supertrait, ..
        } = decl
        {
            let supertraits = supertrait.iter().map(|(t, _)| t.clone()).collect();
            let _ = registry.declare_trait(TraitName::new(name), supertraits);
        }
    }

    // Build a map of trait declarations for looking up method types
    let mut trait_methods: HashMap<(String, String), Type> = HashMap::new();
    for decl in decls {
        if let Decl::TraitDecl {
            name: trait_name,
            methods,
            ..
        } = decl
        {
            for method in methods {
                trait_methods.insert(
                    (trait_name.clone(), method.name.0.clone()),
                    method.ty.clone(),
                );
            }
        }
    }

    // Second pass: insert all implementations
    for decl in decls {
        if let Decl::ImplDecl {
            ty,
            trait_name,
            methods,
            ..
        } = decl
        {
            let mut impl_block = Implementation::new(trait_name.clone(), ty.clone());

            for method in methods {
                // If method type is Unit (not specified), look it up from trait declaration
                let method_ty = if method.ty == Type::unit() {
                    trait_methods
                        .get(&(trait_name.0.clone(), method.name.0.clone()))
                        .cloned()
                        .unwrap_or_else(|| {
                            // Fallback: try to infer from impl type
                            substitute_type_vars(
                                &trait_methods
                                    .get(&(trait_name.0.clone(), method.name.0.clone()))
                                    .cloned()
                                    .unwrap_or(Type::unit()),
                                &ty,
                            )
                        })
                } else {
                    method.ty.clone()
                };

                impl_block = impl_block.add_method(method.name.clone(), (*method.term).clone());
                impl_block = impl_block.with_method_type(method.name.clone(), method_ty);
            }

            let _ = registry.insert(impl_block);
        }
    }

    registry
}

/// Substitute type variables in a type with concrete types.
fn substitute_type_vars(ty: &Type, concrete_type: &Type) -> Type {
    match ty {
        Type::Arrow(mult, param, ret) => Type::Arrow(
            mult.clone(),
            Box::new(substitute_type_vars(param, concrete_type)),
            Box::new(substitute_type_vars(ret, concrete_type)),
        ),
        Type::Var(_v) => concrete_type.clone(),
        Type::Forall(v, inner) => Type::Forall(
            v.clone(),
            Box::new(substitute_type_vars(inner, concrete_type)),
        ),
        Type::TraitConstraint(t, inner) => Type::TraitConstraint(
            t.clone(),
            Box::new(substitute_type_vars(inner, concrete_type)),
        ),
        Type::Inductive(name, params) => Type::Inductive(
            name.clone(),
            params
                .iter()
                .map(|p| substitute_type_vars(p, concrete_type))
                .collect(),
        ),
        Type::Borrow(inner) => Type::Borrow(Box::new(substitute_type_vars(inner, concrete_type))),
        Type::Product(a, b) => Type::Product(
            Box::new(substitute_type_vars(a, concrete_type)),
            Box::new(substitute_type_vars(b, concrete_type)),
        ),
        Type::Sum(a, b) => Type::Sum(
            Box::new(substitute_type_vars(a, concrete_type)),
            Box::new(substitute_type_vars(b, concrete_type)),
        ),
        Type::App(ty, args) => Type::App(
            Box::new(substitute_type_vars(ty, concrete_type)),
            args.iter()
                .map(|a| substitute_type_vars(a, concrete_type))
                .collect(),
        ),
        Type::Native(_) => ty.clone(),
    }
}
