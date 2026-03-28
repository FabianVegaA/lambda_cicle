import LambdaCalculus.Core.Terms

/-!
# Quantity Semiring {0, 1, ω}

Formalization of the quantity semiring for λ◦₀ linear type system.

## Semiring Structure

The quantity axis forms a commutative semiring `(Q, +, ·, 0, 1)`:
- Addition: how uses combine (join)
- Multiplication: how context scales under abstraction

## References

- λ◦ Design Document v2.2 §3.2
- Atkey (2018): Quantitative Type Theory
- Girard (1987): Linear Logic
-/

namespace LambdaCalculus

/-!
## Quantity Type

The quantity axis: how many times a value can be used.
- `zero` (0): erased, compile-time only
- `one` (1): linear, exactly once  
- `omega` (ω): shared, many times
- `borrow` (&): observation mode, not a quantity (handled separately)
-/

inductive Quantity
  | zero : Quantity  -- 0: erased
  | one  : Quantity  -- 1: linear
  | omega : Quantity -- ω: shared
  deriving DecidableEq, Repr

namespace Quantity
  open Quantity

  /-- Addition: how uses combine (join) -/
  def add : Quantity → Quantity → Quantity
    | zero, q => q
    | one, zero => one
    | one, one => omega
    | one, omega => omega
    | omega, _ => omega

  /-- Scalar multiplication: how context scales -/
  def mul : Quantity → Quantity → Quantity
    | zero, _ => zero
    | one, q => q
    | omega, zero => zero
    | omega, one => omega
    | omega, omega => omega

  /-- Additive identity -/
  lemma zero_add (q : Quantity) : add zero q = q := by cases q <;> rfl

  /-- Multiplicative identity -/
  lemma one_mul (q : Quantity) : mul one q = q := by cases q <;> rfl

  /-- Additive commutativity -/
  lemma add_comm (q₁ q₂ : Quantity) : add q₁ q₂ = add q₂ q₁ := by
    cases q₁ <;> cases q₂ <;> rfl

  /-- Additive associativity -/
  lemma add_assoc (q₁ q₂ q₃ : Quantity) : add (add q₁ q₂) q₃ = add q₁ (add q₂ q₃) := by
    cases q₁ <;> cases q₂ <;> cases q₃ <;> rfl

  /-- Multiplicative associativity -/
  lemma mul_assoc (q₁ q₂ q₃ : Quantity) : mul (mul q₁ q₂) q₃ = mul q₁ (mul q₂ q₃) := by
    cases q₁ <;> cases q₂ <;> cases q₃ <;> rfl

  /-- Left distributivity: q₁ · (q₂ + q₃) = (q₁ · q₂) + (q₁ · q₃) -/
  lemma left_distrib (q₁ q₂ q₃ : Quantity) : mul q₁ (add q₂ q₃) = add (mul q₁ q₂) (mul q₁ q₃) := by
    cases q₁ <;> cases q₂ <;> cases q₃ <;> rfl

  /-- Right distributivity: (q₁ + q₂) · q₃ = (q₁ · q₃) + (q₂ · q₃) -/
  lemma right_distrib (q₁ q₂ q₃ : Quantity) : mul (add q₁ q₂) q₃ = add (mul q₁ q₃) (mul q₂ q₃) := by
    cases q₁ <;> cases q₂ <;> cases q₃ <;> rfl

  /-- Additive absorption: ω + q = ω -/
  lemma omega_absorb (q : Quantity) : add omega q = omega := by cases q <;> rfl

  /-- Multiplicative zero: 0 · q = 0 -/
  lemma zero_mul (q : Quantity) : mul zero q = zero := by cases q <;> rfl

  /-- Partial order: 0 ⊑ 1 ⊑ ω -/
  def le : Quantity → Quantity → Bool
    | zero, _ => true
    | one, omega => true
    | one, one => true
    | omega, omega => true
    | _, _ => false

  instance : Add Quantity where add := add
  instance : Mul Quantity where mul := mul
  instance : Zero Quantity where zero := zero
  instance : One Quantity where one := one

end Quantity

/-!
## Mode Type (Borrow)

The mode axis: observation vs ownership.
- `borrow` (&): observe but do not consume
-/

inductive Mode
  | borrow : Mode  -- &: borrowed reference (observer)
  deriving DecidableEq, Repr

namespace Mode
  open Mode

  /-- Borrow mode cannot be scaled -/
  /-- Note: The original statement was malformed (borrow is Mode, not Quantity).
      This lemma is commented out pending clarification of the intended property. -/
  -- lemma borrow_not_scalable : ∀ (q : Quantity), Quantity.mul q borrow = none := sorry

end Mode

/-!
## Multiplicity

Combined view: either a quantity or a borrow mode.
-/

inductive Multiplicity
  | qty (q : Quantity) : Multiplicity
  | mode (m : Mode) : Multiplicity
  deriving DecidableEq, Repr

namespace Multiplicity
  open Multiplicity

  /-- Check if this is a borrow mode -/
  def isBorrow : Multiplicity → Bool
    | mode (Mode.borrow) => true
    | _ => false

  /-- Check if this is a quantity (not borrow) -/
  def isQuantity : Multiplicity → Bool
    | qty _ => true
    | _ => false

end Multiplicity

end LambdaCalculus
