import LambdaCalculus.Core.Semiring

/-!
# Term Syntax for λ◦₀

Formalization of the simply-typed lambda calculus with linear multiplicities.
This is the formal kernel λ◦₀ (multiplicities {0, 1, ω}, no traits, no native types, no match/view).

## Grammar

```
e ::= x                          -- variable
    | λ(x :_q τ). e              -- abstraction, x used q times in e
    | e₁ e₂                      -- application
    | let x :_q τ = e₁ in e₂    -- let binding
```

## References

- λ◦ Design Document v2.2 §2.3
- Phase 0 deliverable 0.3
-/

namespace LambdaCalculus

/-!
## Types

Simple types without native types, traits, or inductive types in λ◦₀.
-/

inductive Type
  | var : String → Type                    -- type variable α
  | arrow (q : Quantity) (τ₁ τ₂ : Type)    -- τ₁ →^q τ₂
  | forall (α : String) (τ : Type)          -- ∀α. τ (parametric polymorphism)
  deriving DecidableEq, Repr

namespace Type
  /-- Arrow type constructor -/
  def arrow (q : Quantity) (τ₁ τ₂ : Type) : Type :=
    Type.arrow q τ₁ τ₂

  /-- Function type notation -/
  notation:50 τ₁ " →^" q " " τ₂ => arrow q τ₁ τ₂

  /-- Linear function type -/
  def linArrow (τ₁ τ₂ : Type) : Type :=
    arrow Quantity.one τ₁ τ₂

  notation:50 τ₁ " → " τ₂ => linArrow τ₁ τ₂

  /-- Unrestricted function type -/
  def omegaArrow (τ₁ τ₂ : Type) : Type :=
    arrow Quantity.omega τ₁ τ₂

  notation:50 τ₁ " →ω " τ₂ => omegaArrow τ₁ τ₂

end Type

/-!
## Terms

Lambda calculus terms with multiplicity annotations.
-/

inductive Term
  | var (x : String)                                    -- x
  | abs (x : String) (q : Quantity) (τ : Type) (e : Term)  -- λ(x :_q τ). e
  | app (e₁ e₂ : Term)                                 -- e₁ e₂
  | letTerm (x : String) (q : Quantity) (τ : Type) (e₁ e₂ : Term)  -- let x :_q τ = e₁ in e₂
  deriving DecidableEq, Repr

namespace Term
  /-- Variable -/
  def var (x : String) : Term := Term.var x

  /-- Lambda abstraction -/
  def abs (x : String) (q : Quantity) (τ : Type) (e : Term) : Term :=
    Term.abs x q τ e

  /-- Application -/
  def app (e₁ e₂ : Term) : Term := Term.app e₁ e₂

  /-- Let binding -/
  def letTerm (x : String) (q : Quantity) (τ : Type) (e₁ e₂ : Term) : Term :=
    Term.letTerm x q τ e₁ e₂

  /-- Check if term is a value (lambda abstraction) -/
  def isValue : Term → Bool
    | abs _ _ _ _ => true
    | _ => false

  /-- Free variables in a term -/
  def freeVars : Term → List String
    | var x => [x]
    | abs x _ _ e => freeVars e |>.filter (· ≠ x)
    | app e₁ e₂ => freeVars e₁ ++ freeVars e₂
    | letTerm x _ _ e₁ e₂ =>
      freeVars e₁ ++ (freeVars e₂ |>.filter (· ≠ x))

  /-- Variable occurs free in term -/
  def occursFree (x : String) (e : Term) : Bool :=
    (freeVars e).contains x

  /-- Capture-avoiding substitution: e[v/x] -/
  def subst (e : Term) (v : Term) (x : String) : Term :=
    let e' := e.replace (fun y => if y = x then some v else none)
    e'

end Term

/-!
## Values

Closed terms that are values (ready to reduce).
-/

inductive Value
  | closure (x : String) (q : Quantity) (τ : Type) (e : Term) (σ : Substitution)
  deriving DecidableEq, Repr

namespace Value
  /-- Every value is a term -/
  def toTerm : Value → Term
    | closure x q τ e _ => Term.abs x q τ e
end Value

/-!
## Substitution Environment

Mapping from variables to values for evaluation.
-/

abbrev Substitution := List (String × Value)

namespace Substitution
  def empty : Substitution := []

  def extend (σ : Substitution) (x : String) (v : Value) : Substitution :=
    (x, v) :: σ

  def lookup (σ : Substitution) (x : String) : Option Value :=
    σ.find? (fun (y, _) => y = x) |>.map (·.2)
end Substitution

/-!
## Reduction

Beta reduction semantics.
-/

inductive ReducesTo
  | step (e₁ e₂ : Term)     -- e₁ → e₂
  | trans (e₁ e₂ e₃ : Term) -- e₁ → e₂ → e₃
  deriving DecidableEq, Repr

namespace ReducesTo
  /-- Beta reduction: (λ(x :_q τ). e) v → e[v/x] -/
  def beta (x : String) (q : Quantity) (τ : Type) (e v : Term) : Term :=
    Term.subst e v x

  /-- Single step reduction -/
  inductive Step : Term → Term → Prop
    | beta (x : String) (q : Quantity) (τ : Type) (e v : Term) :
        Step (Term.app (Term.abs x q τ e) v) (beta x q τ e v)
    | appLeft {e₁ e₁' e₂ : Term} :
        Step e₁ e₁' → Step (Term.app e₁ e₂) (Term.app e₁' e₂)
    | appRight {e₁ e₂ e₂' : Term} (hv : e₁.IsValue) :
        Step e₂ e₂' → Step (Term.app e₁ e₂) (Term.app e₁ e₂')
    | letVal {x : String} {q : Quantity} {τ : Type} {e₁ e₂ v : Term} :
        Step e₁ e₂ → Step (Term.letTerm x q τ e₁ e₂) (Term.letTerm x q τ e₂ e₂)

  infix:50 " →β " => Step

  /-- Reflexive-transitive closure -/
  inductive Reduces : Term → Term → Prop
    | refl (e : Term) : Reduces e e
    | step {e₁ e₂ e₃ : Term} : Step e₁ e₂ → Reduces e₂ e₃ → Reduces e₁ e₃

  infix:50 " →* " => Reduces

end ReducesTo

end LambdaCalculus
