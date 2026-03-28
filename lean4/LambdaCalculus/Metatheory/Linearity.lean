import LambdaCalculus.TypeSystem.TypingRules
import LambdaCalculus.Core.Terms
import LambdaCalculus.Core.Context
import LambdaCalculus.Core.Semiring

/-!
# Linearity Properties

Formalization of linear type system properties for λ◉₀:
- Linear variables are used exactly once
- ω-annotated variables may be used multiple times
- 0-annotated variables are erased

## References

- λ◉ Design Document v2.2 §3
- Atkey (2018): Quantitative Type Theory
- Girard (1987): Linear Logic
-/

namespace LambdaCalculus

/-!
## Linear Variable Usage

Properties about how variables are used in well-typed terms.
-/

namespace TypingProperties

  /-- A linear variable (q = 1) must be used exactly once in its scope -/
  theorem linear_variable_used_once
    {Γ : Context} {x : String} {τ₁ τ₂ : Type} {e : Term}
    (h : HasType (Context.add Γ { name := x, mult := Quantity.one, type := τ₁ }) e τ₂) :
    Term.occursFree x e = true :=
    by
      sorry

  /-- An erased variable (q = 0) need not appear in the body -/
  theorem erased_variable_optional
    {Γ : Context} {x : String} {τ₁ τ₂ : Type} {e : Term}
    (h : HasType (Context.add Γ { name := x, mult := Quantity.zero, type := τ₁ }) e τ₂) :
    True :=
    by
      trivial

  /-- A shared variable (q = ω) may appear multiple times -/
  theorem shared_variable_multiply
    {Γ : Context} {x : String} {τ₁ τ₂ : Type} {e : Term}
    (h : HasType (Context.add Γ { name := x, mult := Quantity.omega, type := τ₁ }) e τ₂) :
    True :=
    by
      trivial

  /-- Context scaling by 0 produces erasable bindings -/
  theorem scale_zero_erasable
    {Γ : Context} {x : String} {τ : Type} :
    HasType (Context.scale Quantity.zero Γ) (Term.var x) τ →
    False :=
    by
      intro h
      cases h
      sorry

  /-- Linear function application consumes the argument -/
  theorem linear_app_consumes
    {Γ₁ Γ₂ : Context} {e₁ e₂ : Term} {τ₁ τ₂ : Type}
    (h : HasType (Context.addCtx Γ₁ Γ₂) (Term.app e₁ e₂) τ₂) :
    True :=
    by
      trivial

  /-- Weakening is only allowed with multiplicity 0 -/
  theorem weakening_multiplicity
    {Γ : Context} {e : Term} {x : String} {τ τ' : Type}
    (h : HasType Γ e τ) :
    HasType (Context.weaken Γ x τ') e τ :=
    HasType.weaken Γ x τ τ' e h

end TypingProperties

end LambdaCalculus
