import LambdaCalculus.Core.Semiring

/-!
# Context Operations

Formalization of typing context operations for λ◦₀:
- Context addition (splitting)
- Scalar multiplication (scaling)
- 0·Γ semantics (produces _0-annotated context, NOT empty)

## References

- λ◦ Design Document v2.2 §4.2
- Phase 0 deliverable 0.2
-/

namespace LambdaCalculus

/-!
## Binding

A single binding in the typing context:
- name → (multiplicity, type)
-/

structure Binding where
  name : String
  mult : Quantity
  type : Type
  deriving DecidableEq, Repr

/-!
## Typing Context

A finite map from names to (multiplicity, type) pairs.
Contexts are split, not shared: using a variable consumes its entry.
-/

abbrev Context := List Binding

namespace Context

  /-- Empty context -/
  def empty : Context := []

  /-- Add a binding to the context -/
  def add (Γ : Context) (b : Binding) : Context :=
    b :: Γ

  /-- Look up a binding by name -/
  def lookup (Γ : Context) (x : String) : Option (Quantity × Type) :=
    match Γ with
    | [] => none
    | b :: Γ' => if b.name = x then some (b.mult, b.type) else lookup Γ' x

  /-- Check if a name is in the domain -/
  def contains (Γ : Context) (x : String) : Bool :=
    match lookup Γ x with
    | some _ => true
    | none => false

  /-- Pointwise addition of contexts (context splitting) -/
  /--
  (Γ₁ + Γ₂)(x) =
    | Γ₁(x) + Γ₂(x)   if x ∈ dom(Γ₁) ∩ dom(Γ₂)
    | Γ₁(x)            if x ∈ dom(Γ₁) \ dom(Γ₂)
    | Γ₂(x)            if x ∈ dom(Γ₂) \ dom(Γ₁)
  -/
  def addCtx (Γ₁ Γ₂ : Context) : Context :=
    let dom₁ := Γ₁.map (·.name)
    let dom₂ := Γ₂.map (·.name)
    let common := dom₁.filter (fun x => dom₂.contains x)
    let only₁ := dom₁.filter (fun x => !dom₂.contains x)
    let only₂ := dom₂.filter (fun x => !dom₁.contains x)
    let addBinding (b₁ b₂ : Binding) : Binding :=
      { b₁ with mult := Quantity.add b₁.mult b₂.mult }
    let merged := common.map (fun x =>
      match (lookup Γ₁ x, lookup Γ₂ x) with
      | (some (q₁, τ₁), some (q₂, τ₂)) =>
        some { name := x, mult := Quantity.add q₁ q₂, type := τ₁ }
      | _ => none
    ) |>.keepSome
    let from₁ := only₁.map (fun x => Γ₁.find? (·.name = x) |>.get!)
    let from₂ := only₂.map (fun x => Γ₂.find? (·.name = x) |>.get!)
    merged ++ from₁ ++ from₂

  /-- Scalar multiplication: scales all entries in context -/
  /--
  (q · Γ)(x) = q · Γ(x)    for all x ∈ dom(Γ)

  Critical: 0 · Γ produces a context where EVERY binding is annotated _0,
  NOT the empty context. _0 bindings still trigger Weaken and insert
  ε-agents in translation.
  -/
  def scale (q : Quantity) (Γ : Context) : Context :=
    Γ.map (fun b => { b with mult := Quantity.mul q b.mult })

  /-- Context scaling distributes over context addition -/
  /-- q · (Γ₁ + Γ₂) = (q · Γ₁) + (q · Γ₂) -/
  lemma scale_distributes (q : Quantity) (Γ₁ Γ₂ : Context) :
    scale q (addCtx Γ₁ Γ₂) = addCtx (scale q Γ₁) (scale q Γ₂) :=
    by
      -- Proof uses the distributivity law: q * (a + b) = q*a + q*b
      -- Unfold definitions and apply pointwise
      unfold scale addCtx
      -- Each binding's multiplicity is scaled after/before addition identically
      -- by Quantity.left_distrib
      simp only [List.map_map, Function.comp_apply, List.map_append]
      -- Need to show equality of the resulting lists
      -- This follows from pointwise equality using left_distrib
      ext ⟨name, mult, type⟩
      simp
      -- For each name in the combined context:
      -- LHS: scale after add → q * (mult₁ + mult₂)
      -- RHS: add after scale → q*mult₁ + q*mult₂
      -- These are equal by left_distrib
      exact Quantity.left_distrib q _ _

  /-- Weakening: add a _0 binding to the context -/
  def weaken (Γ : Context) (x : String) (τ : Type) : Context :=
    { name := x, mult := Quantity.zero, type := τ } :: Γ

  /-- Check if context is empty -/
  def isEmpty (Γ : Context) : Bool :=
    Γ.isEmpty

  /-- Domain of context -/
  def dom (Γ : Context) : List String :=
    Γ.map (·.name)

end Context

/-!
## Context Operations as Infix -/

infixl:65 " +ctx " => Context.addCtx
infixl:70 " ·ctx " => Context.scale

end LambdaCalculus
