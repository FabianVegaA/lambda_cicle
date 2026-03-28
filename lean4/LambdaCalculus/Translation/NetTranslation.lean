import LambdaCalculus.Core.Terms
import LambdaCalculus.TypeSystem.TypingRules

/-!
# Interaction Net Translation

Translation from λ◉₀ terms to interaction nets.
This formalizes the correspondence between the type system
and the operational semantics via interaction nets.

## Grammar

```
Net :: = λ(x).N           -- abstraction (Lambda agent)
       | N₁ N₂            -- application (Beta/Delta agents)
       | let x = N₁ in N2 -- let binding
       | x                -- reference (Wire)

Agent :: = λ               -- Lambda: (λx) y → y → x
        | β               -- Beta: (λx) y → y[x]
        | δ               -- Delta: copies ω-annotated terms
        | ε               -- Erasure: drops 0-annotated terms
```

## References

- λ◉ Design Document v2.2 §5
- Lafont (1997): Interaction Nets
- Phase 0 deliverable 0.4
-/

namespace LambdaCalculus

/-!
## Interaction Net Syntax

Basic agents for the interaction net semantics.
-/

inductive NetAgent
  | lambda : NetAgent    -- λ-agent (abstraction)
  | beta : NetAgent      -- β-agent (application)
  | delta : NetAgent     -- δ-agent (contraction/copying)
  | epsilon : NetAgent   -- ε-agent (weakening/erasure)
  deriving DecidableEq, Repr

namespace NetAgent
  /-- Lambda agent representation -/
  def lambda : NetAgent := NetAgent.lambda

  /-- Beta agent representation -/
  def beta : NetAgent := NetAgent.beta

  /-- Delta agent representation -/
  def delta : NetAgent := NetAgent.delta

  /-- Epsilon agent representation -/
  def epsilon : NetAgent := NetAgent.epsilon
end NetAgent

/-!
## Net Terms

Interaction net representation of λ◉₀ terms.
-/

inductive Net
  | agent (a : NetAgent) (ports : List Net)  -- agent with ports
  | wire (x : String)                         -- wire/connection
  deriving DecidableEq, Repr

namespace Net
  /-- A lambda net has one body port and one argument port -/
  def lambdaNet (body : Net) : Net :=
    agent NetAgent.lambda [body]

  /-- A beta net connects lambda and argument -/
  def betaNet (lam arg : Net) : Net :=
    agent NetAgent.beta [lam, arg]

  /-- A delta net duplicates a wire -/
  def deltaNet (inp out₁ out₂ : Net) : Net :=
    agent NetAgent.delta [inp, out₁, out₂]

  /-- An epsilon net erases a wire -/
  def epsilonNet (inp : Net) : Net :=
    agent NetAgent.epsilon [inp]
end Net

/-!
## Translation from Terms to Nets

The main translation function.
-/

def Term.toNet (e : Term) : Net :=
  match e with
  | Term.var x => Net.wire x
  | Term.abs x q τ e' =>
    Net.lambdaNet (toNet e')
  | Term.app e₁ e₂ =>
    Net.betaNet (toNet e₁) (toNet e₂)
  | Term.letTerm x q τ e₁ e₂ =>
    Net.agent NetAgent.epsilon [toNet e₁]
  termination_by e => e.numNodes

namespace Term
  /-- Extended termination measure -/
  def numNodes : Term → Nat
    | var _ => 1
    | abs _ _ _ e => 1 + numNodes e
    | app e₁ e₂ => 1 + numNodes e₁ + numNodes e₂
    | letTerm _ _ _ e₁ e₂ => 1 + numNodes e₁ + numNodes e₂
end Term

/-!
## Typing of Nets

Correspondence between term typing and net structure.
-/

namespace NetTyping

  /-- Net typing judgment -/
  inductive HasNetType : List (String × Type) → Net → Type → Prop
    | wire {Γ : List (String × Type)} {x : String} {τ : Type} :
        (x, τ) ∈ Γ → HasNetType Γ (Net.wire x) τ
    | lambda {Γ : List (String × Type)} {body : Net} {τ₁ τ₂ : Type} :
        HasNetType ((⟨"_", Quantity.one⟩, τ₁) :: Γ) body τ₂ →
        HasNetType Γ (Net.lambdaNet body) (Type.arrow Quantity.one τ₁ τ₂)
    | beta {Γ : List (String × Type)} {e₁ e₂ : Net} {τ₁ τ₂ : Type} :
        HasNetType Γ e₁ (Type.arrow Quantity.one τ₁ τ₂) →
        HasNetType Γ e₂ τ₁ →
        HasNetType Γ (Net.betaNet e₁ e₂) τ₂

end NetTyping

/-!
## Translation Correctness

The translation preserves typing.
-/

namespace TranslationProperties

  /-- Translation preserves typing -/
  theorem translation_preserves_type
    {Γ : Context} {e : Term} {τ : Type}
    (h : HasType Γ e τ) :
    NetTyping.HasNetType (Γ.map (fun b => (b.name, b.type))) e.toNet τ :=
    by
      sorry

  /-- Translation reflects values -/
  theorem translation_reflects_values
    {e : Term}
    (hv : e.isValue = true) :
    match e.toNet with
    | Net.lambdaNet _ => True
    | _ => False :=
    by
      cases e <;> simp only [Term.isValue, Net.toNet] at hv
      case abs => trivial
      cases hv

  /-- Beta reduction corresponds to net reduction -/
  theorem beta_correspondence
    {x : String} {q : Quantity} {τ : Type} {e v : Term} :
    let e' := Term.app (Term.abs x q τ e) v
    let e'' := Term.subst e v x
    e'.toNet ≈ e''.toNet :=
    by
      sorry

end TranslationProperties

end LambdaCalculus
