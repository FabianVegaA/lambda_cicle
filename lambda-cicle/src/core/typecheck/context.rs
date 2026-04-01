use crate::core::ast::types::{Multiplicity, Type};
use crate::core::multiplicity::semiring::Quantity;
use crate::core::multiplicity::BorrowContextMix;
use hashbrown::HashMap;

#[derive(Debug, Clone)]
pub struct ConstructorInfo {
    pub type_name: String,
    pub field_types: Vec<Type>,
    pub result_type: Type,
}

#[derive(Debug, Clone, Default)]
pub struct TypeContext {
    bindings: HashMap<String, (Multiplicity, Type)>,
    constructors: HashMap<String, ConstructorInfo>,
}

impl TypeContext {
    pub fn new() -> TypeContext {
        TypeContext {
            bindings: HashMap::new(),
            constructors: HashMap::new(),
        }
    }

    pub fn extend(&self, var: String, mult: Multiplicity, ty: Type) -> TypeContext {
        let mut bindings = self.bindings.clone();
        bindings.insert(var, (mult, ty));
        TypeContext {
            bindings,
            constructors: self.constructors.clone(),
        }
    }

    pub fn get(&self, var: &str) -> Option<&(Multiplicity, Type)> {
        self.bindings.get(var)
    }

    pub fn contains(&self, var: &str) -> bool {
        self.bindings.contains_key(var)
    }

    pub fn get_constructor(&self, name: &str) -> Option<&ConstructorInfo> {
        self.constructors.get(name)
    }

    pub fn register_constructor(&self, name: String, info: ConstructorInfo) -> TypeContext {
        let mut constructors = self.constructors.clone();
        constructors.insert(name, info);
        TypeContext {
            bindings: self.bindings.clone(),
            constructors,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn add(&self, other: &TypeContext) -> Result<TypeContext, BorrowContextMix> {
        let mut result = self.bindings.clone();

        for (var, (mult2, ty2)) in &other.bindings {
            match result.get(var) {
                Some((mult1, _)) => {
                    let new_mult = add_multiplicity(mult1, mult2)?;
                    result.insert(var.clone(), (new_mult, ty2.clone()));
                }
                None => {
                    result.insert(var.clone(), (mult2.clone(), ty2.clone()));
                }
            }
        }

        Ok(TypeContext {
            bindings: result,
            constructors: self.constructors.clone(),
        })
    }

    pub fn scale(&self, q: Quantity) -> Result<TypeContext, BorrowContextMix> {
        let mut result = HashMap::new();

        for (var, (mult, ty)) in &self.bindings {
            let new_mult = match mult {
                Multiplicity::Borrow => return Err(BorrowContextMix::CannotScaleBorrow),
                Multiplicity::Zero => Multiplicity::Zero,
                Multiplicity::One => match q {
                    Quantity::Zero => Multiplicity::Zero,
                    Quantity::One => Multiplicity::One,
                    Quantity::Omega => Multiplicity::Omega,
                },
                Multiplicity::Omega => Multiplicity::Omega,
            };
            result.insert(var.clone(), (new_mult, ty.clone()));
        }

        Ok(TypeContext {
            bindings: result,
            constructors: self.constructors.clone(),
        })
    }

    pub fn weaken(&self, var: String, ty: Type) -> TypeContext {
        self.extend(var, Multiplicity::Zero, ty)
    }
}

fn add_multiplicity(
    m1: &Multiplicity,
    m2: &Multiplicity,
) -> Result<Multiplicity, BorrowContextMix> {
    match (m1, m2) {
        (Multiplicity::Borrow, _) | (_, Multiplicity::Borrow) => {
            Err(BorrowContextMix::CannotMixBorrowWithQuantity)
        }
        (Multiplicity::Zero, m) | (m, Multiplicity::Zero) => Ok(m.clone()),
        (Multiplicity::One, Multiplicity::One) => Ok(Multiplicity::Omega),
        (Multiplicity::One, Multiplicity::Omega) | (Multiplicity::Omega, Multiplicity::One) => {
            Ok(Multiplicity::Omega)
        }
        (Multiplicity::Omega, Multiplicity::Omega) => Ok(Multiplicity::Omega),
    }
}
