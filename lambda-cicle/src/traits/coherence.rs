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

pub struct DefiningModules {
    pub trait_modules: std::collections::HashMap<String, String>,
    pub type_modules: std::collections::HashMap<String, String>,
}

impl DefiningModules {
    pub fn new() -> Self {
        DefiningModules {
            trait_modules: std::collections::HashMap::new(),
            type_modules: std::collections::HashMap::new(),
        }
    }

    pub fn register_trait(&mut self, trait_name: &str, module: &str) {
        self.trait_modules
            .insert(trait_name.to_string(), module.to_string());
    }

    pub fn register_type(&mut self, type_name: &str, module: &str) {
        self.type_modules
            .insert(type_name.to_string(), module.to_string());
    }

    pub fn get_trait_module(&self, trait_name: &str) -> Option<&String> {
        self.trait_modules.get(trait_name)
    }

    pub fn get_type_module(&self, type_name: &str) -> Option<&String> {
        self.type_modules.get(type_name)
    }
}

impl Default for DefiningModules {
    fn default() -> Self {
        Self::new()
    }
}

pub fn check_orphan_rule(
    impl_module: &str,
    trait_name: &str,
    type_name: &str,
    def_modules: &DefiningModules,
) -> Result<(), TraitError> {
    let trait_defining_module = def_modules.get_trait_module(trait_name);
    let type_defining_module = def_modules.get_type_module(type_name);

    let in_trait_module = trait_defining_module
        .map(|m| m == impl_module)
        .unwrap_or(false);
    let in_type_module = type_defining_module
        .map(|m| m == impl_module)
        .unwrap_or(false);

    if !in_trait_module && !in_type_module {
        return Err(TraitError::OrphanImpl {
            impl_module: impl_module.to_string(),
            trait_name: crate::core::ast::TraitName::new(trait_name),
            type_name: type_name.to_string(),
        });
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

    #[test]
    fn test_orphan_impl_in_trait_module_legal() {
        let mut def_modules = DefiningModules::new();
        def_modules.register_trait("Add", "Std");
        def_modules.register_type("Int", "Std");

        let result = check_orphan_rule("Std", "Add", "Int", &def_modules);
        assert!(result.is_ok());
    }

    #[test]
    fn test_orphan_impl_in_type_module_legal() {
        let mut def_modules = DefiningModules::new();
        def_modules.register_trait("Add", "Std");
        def_modules.register_type("Int", "MyModule");

        let result = check_orphan_rule("MyModule", "Add", "Int", &def_modules);
        assert!(result.is_ok());
    }

    #[test]
    fn test_orphan_impl_in_other_module_illegal() {
        let mut def_modules = DefiningModules::new();
        def_modules.register_trait("Add", "Std");
        def_modules.register_type("Int", "Std");

        let result = check_orphan_rule("MyModule", "Add", "Int", &def_modules);
        assert!(result.is_err());
        match result.unwrap_err() {
            TraitError::OrphanImpl { .. } => (),
            _ => panic!("Expected OrphanImpl error"),
        }
    }
}
