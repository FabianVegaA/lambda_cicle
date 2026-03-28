import LambdaCalculus.TypeSystem.TypingRules
import LambdaCalculus.Core.Terms

/-!
# Preservation Lemma

If Γ ⊢ e : τ and e → e', then Γ ⊢ e' : τ.

## References

- λ◉ Design Document v2.2 §4.4
- Phase 0 deliverable 0.3
-/

namespace LambdaCalculus

/-!
## Preservation Property

Type preservation under reduction: the type of a term is preserved
under one step of β-reduction.
-/

namespace TypingProperties

  /-- Preservation: if Γ ⊢ e : τ and e → e', then Γ ⊢ e' : τ -/
  theorem preservation
    {Γ : Context} {e e' : Term} {τ : Type}
    (h : HasType Γ e τ)
    (hStep : ReducesTo.Step e e') :
    HasType Γ e' τ :=
    by
      induction h generalizing e'
      case var =>
        cases hStep
      case varOmega =>
        cases hStep
      case abs Γ x q τ₁ e ih =>
        cases hStep
      case app Γ₁ Γ₂ e₁ e₂ q τ₁ τ₂ h1 ih1 h2 ih2 =>
        cases hStep
        case beta x q τ e v =>
          simp only [ReducesTo.beta] at hStep
          rw [←hStep]
          have hx : x = x := rfl
          have hQ : q = q := rfl
          have hT : τ = τ₁ := rfl
          rename_i h
          have : HasType (Context.add Γ₁ { name := x, mult := q, type := τ₁ }) e τ₂ := by
            cases h1 with
            | abs _ _ _ _ _ h => exact h
          exact substitution this h2
        case appLeft h =>
          apply HasType.app
          · exact ih1 h
          · exact h2
        case appRight h hv =>
          apply HasType.app
          · exact h1
          · exact ih2 h
      case letTerm Γ₁ Γ₂ x q τ₁ τ₂ e₁ e₂ h1 ih1 h2 ih2 =>
        cases hStep
        case letVal h =>
          apply HasType.letTerm
          · exact ih1 h
          · exact h2
      case weaken Γ x τ τ' e h ih =>
        cases hStep
        apply HasType.weaken
        exact ih hStep

end TypingProperties

end LambdaCalculus
