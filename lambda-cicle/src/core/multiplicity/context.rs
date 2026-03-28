use crate::core::ast::types::{Multiplicity, Type};
use crate::core::multiplicity::semiring::Quantity;
use hashbrown::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorrowContextMix {
    CannotMixBorrowWithQuantity,
    CannotScaleBorrow,
}

impl std::fmt::Display for BorrowContextMix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BorrowContextMix::CannotMixBorrowWithQuantity => {
                write!(f, "Cannot mix borrow context with quantity context")
            }
            BorrowContextMix::CannotScaleBorrow => {
                write!(f, "Cannot scale borrow")
            }
        }
    }
}

impl std::error::Error for BorrowContextMix {}

#[derive(Debug, Clone, Default)]
pub struct Context {
    bindings: HashMap<String, (Multiplicity, Type)>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            bindings: HashMap::new(),
        }
    }

    pub fn extend(&self, var: String, mult: Multiplicity, ty: Type) -> Context {
        let mut bindings = self.bindings.clone();
        bindings.insert(var, (mult, ty));
        Context { bindings }
    }

    pub fn get(&self, var: &str) -> Option<&(Multiplicity, Type)> {
        self.bindings.get(var)
    }

    pub fn contains(&self, var: &str) -> bool {
        self.bindings.contains_key(var)
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.bindings.keys()
    }

    pub fn add(&self, other: &Context) -> Result<Context, BorrowContextMix> {
        let mut result = self.bindings.clone();

        for (var, (mult2, ty2)) in &other.bindings {
            match result.get(var) {
                Some((mult1, ty1)) => {
                    let new_mult = mult_add(mult1, mult2)?;
                    result.insert(var.clone(), (new_mult, ty1.clone()));
                }
                None => {
                    result.insert(var.clone(), (mult2.clone(), ty2.clone()));
                }
            }
        }

        Ok(Context { bindings: result })
    }

    pub fn scale(&self, q: Quantity) -> Result<Context, BorrowContextMix> {
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

        Ok(Context { bindings: result })
    }

    pub fn to_quantity_context(&self) -> QuantityContext {
        let mut bindings = HashMap::new();

        for (var, (mult, ty)) in &self.bindings {
            let q = match mult {
                Multiplicity::Zero => Quantity::Zero,
                Multiplicity::One => Quantity::One,
                Multiplicity::Omega => Quantity::Omega,
                Multiplicity::Borrow => continue,
            };
            bindings.insert(var.clone(), (q, ty.clone()));
        }

        QuantityContext { bindings }
    }
}

fn mult_add(m1: &Multiplicity, m2: &Multiplicity) -> Result<Multiplicity, BorrowContextMix> {
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

pub fn ctx_add(c1: Context, c2: Context) -> Result<Context, BorrowContextMix> {
    c1.add(&c2)
}

pub fn ctx_scale(q: Quantity, ctx: Context) -> Result<Context, BorrowContextMix> {
    ctx.scale(q)
}

#[derive(Debug, Clone, Default)]
pub struct QuantityContext {
    bindings: HashMap<String, (Quantity, Type)>,
}

impl QuantityContext {
    pub fn new() -> QuantityContext {
        QuantityContext {
            bindings: HashMap::new(),
        }
    }

    pub fn extend(&self, var: String, q: Quantity, ty: Type) -> QuantityContext {
        let mut bindings = self.bindings.clone();
        bindings.insert(var, (q, ty));
        QuantityContext { bindings }
    }

    pub fn get(&self, var: &str) -> Option<&(Quantity, Type)> {
        self.bindings.get(var)
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    pub fn add(&self, other: &QuantityContext) -> Result<QuantityContext, BorrowContextMix> {
        let mut result = self.bindings.clone();

        for (var, (q2, ty2)) in &other.bindings {
            match result.get(var) {
                Some((q1, ty1)) => {
                    let new_q = crate::core::multiplicity::semiring::quantity_add(*q1, *q2);
                    result.insert(var.clone(), (new_q, ty1.clone()));
                }
                None => {
                    result.insert(var.clone(), (*q2, ty2.clone()));
                }
            }
        }

        Ok(QuantityContext { bindings: result })
    }

    pub fn scale(&self, q: Quantity) -> QuantityContext {
        let mut result = HashMap::new();

        for (var, (q1, ty)) in &self.bindings {
            let new_q = crate::core::multiplicity::semiring::quantity_mul(q, *q1);
            result.insert(var.clone(), (new_q, ty.clone()));
        }

        QuantityContext { bindings: result }
    }
}
