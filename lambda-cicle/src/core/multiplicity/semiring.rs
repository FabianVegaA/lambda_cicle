use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quantity {
    Zero,
    One,
    Omega,
}

impl Quantity {
    pub fn zero() -> Quantity {
        Quantity::Zero
    }

    pub fn one() -> Quantity {
        Quantity::One
    }

    pub fn omega() -> Quantity {
        Quantity::Omega
    }
}

pub fn quantity_add(q1: Quantity, q2: Quantity) -> Quantity {
    match (q1, q2) {
        (Quantity::Zero, q) => q,
        (q, Quantity::Zero) => q,
        _ => Quantity::Omega,
    }
}

pub fn quantity_mul(q1: Quantity, q2: Quantity) -> Quantity {
    match (q1, q2) {
        (Quantity::Zero, _) | (_, Quantity::Zero) => Quantity::Zero,
        (Quantity::One, q) | (q, Quantity::One) => q,
        (Quantity::Omega, Quantity::Omega) => Quantity::Omega,
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quantity::Zero => write!(f, "0"),
            Quantity::One => write!(f, "1"),
            Quantity::Omega => write!(f, "ω"),
        }
    }
}
