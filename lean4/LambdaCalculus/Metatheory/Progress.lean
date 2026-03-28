import LambdaCalculus.TypeSystem.TypingRules
import LambdaCalculus.Core.Terms

/-!
# Progress Lemma

If ∅ ⊢ e : τ, then either e is a value or e can take a step.

## References

- λ◉ Design Document v2.2 §4.4
- Phase 0 deliverable 0.3
-/

namespace LambdaCalculus

/-!
## Progress Property

Well-typed terms don't get stuck: they are either values or can reduce.
-/

namespace TypingProperties

  /-- Progress: if ∅ ⊢ e : τ, then e is a value or there exists e' such that e → e' -/
  theorem progress
    {e : Term} {τ : Type}
    (h : HasType Context.empty e τ) :
    e.IsValue ∨ ∃ e', ReducesTo.Step e e' :=
    by
      induction h with
      | var =>
        left
        rfl
      | varOmega =>
        left
        rfl
      | abs Γ x q τ₁ e ih =>
        left
        rfl
      | app Γ₁ Γ₂ e₁ e₂ q τ₁ τ₂ h1 ih1 h2 ih2 =>
        right
        cases ih1 with
        | inl hv1 =>
          cases ih2 with
          | inl hv2 =>
            exists ReducesTo.beta x q τ₁ e₁ e₂
          | inr ⟨e₂', hstep₂⟩ =>
            exists ReducesTo.Step.appRight hv2 hstep₂
        | inr ⟨e₁', hstep₁⟩ =>
            exists ReducesTo.Step.appLeft hstep₁
      | letTerm Γ₁ Γ₂ x q τ₁ τ₂ e₁ e₂ h1 ih1 h2 ih2 =>
        right
        cases ih1 with
        | inl hv1 =>
          cases ih2 with
          | inl hv2 =>
            exists ReducesTo.beta x q τ₁ e₂ e₁
          | inr ⟨e₂', hstep₂⟩ =>
            exists ReducesTo.Step.letVal hstep₂
        | inr ⟨e₁', hstep₁⟩ =>
          exists ReducesTo.Step.letVal hstep₁
      | weaken =>
        cases h
        cases h_1

end TypingProperties

end LambdaCalculus
