use lambda_cicle::core::multiplicity::semiring::{quantity_add, quantity_mul, Quantity};

#[test]
fn test_add_identity_zero() {
    assert_eq!(quantity_add(Quantity::Zero, Quantity::One), Quantity::One);
    assert_eq!(quantity_add(Quantity::One, Quantity::Zero), Quantity::One);
    assert_eq!(
        quantity_add(Quantity::Omega, Quantity::Zero),
        Quantity::Omega
    );
}

#[test]
fn test_add_one_plus_one() {
    assert_eq!(quantity_add(Quantity::One, Quantity::One), Quantity::Omega);
}

#[test]
fn test_add_omega() {
    assert_eq!(
        quantity_add(Quantity::Omega, Quantity::One),
        Quantity::Omega
    );
    assert_eq!(
        quantity_add(Quantity::One, Quantity::Omega),
        Quantity::Omega
    );
    assert_eq!(
        quantity_add(Quantity::Omega, Quantity::Omega),
        Quantity::Omega
    );
}

#[test]
fn test_mul_zero() {
    assert_eq!(quantity_mul(Quantity::Zero, Quantity::One), Quantity::Zero);
    assert_eq!(quantity_mul(Quantity::One, Quantity::Zero), Quantity::Zero);
    assert_eq!(
        quantity_mul(Quantity::Zero, Quantity::Omega),
        Quantity::Zero
    );
}

#[test]
fn test_mul_identity_one() {
    assert_eq!(quantity_mul(Quantity::One, Quantity::One), Quantity::One);
    assert_eq!(
        quantity_mul(Quantity::One, Quantity::Omega),
        Quantity::Omega
    );
    assert_eq!(
        quantity_mul(Quantity::Omega, Quantity::One),
        Quantity::Omega
    );
}

#[test]
fn test_mul_omega() {
    assert_eq!(
        quantity_mul(Quantity::Omega, Quantity::Omega),
        Quantity::Omega
    );
}
