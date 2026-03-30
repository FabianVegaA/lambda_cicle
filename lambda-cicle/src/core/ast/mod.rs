pub mod patterns;
pub mod terms;
pub mod types;

pub use types::{MethodName, Multiplicity, NativeKind, TraitName, Type, TypeName};

pub use terms::{Arm, Decl, Literal, MethodDef, MethodSig, Term, UseMode, Visibility};

pub use patterns::Pattern;

use hashbrown::HashMap;
use std::ops::Index;

#[derive(Debug, Clone, Default)]
pub struct TypeArena {
    types: Vec<Type>,
}

impl TypeArena {
    pub fn new() -> TypeArena {
        TypeArena { types: Vec::new() }
    }

    pub fn alloc(&mut self, ty: Type) -> TypeId {
        let id = TypeId(self.types.len());
        self.types.push(ty);
        id
    }

    pub fn get(&self, id: TypeId) -> Option<&Type> {
        self.types.get(id.0)
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

impl Index<TypeId> for TypeArena {
    type Output = Type;

    fn index(&self, index: TypeId) -> &Self::Output {
        &self.types[index.0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(usize);

impl TypeId {
    pub fn new(idx: usize) -> TypeId {
        TypeId(idx)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct TermArena {
    terms: Vec<Term>,
    type_annotations: HashMap<usize, TypeId>,
}

impl TermArena {
    pub fn new() -> TermArena {
        TermArena {
            terms: Vec::new(),
            type_annotations: HashMap::new(),
        }
    }

    pub fn alloc(&mut self, term: Term) -> TermId {
        let id = TermId(self.terms.len());
        self.terms.push(term);
        id
    }

    pub fn alloc_with_type(&mut self, term: Term, ty: TypeId) -> TermId {
        let id = self.alloc(term);
        self.type_annotations.insert(id.0, ty);
        id
    }

    pub fn get(&self, id: TermId) -> Option<&Term> {
        self.terms.get(id.0)
    }

    pub fn get_type(&self, id: TermId) -> Option<TypeId> {
        self.type_annotations.get(&id.0).copied()
    }

    pub fn len(&self) -> usize {
        self.terms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

impl Index<TermId> for TermArena {
    type Output = Term;

    fn index(&self, index: TermId) -> &Self::Output {
        &self.terms[index.0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TermId(usize);

impl TermId {
    pub fn new(idx: usize) -> TermId {
        TermId(idx)
    }

    pub fn index(&self) -> usize {
        self.0
    }
}
