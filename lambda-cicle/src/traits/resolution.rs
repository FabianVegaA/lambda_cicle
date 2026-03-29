use super::{Registry, TraitError};
use crate::core::ast::{MethodName, Term, TraitName, Type};
use std::collections::HashSet;

pub fn resolve_method(
    trait_name: &TraitName,
    ty: &Type,
    method: &MethodName,
    registry: &Registry,
) -> Result<Term, TraitError> {
    let mut visited = HashSet::new();
    resolve_method_dfs(trait_name, ty, method, registry, &mut visited)
}

fn resolve_method_dfs(
    trait_name: &TraitName,
    ty: &Type,
    method: &MethodName,
    registry: &Registry,
    visited: &mut HashSet<TraitName>,
) -> Result<Term, TraitError> {
    if visited.contains(trait_name) {
        return Err(TraitError::TraitNotFound(trait_name.clone(), ty.clone()));
    }
    visited.insert(trait_name.clone());

    if let Some(impl_) = registry.get(trait_name, ty) {
        if let Some(term) = impl_.get_method(method) {
            return Ok(term.clone());
        }
    }

    if let Some(supertraits) = registry.get_trait_declaration(trait_name) {
        for supertrait in supertraits.iter() {
            if let Ok(term) = resolve_method_dfs(supertrait, ty, method, registry, visited) {
                return Ok(term);
            }
        }
    }

    Err(TraitError::TraitNotFound(trait_name.clone(), ty.clone()))
}

pub fn resolve_method_with_cache(
    trait_name: &TraitName,
    ty: &Type,
    method: &MethodName,
    registry: &Registry,
    cache: &mut HashSet<(TraitName, Type, MethodName)>,
) -> Result<Term, TraitError> {
    let cache_key = (trait_name.clone(), ty.clone(), method.clone());

    if cache.contains(&cache_key) {
        return Err(TraitError::TraitNotFound(trait_name.clone(), ty.clone()));
    }

    cache.insert(cache_key.clone());

    let result = resolve_method(trait_name, ty, method, registry);
    cache.remove(&cache_key);

    result
}
