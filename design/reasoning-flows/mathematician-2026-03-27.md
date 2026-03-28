# Mathematician Analysis: λ◦ (Lambda-Circle) Design Document

**Methodology**: CSP + Natural Deduction + Tableaux + Hoare  
**Analysis Date**: 2026-03-27  
**Document Version**: 2.0  

---

## Reasoning Flow Report

### Phase 1: CSP - Invariant Discovery

#### 1.1 Type System as CSP Process Calculus

The type system can be modeled as a CSP process where well-typed terms correspond to processes satisfying resource constraints. The multiplicity system acts as a **resource-aware type discipline**.

**Variables and Domains**:
- `q ∈ Q = {0, 1, ω}` — quantity axis (resource semiring)
- `m ∈ M = {&, borrow}` — mode axis (observation)
- `Γ : Var → (M ∪ Q) × Type` — typing context as finite map
- `τ ∈ Type` — syntactic types (arrow, inductive, trait,forall, borrow)

**Key Invariants Discovered**:

1. **Resource Conservation**: For any valid typing derivation, the total "resource weight" in the context equals the resource weight required to produce the term. This is enforced by the App rule: `Γ₁ + q·Γ₂`.

2. **Borrow Isolation**: The mode `&` is *orthogonal* to quantities. This creates a **two-dimensional type system** where:
   - Quantity axis: `{0, 1, ω}` with semiring structure
   - Mode axis: `{&}` with only identity composition
   
   The invariant: `&` never mixes with quantities in context operations.

3. **Strict Positivity**: Inductive types `μα.τ` require `α` not in negative position. This prevents non-termination from recursive type definitions (like `μα. α → τ`).

#### 1.2 Formal Gaps in CSP Model

**Gap 1.2.1**: The document treats `&` as a "mode annotation" but provides no formal algebra for mode composition beyond `& + & = &`. The interaction between multiple borrows of the same variable is underspecified.

**Gap 1.2.2**: The context scaling operation `q·Γ` is defined pointwise but lacks a formal proof that it respects the semiring laws. Specifically:
- Is `0·Γ = ∅` (empty context)? The table shows `0·Γ = 0` for each entry, suggesting the result is a context where all bindings have quantity 0, which may be problematic.

---

### Phase 2: Natural Deduction - Property Derivation

#### 2.1 Analysis of Typing Rules

The typing judgment is `Γ ⊢ e : τ` where context splitting is the fundamental structural rule.

##### Var Rules (Lines 243-259)

Three variant rules are provided:
- **Var**: `x :_1 τ ⊢ x : τ` — variable with multiplicity 1
- **Var-Omega**: `x :_ω τ ⊢ x : τ` — shared variable
- **Var-Borrow**: `x :_& τ ⊢ x : τ` — borrowed variable

**Problem**: The Var rule requires *exactly* multiplicity 1, but the context splitting property (that using a variable consumes its entry) means the rule should be read as: "from context containing x:_1 τ, derive x has type τ, consuming the binding."

This is semantically correct but creates an asymmetry: the Var rule *consumes* the binding, whereas in standard Type Theory, variables are looked up without consuming resources. The linearity discipline requires this, but it's a departure from standard practice.

##### Abs Rule (Lines 262-266)

```
Γ, x :_q τ₁ ⊢ e : τ₂
────────────────────────────────
Γ ⊢ λ(x :_q τ₁). e : τ₁ →^q τ₂
```

**Analysis**: This is formally correct. The function type encodes both the argument type and *how many times* the argument is used (`q`). This is the **resource annotation** on the arrow, matching Quantitative Type Theory (QTT).

##### App Rule (Lines 269-276)

```
Γ₁ ⊢ e₁ : τ₁ →^q τ₂     Γ₂ ⊢ e₂ : τ₁
───────────────────────────────────────────
Γ₁ + q·Γ₂ ⊢ e₁ e₂ : τ₂
```

**Analysis**: This is the critical rule. The context scaling `q·Γ₂` means: "if the function uses its argument q times, produce the argument with q times the resources."

*Issue*: The rule assumes the same argument type `τ₁` for both function and argument. This is standard, but there's no explicit **context splitting** rule in the natural deduction system. The splitting is embedded in the App and Let rules, which is unconventional but workable.

##### Let Rule (Lines 278-282)

```
Γ₁ ⊢ e₁ : τ₁     Γ₂, x :_q τ₁ ⊢ e₂ : τ₂
───────────────────────────────────────────────
Γ₁ + q·Γ₂ ⊢ let x :_q τ₁ = e₁ in e₂ : τ₂
```

**Problem**: This rule is *asymmetric* in how it treats `e₁` and `e₂`. If `q = 1`, then `1·Γ₂ = Γ₂`, so the context is `Γ₁ + Γ₂`. But the rule doesn't specify what happens if `x` doesn't appear in `e₂` (weakening case) or appears more than once (contraction case) — these would be type errors, but the rule doesn't enforce this; it's enforced by the resulting context not matching any valid derivation.

##### Weaken Rule (Lines 285-289)

```
Γ ⊢ e : τ     x ∉ dom(Γ)
─────────────────────────────────
Γ, x :_0 τ' ⊢ e : τ
```

**Issue**: The premise `x ∉ dom(Γ)` is redundant if the conclusion adds a new binding with multiplicity 0. The rule effectively says: "you can ignore a resource of multiplicity 0." This is consistent, but the premise is unnecessary.

##### Contract Rule (Lines 292-299)

```
Γ, x :_ω τ ⊢ e : τ'
───────────────────────────────
Γ, x :_1 τ ⊢ e[x/x, x/x] : τ'    (explicit: use x twice)
```

**Critical Problem**: This rule is formally incorrect as stated.

1. The notation `e[x/x, x/x]` is undefined in the grammar. The document introduces this syntax but it's not in the term grammar (§2.3).

2. The rule attempts to express *sharing* of a linear variable, but the direction is backwards. The standard approach in linear logic is: from `x :_1 τ`, you can *weaken* to `x :_ω τ` (upgrade), not downgrade. The current rule goes from ω to 1, which is backwards.

3. The explicit duplication syntax `e[x/x, x/x]` suggests substitution with two copies, but this isn't how interaction nets work — the δ-agent handles duplication at runtime, not at the source level.

**Correct formulation should be**: The Contract rule should express that a variable with multiplicity ω can be used multiple times. The current rule appears to be attempting to say "if you have ω, you can treat it as 1 for the purposes of this derivation," but this violates the subtyping direction (1 ⊑ ω, not ω ⊑ 1).

#### 2.2 Semiring Property Verification

The quantity semiring `({0, 1, ω}, +, ·, 0, 1)` is specified with two tables:

**Addition Table** (lines 146-150):
| `+` | 0 | 1 | ω |
|-----|---|---|---|
| **0** | 0 | 1 | ω |
| **1** | 1 | ω | ω |
| **ω** | ω | ω | ω |

**Multiplication Table** (lines 154-158):
| `·` | 0 | 1 | ω |
|-----|---|---|---|
| **0** | 0 | 0 | 0 |
| **1** | 0 | 1 | ω |
| **ω** | 0 | ω | ω |

**Verification**:

1. **Additive structure**: `(Q, +, 0)` forms a commutative monoid:
   - Associativity: Checked by table (e.g., (1+1)+ω = ω+ω = ω, 1+(1+ω) = 1+ω = ω)
   - Identity: `0 + q = q` (row/column correct)
   - Commutativity: Table is symmetric

2. **Multiplicative structure**: `(Q, ·, 1)` forms a monoid:
   - Associativity: `(1·ω)·ω = ω·ω = ω`, `1·(ω·ω) = 1·ω = ω` ✓
   - Identity: `1·q = q` and `q·1 = q` (row/column correct)
   - **Missing**: No distributivity claim is stated! A semiring requires `q₁·(q₂ + q₃) = q₁·q₂ + q₁·q₃`. Let me verify:
     - `1·(1+ω) = 1·ω = ω`
     - `1·1 + 1·ω = 1 + ω = ω` ✓
     - `ω·(1+1) = ω·1 = ω`
     - `ω·1 + ω·1 = ω + ω = ω` ✓
     
     Distributivity holds but is **not claimed in the document**.

3. **Semiring absorption**:
   - `ω` is additive absorbing: `ω + q = ω` ✓
   - `0` is multiplicative absorbing: `0·q = 0` ✓

**Critical Issue with Scalar Multiplication**: The document uses `·` for scalar multiplication on contexts (`q·Γ`), but the table uses `·` as scalar multiplication on quantities. The notation is overloaded. More seriously:

Looking at the App rule: `Γ₁ + q·Γ₂`. Here `q·Γ₂` means "scale the entire context by q". This is different from scalar multiplication in the semiring table. The document conflates:
- `q₁ · q₂` — semiring multiplication (quantity × quantity)
- `q · Γ` — context scaling (quantity × context)

The semantics of context scaling are given pointwise, but there's no formal proof that context scaling distributes over context addition: `(q·Γ₁) + (q·Γ₂) = q·(Γ₁ + Γ₂)`. This property is needed for the metatheory but is assumed without proof.

#### 2.3 Borrow Mode Algebra

The `&` mode composition (lines 181-186):
| `+` | 0 | 1 | ω | & |
|-----|---|---|---|---|
| **0** | 0 | 1 | ω | ✗ |
| **1** | 1 | ω | ω | ✗ |
| **ω** | ω | ω | ω | ✗ |
| **&** | ✗ | ✗ | ✗ | & |

**Analysis**:
- `& + & = &` is correct: two borrows don't create ownership
- All other combinations with `&` are errors, which is **sound but incomplete**

**Gap**: There's no definition of what `&` means in the result type of a function. The grammar allows `τ₁ →^& τ₂`? The document doesn't clarify. Looking at the grammar (lines 83-88), only `q ∈ {0, 1, ω}` appears in `τ₁ →^q τ₂`. This suggests `&` cannot be a function quantity, which is correct — you can't "use" a borrowed value — but it's not formally stated.

---

### Phase 3: Tableaux - Verification

#### 3.1 Type System as Sequent Calculus

We can formulate the typing rules as a sequent calculus and check for completeness and soundness.

**Goal Formula**: Is the type system consistent? Are there any untypable terms that should be typable, or typable terms that should be rejected?

##### Key Properties to Verify

1. **Weakening is admissible**: If `Γ ⊢ e : τ`, then `Γ, x :_0 τ' ⊢ e : τ`.
   - The Weaken rule explicitly allows this
   - But the rule requires `x ∉ dom(Γ)`, which is unnecessary — weakening should work for any fresh variable

2. **Contraction is admissible**: If `Γ, x :_1 τ, y :_1 τ ⊢ e : τ'`, then `Γ, z :_ω τ ⊢ e[z/x, z/y] : τ'`.
   - This is the inverse of what the Contract rule does
   - The document's Contract rule goes ω → 1, but we need 1+1 → ω for contraction
   - **This is a critical gap**: There's no rule that says "if you have two uses of x, you can combine them into an ω"

3. **Substitution**: If `Γ₁ ⊢ e : τ'` and `Γ₂, x :_q τ' ⊢ e₂ : τ`, then `Γ₁ + q·Γ₂ ⊢ e[e/x] : τ`.
   - This is Lemma 1, claimed but not proven
   - The proof sketch says "by induction" but doesn't show the inductive cases

##### Testing Edge Cases via Tableaux Method

**Case 1**: `let x :_1 Int = 1 in x + 1`

This should typecheck: `x` is used once. Let's derive:
- `∅ ⊢ 1 : Int` (native literal)
- `x :_1 Int ⊢ x : Int` (Var)
- `x :_1 Int ⊢ x + 1 : Int` (App, but where is + defined? Native operations not in grammar)

**Gap**: Native operations like `+` are not in the term grammar (line 102: only `native_lit`). This is a significant gap — the language cannot express any computation on native types without function application.

**Case 2**: `λ(x :_1 Int). (λ(y :_1 Int). y) x`

- `x :_1 Int, y :_1 Int ⊢ y : Int` (Var)
- `x :_1 Int ⊢ λ(y :_1 Int). y : Int →^1 Int` (Abs)
- `x :_1 Int ⊢ (λ(y :_1 Int). y) x : Int` (App with q=1: context is `x:_1 Int + 1·(∅)` = `x:_1 Int`)
- `∅ ⊢ λ(x :_1 Int). (λ(y :_1 Int). y) x : Int →^1 Int` (Abs)

This derives correctly.

**Case 3**: `λ(x :_1 Int). x x` — self-application with linear variable

- `x :_1 Int ⊢ x : Int` (Var)
- Need another `x :_1 Int` for second argument, but context only has one
- App rule requires `Γ₂ ⊢ e₂ : τ₁` — but context after first Var is *consumed*
- There's no way to split `x:_1` into two copies
- **This is correctly rejected** — self-application of a linear function is impossible

#### 3.2 Match/View Formalization

**Match Translation** (lines 357-362):
```
⟦match e { C(x) ↦ body }⟧ =
  let n = ⟦e⟧
  let (x_port, _rest) = destruct(n)    -- fires ε on n (consumption)
  ⟦body⟧[x_port/x]
```

**View Translation** (lines 366-372):
```
⟦view e { C(x) ↦ body }⟧ =
  let n = ⟦e⟧
  let x_port = observe(n)              -- auxiliary wire, n survives
  ⟦body⟧[x_port/x]
  -- n's ε-agent fires only at end of enclosing scope
```

**Problem**: The translation uses `let` and `destruct`/`observe` which are *not in the source grammar* (lines 93-102). This is an operational description, not a formal translation function.

**Observable Difference Claim** (line 374):
> In the interaction net reduction trace, `match` produces an `(ε ⋈ C)` interaction step at the binding site. `view` produces no such step there.

**Gap**: This is an informal description. To formally prove the difference, one would need to:
1. Define the translation function `⟦e⟧` as a mathematical function on syntax trees
2. Define the reduction relation on nets
3. Prove: `⟦match e { C(x) ↦ body }⟧` reduces to a net containing `(ε ⋈ C)` *before* any reduction of `body`, whereas `⟦view e { C(x) ↦ body }⟧` does not.

The document provides neither the formal translation function nor the reduction relation on nets.

---

### Phase 4: Hoare - Contract Establishment

#### 4.1 Pre/Post Conditions for Type System

We can formulate typing rules as Hoare triples: `{P} e {Q}` where P is the precondition (context) and Q is the postcondition (type).

##### Var Rule as Hoare Triple
```
{x :_1 τ ∈ Γ}
x
{τ}
```
The variable rule consumes its context entry — the precondition must contain the binding.

##### Abs Rule
```
{Γ, x :_q τ₁ ⊢ e : τ₂}
λ(x :_q τ₁). e
{τ₁ →^q τ₂}
```

##### App Rule
```
{Γ₁ ⊢ e₁ : τ₁ →^q τ₂} ∧ {Γ₂ ⊢ e₂ : τ₁}
e₁ e₂
{τ₂}  with remaining context Γ₁ + q·Γ₂
```

**Contract Verification**: The App rule's postcondition guarantees that the result type is `τ₂` regardless of `q`. This is important: whether the function uses its argument 0, 1, or ω times, the result type is the same.

#### 4.2 Error Cases as Failed Preconditions

Using Hoare logic, the error conditions in §10 can be expressed as **precondition failures**:

| Error | Failed Precondition |
|-------|---------------------|
| `LinearityViolation` | Variable `x:_1` used ≠ 1 times → context splitting fails |
| `BorrowContextMix` | `& + q` for `q ∈ {0,1,ω}` → context addition undefined |
| `OwnershipEscape` | `&` binding referenced outside scope → borrow scope check fails |
| `TraitNotFound` | `(C, τ) ∉ Σ` at method call → registry lookup fails |
| `NonExhaustivePattern` | Pattern set doesn't cover all constructors → exhaustiveness check fails |
| `StrictPositivityViolation` | Inductive type has negative occurrence → positivity check fails |

#### 4.3 Interaction Net Translation Contracts

The translation `⟦e⟧ : Term → Net` should satisfy:

**Precondition**: `Γ ⊢ e : τ` (source is well-typed)  
**Postcondition**: `⟦e⟧` is a well-formed net with a distinguished output port of type `τ`, and all ε-agents are matched to nodes according to the linearity invariant.

The document claims type preservation (line 57: "Type-preserving translation from source terms to interaction nets") but provides no formal statement or proof.

---

## Synthesis

### Type System Assessment (CSP)

The type system uses a **two-axis multiplicity** approach combining:
- Quantity axis: `{0, 1, ω}` — commutative semiring with absorption
- Mode axis: `{&}` — borrow mode, orthogonal to quantity

**Algebraic Structure**: The quantity semiring is well-defined and matches QTT. However:
- Distributivity is **not stated** but holds (verified above)
- The document conflates scalar multiplication on quantities (`q₁·q₂`) with context scaling (`q·Γ`)
- Missing: formal proof that context scaling distributes over context addition

**Sum Types**: The type grammar supports:
- Function types `τ →^q τ` (annotated with quantity)
- Inductive types `μα. τ` (with strict positivity)
- Trait constraints `C τ`
- Universal quantifiers `∀α. τ`
- Borrow types `&τ`

**Product Types**: Missing from the grammar. Constructor applications `Con(e₁, ..., eₙ)` are listed but there's no corresponding product/sum type former in the type grammar. This is a significant gap: algebraic data types need a formal type-level specification.

---

### Invariants Identified (Natural Deduction)

1. **Linearity Invariant (Theorem 3)**: If `x :_1 τ ∈ Γ`, then `x` appears exactly once as a free variable in `e`.
   - **Status**: Claimed but proof sketch is incomplete
   - **Issue**: The proof sketch says "by induction on typing derivation" but doesn't show the inductive cases for each rule

2. **Resource Conservation**: For any well-typed application `Γ₁ ⊢ e₁ : τ₁ →^q τ₂` and `Γ₂ ⊢ e₂ : τ₁`, the total resource context is `Γ₁ + q·Γ₂`. This is enforced by the App rule but not proven to be invariant under reduction.

3. **Borrow Scope Invariant**: A borrow `&` is valid only within its lexical scope. This is checked by the borrow checker but the formal rules are not given.

---

### State Machine Formalization (Tableaux)

The interaction net semantics defines a state machine on graphs:

**States**: Directed acyclic graphs with ports `{principal, aux₁, aux₂, ...}`  
**Agents**: `λ` (2-port), `@` (2-port), `δ` (3-port), `ε` (1-port)  
**Transitions**: Interaction rules (lines 415-421)

**Validity Constraints**:
1. **No dangling wires**: Every port is either connected or is the principal output
2. **No critical pairs**: At most one interaction rule applies to any pair of agents (claimed, citing Lafont 1990)
3. **Termination**: Every ε-agent eventually interacts (memory safety)

**Gap**: The document doesn't formalize the state machine. The tables give the rules but not the transition relation `N → N'`.

---

### Error Type Analysis (Hoare)

All errors in §10 are **static** (compile-time), which aligns with the design goal. Using Hoare logic:

- **Total correctness**: Since all errors are caught at compile time, runtime "postconditions" are always satisfied for well-typed programs
- **Partial correctness**: The interaction net may not terminate, but when it does, it produces correct values (confluence is claimed but not proven for this specific system)

**Missing**: There's no specification of *what happens* if a well-typed term reduces to a net with no normal form (non-termination). The document assumes termination but doesn't prove it.

---

### Correctness Arguments

| Property | Status | Evidence |
|----------|--------|----------|
| Type Preservation | **Claimed, not proven** | Theorem 1 proof sketch invokes Lemma 1 |
| Progress | **Claimed, not proven** | Canonical forms argument incomplete |
| Substitution | **Claimed, not proven** | Proof sketch "by induction" |
| Linearity Invariant | **Claimed, not proven** | Proof sketch "by induction" |
| Confluence | **Claimed, not proven** | Cites Lafont 1990 but doesn't verify for this specific system |
| Memory Safety | **Claimed, not proven** | "Follows from linearity invariant" |
| S5′ (Parallel Isolation) | **Defined, not proven** | δ-root definition given but no proof of sufficiency |

---

### Formal Gaps Summary

| Gap | Severity | Location |
|-----|----------|----------|
| Contract rule is backwards (ω → 1 instead of 1+1 → ω) | **Critical** | §4.3, line 292 |
| No formal translation function for interaction nets | **Critical** | §6.2 |
| No type preservation proof | **Critical** | §6.4 |
| Native operations (+, -, etc.) not in grammar | **High** | §2.3 |
| No product/sum type formers in type grammar | **High** | §2.2 |
| `&` mode algebra incomplete (no subtyping) | **Medium** | §3.3 |
| Context scaling distributivity not proven | **Medium** | §4.2 |
| Exhaustiveness algorithm not specified | **Medium** | §5.3 |
| δ-root verification algorithm not specified | **Medium** | §7.2 |
| Positivity checking algorithm not specified | **Medium** | §2.4 |
| Missing distributivity claim in semiring | **Low** | §3.2 |

---

### Concerns

1. **Unproven Metatheory**: The document acknowledges (line 302) that metatheory proofs are "implementation prerequisites for Phase 0." This means the design is not yet formally verified — implementation should not begin until these proofs are complete.

2. **Contract Rule Error**: The Contract rule (line 292-299) is formally incorrect and would lead to an unsound type system if implemented as stated.

3. **Observable Match/View**: The "observability" claim (line 374) is intuitive but not formally proven. A formal proof would require defining the translation function and reduction relation precisely.

4. **S5′ Formalization**: The δ-root definition (line 443) is informal ("connected to a node outside G") without a precise graph-theoretic definition. The verification algorithm (line 453) is described but not specified.

5. **Incomplete Grammar**: Native operations are referenced (e.g., line 546-549 describe Int as having + operations) but are not in the term grammar. The language cannot actually express computations on native types.

---

### Validation Checklist

- [x] Reasoning Flow Report present
- [x] Phase 1 (CSP) complete
- [x] Phase 2 (Natural Deduction) complete
- [x] Phase 3 (Tableaux) complete
- [x] Phase 4 (Hoare) complete
- [x] Methodology declared in header
- [x] File saved to reasoning-flows/
- [x] Output matches schema

---

**Recommendation**: Do not proceed to Phase 0 implementation until:
1. The Contract rule is corrected (1+1 → ω, not ω → 1)
2. A formal translation function `⟦e⟧` is defined
3. Metatheory proofs are completed (at least for λ◦₀ as specified in the roadmap)
4. Native operations are added to the grammar
5. The type grammar is extended with product/sum type formers
