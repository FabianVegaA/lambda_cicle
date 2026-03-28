import LambdaCalculus.Core.Terms
import LambdaCalculus.Core.Context

/-!
# Typing Rules for λ◦₀

Formalization of the typing rules for the formal kernel.

## Typing Judgment

`Γ ⊢ e : τ` — in context Γ, term e has type τ.

## Typing Rules

| Rule    | Description |
|---------|-------------|
| Var     | x :_1 τ ⊢ x : τ (linear variable, consumed on use) |
| Var-Ω   | x :_ω τ ⊢ x : τ (shared variable, may be used multiply) |
| Abs     | Γ, x :_q τ₁ ⊢ e : τ₂ → Γ ⊢ λ(x :_q τ₁). e : τ₁ →^q τ₂ |
| App     | Γ₁ ⊢ e₁ : τ₁ →^q τ₂, Γ₂ ⊢ e₂ : τ₁ → Γ₁ + q·Γ₂ ⊢ e₁ e₂ : τ₂ |
| Let     | q·Γ₁ ⊢ e₁ : τ₁, Γ₂, x :_q τ₁ ⊢ e₂ : τ₂ → Γ₁ + Γ₂ ⊢ let x :_q τ₁ = e₁ in e₂ : τ₂ |
| Weaken  | Γ ⊢ e : τ → Γ, x :_0 τ' ⊢ e : τ |

## References

- λ◦ Design Document v2.2 §4.3
- Phase 0 deliverable 0.3
-/

namespace LambdaCalculus

/-!
## Typing Relation

`Γ ⊢ e : τ` — in context Γ, term e has type τ.
-/

inductive HasType : Context → Term → Type → Prop
  /-- Var (linear variable, consumed on use) -/
  | var (Γ : Context) (x : String) (τ : Type) :
      HasType (Context.weaken Γ x τ) (Term.var x) τ

  /-- Var-Omega (shared variable, may be used multiply) -/
  | varOmega (Γ : Context) (x : String) (τ : Type) :
      HasType (Context.weaken Γ x (Type.arrow Quantity.omega τ τ)) (Term.var x) τ

  /-- Abstraction -/
  | abs (Γ : Context) (x : String) (q : Quantity) (τ₁ τ₂ : Type) (e : Term) :
      HasType (Context.add Γ { name := x, mult := q, type := τ₁ }) e τ₂ →
      HasType Γ (Term.abs x q τ₁ e) (Type.arrow q τ₁ τ₂)

  /-- Application -/
  | app (Γ₁ Γ₂ : Context) (e₁ e₂ : Term) (q : Quantity) (τ₁ τ₂ : Type) :
      HasType Γ₁ e₁ (Type.arrow q τ₁ τ₂) →
      HasType Γ₂ e₂ τ₁ →
      HasType (Context.addCtx Γ₁ (Context.scale q Γ₂)) (Term.app e₁ e₂) τ₂

  /-- Let binding (corrected v2.2) -/
  | letTerm (Γ₁ Γ₂ : Context) (x : String) (q : Quantity) (τ₁ τ₂ : Type) (e₁ e₂ : Term) :
      HasType (Context.scale q Γ₁) e₁ τ₁ →
      HasType (Context.add Γ₂ { name := x, mult := q, type := τ₁ }) e₂ τ₂ →
      HasType (Context.addCtx Γ₁ Γ₂) (Term.letTerm x q τ₁ e₁ e₂) τ₂

  /-- Weakening (drop an unused binding) -/
  | weaken (Γ : Context) (x : String) (τ τ' : Type) (e : Term) :
      HasType Γ e τ →
      HasType (Context.weaken Γ x τ') e τ

  /-- Contraction (not in surface type system; handled by ω annotation + δ-agent) -/
  /-- Note: Contract is NOT a source-level rule. Sharing of ω-annotated bindings
      is handled implicitly by Var-Omega (allows multiple uses) and explicitly in
      interaction net translation by inserting a δ-agent. -/

infix:60 " ⊢ " => HasType

/-!
## Typing Rules as Inductive Predicates

Each rule as a separate constructor for clarity.
-/

namespace TypingRules

  /-- Var rule: x :_1 τ ⊢ x : τ -/
  theorem var_rule {Γ : Context} {x : String} {τ : Type} :
    HasType (Context.weaken Γ x τ) (Term.var x) τ :=
    HasType.var Γ x τ

  /-- Var-Omega rule: x :_ω τ ⊢ x : τ -/
  theorem varOmega_rule {Γ : Context} {x : String} {τ : Type} :
    HasType (Context.weaken Γ x (Type.arrow Quantity.omega τ τ)) (Term.var x) τ :=
    HasType.varOmega Γ x τ

  /-- Abs rule: Γ, x :_q τ₁ ⊢ e : τ₂ → Γ ⊢ λ(x :_q τ₁). e : τ₁ →^q τ₂ -/
  theorem abs_rule {Γ : Context} {x : String} {q : Quantity} {τ₁ τ₂ : Type} {e : Term}
    (h : HasType (Context.add Γ { name := x, mult := q, type := τ₁ }) e τ₂) :
    HasType Γ (Term.abs x q τ₁ e) (Type.arrow q τ₁ τ₂) :=
    HasType.abs Γ x q τ₁ τ₂ e h

  /-- App rule: Γ₁ ⊢ e₁ : τ₁ →^q τ₂, Γ₂ ⊢ e₂ : τ₁ → Γ₁ + q·Γ₂ ⊢ e₁ e₂ : τ₂ -/
  theorem app_rule {Γ₁ Γ₂ : Context} {e₁ e₂ : Term} {q : Quantity} {τ₁ τ₂ : Type}
    (h₁ : HasType Γ₁ e₁ (Type.arrow q τ₁ τ₂))
    (h₂ : HasType Γ₂ e₂ τ₁) :
    HasType (Context.addCtx Γ₁ (Context.scale q Γ₂)) (Term.app e₁ e₂) τ₂ :=
    HasType.app Γ₁ Γ₂ e₁ e₂ q τ₁ τ₂ h₁ h₂

  /-- Let rule (corrected v2.2): q·Γ₁ ⊢ e₁ : τ₁, Γ₂, x :_q τ₁ ⊢ e₂ : τ₂ → Γ₁ + Γ₂ ⊢ let x :_q τ₁ = e₁ in e₂ : τ₂ -/
  theorem let_rule {Γ₁ Γ₂ : Context} {x : String} {q : Quantity} {τ₁ τ₂ : Type} {e₁ e₂ : Term}
    (h₁ : HasType (Context.scale q Γ₁) e₁ τ₁)
    (h₂ : HasType (Context.add Γ₂ { name := x, mult := q, type := τ₁ }) e₂ τ₂) :
    HasType (Context.addCtx Γ₁ Γ₂) (Term.letTerm x q τ₁ e₁ e₂) τ₂ :=
    HasType.letTerm Γ₁ Γ₂ x q τ₁ τ₂ e₁ e₂ h₁ h₂

  /-- Weaken rule: Γ ⊢ e : τ → Γ, x :_0 τ' ⊢ e : τ -/
  theorem weaken_rule {Γ : Context} {x : String} {τ τ' : Type} {e : Term}
    (h : HasType Γ e τ) :
    HasType (Context.weaken Γ x τ') e τ :=
    HasType.weaken Γ x τ τ' e h

end TypingRules

/-!
## Structural Properties

Key lemmas about the typing relation.
-/

namespace TypingProperties

  /-- Lemma 1: Substitution -/
  /-- If Γ₁, x :_q τ₁ ⊢ e : τ₂ and Γ₂ ⊢ v : τ₁,
      then Γ₁ + q·Γ₂ ⊢ e[v/x] : τ₂ -/
  theorem substitution
    {Γ₁ Γ₂ : Context} {e v : Term} {x : String} {q : Quantity} {τ₁ τ₂ : Type}
    (h₁ : HasType (Context.add Γ₁ { name := x, mult := q, type := τ₁ }) e τ₂)
    (h₂ : HasType Γ₂ v τ₁) :
    HasType (Context.addCtx Γ₁ (Context.scale q Γ₂)) (Term.subst e v x) τ₂ :=
    by
      induction h₁ with
      | var Γ y τ =>
        cases h₁_1 : y = x
        case false =>
          simp only [Term.subst, h₁_1, Function.ite_false]
          rw [Context.addCtx_comm _ _ (by simp only [Context.add, List.cons_append, not_false_eq_true])]
          apply HasType.var
        case eq =>
          subst h₁_1
          simp only [Term.subst, if_self]
          exact HasType.varOmega (Context.addCtx Γ₁ (Context.scale q Γ₂)) x τ
      | varOmega Γ y τ =>
        cases h₁_1 : y = x
        case false =>
          simp only [Term.subst, h₁_1, Function.ite_false]
          rw [Context.addCtx_comm _ _ (by simp only [Context.add, List.cons_append, not_false_eq_true])]
          apply HasType.varOmega
        case eq =>
          subst h₁_1
          simp only [Term.subst, if_self]
          exact HasType.varOmega (Context.addCtx Γ₁ (Context.scale q Γ₂)) x τ
      | abs Γ y r τ₁ e ih =>
        by_cases hxy : y = x
        case false =>
          simp only [Term.subst, hxy, Function.ite_false]
          apply HasType.abs
          apply ih
          rfl
        case eq =>
          subst eq
          simp only [Term.subst, if_self]
          apply HasType.abs
          apply ih
          rfl
      | app Γ₁ Γ₂ e₁ e₂ r τ₁ τ₂ h1 ih1 h2 ih2 =>
        simp only [Term.subst]
        apply HasType.app
        · exact ih1 h2
        · exact ih2 h2
      | letTerm Γ₁ Γ₂ y r τ₁ τ₂ e₁ e₂ h1 ih1 h2 ih2 =>
        simp only [Term.subst]
        apply HasType.letTerm
        · apply ih1
        · apply ih2
      | weaken Γ y τ τ' e h ih =>
        simp only [Term.subst]
        apply HasType.weaken
        apply ih

  /-- Weakening is admissible -/
  /-- If Γ ⊢ e : τ then Γ, x :_0 τ' ⊢ e : τ -/
  theorem weakening_admissible
    {Γ : Context} {e : Term} {x : String} {τ τ' : Type}
    (h : HasType Γ e τ) :
    HasType (Context.weaken Γ x τ') e τ :=
    HasType.weaken Γ x τ τ' e h

  /-- Contraction is NOT admissible as a rule -/
  /-- Note: We cannot derive Γ, x :_ω τ ⊢ e : τ' from Γ, x :_ω τ, x :_ω τ ⊢ e : τ'
      at the source level. Instead, ω-annotated bindings are duplicated
      via δ-agents in the interaction net translation. -/

end TypingProperties

end LambdaCalculus
