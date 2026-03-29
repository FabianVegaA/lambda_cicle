use super::{Registry, TraitError};

pub fn check_coherence(registry: &Registry) -> Result<(), TraitError> {
    let mut seen: std::collections::HashSet<(&str, &crate::core::ast::Type)> =
        std::collections::HashSet::new();

    for (trait_name, ty, _impl) in registry.iter() {
        let key = (trait_name.0.as_str(), ty);
        if seen.contains(&key) {
            return Err(TraitError::DuplicateImpl(trait_name.clone(), ty.clone()));
        }
        seen.insert(key);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::{MethodName, TraitName, Type};
    use crate::Implementation;

    #[test]
    fn test_coherence_no_duplicates() {
        let mut registry = Registry::new();
        let impl1 = Implementation::new(TraitName::new("Add"), Type::int()).add_method(
            MethodName::new("add"),
            crate::core::ast::Term::NativeLiteral(crate::core::ast::Literal::Unit),
        );

        registry.insert(impl1).unwrap();
        assert!(check_coherence(&registry).is_ok());
    }

    #[test]
    fn test_coherence_duplicate() {
        let mut registry = Registry::new();
        let impl1 = Implementation::new(TraitName::new("Add"), Type::int()).add_method(
            MethodName::new("add"),
            crate::core::ast::Term::NativeLiteral(crate::core::ast::Literal::Unit),
        );

        let impl2 = Implementation::new(TraitName::new("Add"), Type::int()).add_method(
            MethodName::new("add"),
            crate::core::ast::Term::NativeLiteral(crate::core::ast::Literal::Unit),
        );

        registry.insert(impl1).unwrap();
        registry.insert(impl2).unwrap_err();
    }
}
