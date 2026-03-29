use super::TraitError;
use crate::core::ast::{MethodName, Term, TraitName, Type};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Implementation {
    pub trait_name: TraitName,
    pub for_type: Type,
    pub methods: HashMap<MethodName, Term>,
    pub supertraits: Vec<TraitName>,
}

impl Implementation {
    pub fn new(trait_name: TraitName, for_type: Type) -> Self {
        Implementation {
            trait_name,
            for_type,
            methods: HashMap::new(),
            supertraits: Vec::new(),
        }
    }

    pub fn add_method(mut self, name: MethodName, term: Term) -> Self {
        self.methods.insert(name, term);
        self
    }

    pub fn with_supertraits(mut self, supertraits: Vec<TraitName>) -> Self {
        self.supertraits = supertraits;
        self
    }

    pub fn get_method(&self, name: &MethodName) -> Option<&Term> {
        self.methods.get(name)
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
}

impl Default for Implementation {
    fn default() -> Self {
        Implementation::new(TraitName::new(""), Type::unit())
    }
}
