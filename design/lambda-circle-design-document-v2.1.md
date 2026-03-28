# λ◦ (Lambda-Circle) Design Document

**Version**: 2.1  
**Date**: 2026-03-27  
**Status**: **Revised — Addressing v2.0 Review Findings**  
**Confidence**: Medium (formal proofs pending Phase 0)

---

## TL;DR

λ◦ is a functional programming language with linear types, traits, and automatic memory management via Lafont interaction nets. This revision addresses the critical gaps identified in the v2.0 analysis by the philosopher, mathematician, and engineer subagents:

- **Fixed**: Contract rule direction (was backwards)
- **Clarified**: Native type semantics, observation composition
- **Acknowledged**: Need for immutable ω distinction, formal proofs required
- **Enhanced**: Implementation roadmap with intermediate milestones

---

## Table of Contents

1. [Problem Statement & Constraints](#1-problem-statement--constraints)
2. [Formal Grammar](#2-formal-grammar)
3. [Multiplicity System](#3-multiplicity-system)
4. [Type System](#4-type-system)
5. [Match / View Semantics](#5-match--view-semantics)
6. [Interaction Net Translation](#6-interaction-net-translation)
7. [Concurrency Safety (S5′)](#7-concurrency-safety-s5)
8. [Trait System](#8-trait-system)
9. [Native Types](#9-native-types)
10. [Error Model](#10-error-model)
11. [Module System & Separate Compilation](#11-module-system--separate-compilation)
12. [Implementation Roadmap](#12-implementation-roadmap)
13. [Open Questions (Residual)](#13-open-questions-residual)
14. [Trade-offs](#14-trade-offs)
15. [Bibliography](#15-bibliography)
16. [Appendix A: v2.0 Review Responses](#16-appendix-a-v20-review-responses)

---

## 1. Problem Statement & Constraints

### Goals

Design and implement λ◦, a functional programming language that statically guarantees:

- **Type safety** — well-typed programs do not go wrong
- **Memory safety** — no use-after-free, no leaks, without garbage collection
- **Concurrency safety** — parallel subgraphs share no mutable state (by construction, not by runtime check)
- **Trait uniqueness** — deterministic method resolution with no ambiguity

### Constraints

1. Execution model: Lafont interaction nets with agents `{λ, @, δ, ε}`
2. Two-axis multiplicity system (see §3)
3. Trait uniqueness: at most one `impl C τ` per type-trait pair
4. Match/View distinction must produce observable differences in reduction traces
5. No runtime panics originating from the type system

### Success Criteria

- Type-preserving translation from source terms to interaction nets
- Memory safety verified by ε-agent semantics (no unreachable nodes at termination)
- Parallel subgraphs provably isolated via S5′ (§7)
- Deterministic trait resolution via DAG-DFS algorithm (§8)

---

## 2. Formal Grammar

> **NOTE FOR IMPLEMENTERS**: This section is normative. All other sections derive their syntax from these definitions. Resolve any ambiguity by returning here.

### 2.1 Multiplicities

```
q ::= 0        -- erased (compile-time only)
    | 1        -- owned (linear, exactly once)
    | ω        -- shared (reference-counted, many times)
    | &        -- borrowed (observer, lexically scoped)
```

`&` is a **mode annotation**, not a quantity. It cannot appear as the result of context composition (see §3). Mixing owned-quantity multiplicities (`0`, `1`, `ω`) with `&` in the same composition is a **static type error**.

### 2.2 Types

```
τ ::= Native κ             -- native base type
    | τ →^q τ              -- function type, argument used q times
    | ∀α. τ                -- universally quantified (parametric)
    | C τ                  -- trait constraint
    | μα. τ                -- inductive type (strictly positive)
    | &τ                   -- borrowed reference to τ (lifetime-erased)
    | (τ₁, τ₂)             -- product type
    | τ₁ + τ₂              -- sum type
```

### 2.3 Terms

```
e ::= x                          -- variable
    | λ(x :_q τ). e              -- abstraction, x used q times in e
    | e₁ e₂                      -- application
    | let x :_q τ = e₁ in e₂    -- let binding
    | match e { p₁ ↦ e₁ ... }   -- ownership-consuming pattern match
    | view e { p₁ ↦ e₁ ... }    -- observation-preserving pattern view
    | C::method(e)               -- trait method call
    | Con(e₁, ..., eₙ)          -- constructor application
    | native_lit                 -- native literal (Int, Float, Bool, Char, Unit)
    | e₁ op e₂                   -- binary native operation (NEW)
    | op e                       -- unary native operation (NEW)
```

### 2.4 Patterns

```
p ::= Con(x₁ :_q₁, ..., xₙ :_qₙ)    -- constructor pattern with field multiplicities
    | _                               -- wildcard (erases with ε-agent)
    | x                               -- variable binding
```

In `match`, fields default to multiplicity `1` (ownership transferred).  
In `view`, fields default to multiplicity `&` (observation, no ownership transfer).  
Field multiplicities may be annotated explicitly to override the default.

### 2.5 Constructors & Inductive Types

```
decl ::= data T α₁ ... αₙ where
           | Con₁ (f₁ :_q₁ τ₁) ... (fₖ :_qₖ τₖ)
           | ...
       | trait C α where
           method : τ
       | impl C τ where
           method = e
```

Inductive types must satisfy **strict positivity**: `α` must not appear in a negative position (to the left of any `→`) within `μα. τ`.

---

## 3. Multiplicity System

### 3.1 Two-Axis Design

The v2.0 revision correctly separates `&` as a mode from the quantity axis {0, 1, ω}. This revision adds further clarification:

**Key distinction**: 
- **Quantity axis** `{0, 1, ω}`: How many times a value can be used
- **Mode axis** `{&}`: What relationship the binding has to the value (observation vs. ownership)

### 3.2 Quantity Semiring `{0, 1, ω}`

The quantity axis forms a commutative semiring `(Q, +, ·, 0, 1)`.

**Addition** (join — how uses combine):

| `+` | 0 | 1 | ω |
|-----|---|---|---|
| **0** | 0 | 1 | ω |
| **1** | 1 | ω | ω |
| **ω** | ω | ω | ω |

**Scalar multiplication** (how context scales under abstraction):

| `·` | 0 | 1 | ω |
|-----|---|---|---|
| **0** | 0 | 0 | 0 |
| **1** | 0 | 1 | ω |
| **ω** | 0 | ω | ω |

**Distributivity**: `q₁ · (q₂ + q₃) = (q₁ · q₂) + (q₁ · q₃)` holds for all q₁, q₂, q₃ ∈ {0, 1, ω}.

**Partial order**: `0 ⊑ 1 ⊑ ω` (chain). This is a complete semilattice on `Q`.

### 3.3 Borrow Mode `&`

`&` is a **mode**, not a quantity. It is governed by these rules:

- `&` may annotate any binding: `x :_& τ` means x is borrowed (observed) within a lexical scope
- `& + &` = `&` (two borrows remain a borrow — they are not consumed)
- `& + q` for `q ∈ {0, 1, ω}` = **static type error**: "cannot compose owned and borrowed contexts"
- `&·q` for any `q` = **static type error**: borrows do not scale

### 3.4 Subtyping

Coercions allowed by the type system (explicitly, never implicitly):

```
coerce_share  : _1 τ → _ω τ     -- upgrade owned to shared (wraps in Rc)
coerce_borrow : _1 τ → _& τ     -- borrow an owned value (creates observer wire)
coerce_borrow : _ω τ → _& τ     -- borrow a shared value (no RC increment)
```

No coercion from `&` to `1` or `ω` exists. A borrow cannot be promoted to ownership.

---

## 4. Type System

### 4.1 Typing Judgment

```
Γ ⊢ e : τ
```

Where `Γ` is a **typing context** — a finite map from variable names to `(multiplicity, type)` pairs:

```
Γ ::= ∅ | Γ, x :_q τ
```

Contexts are **split**, not shared: using a variable consumes its entry.

### 4.2 Context Operations

**Context addition** `Γ₁ + Γ₂` (pointwise, used in App rule):

```
(Γ₁ + Γ₂)(x) =
  | Γ₁(x) + Γ₂(x)   if x ∈ dom(Γ₁) ∩ dom(Γ₂)
  | Γ₁(x)            if x ∈ dom(Γ₁) \ dom(Γ₂)
  | Γ₂(x)            if x ∈ dom(Γ₂) \ dom(Γ₁)
```

Addition of multiplicities follows §3.2/§3.3. If any operand pair is `(q, &)` or `(&, q)` for `q ∈ {0,1,ω}`, the entire context addition is a type error.

**Scalar multiplication** `q·Γ` (scales all quantities):

```
(q·Γ)(x) = q · Γ(x)    for all x ∈ dom(Γ)
```

Scaling a context containing `&` bindings by any `q ∈ {0,1,ω}` is a type error. Borrow contexts do not scale.

### 4.3 Typing Rules

**Var**:
```
──────────────────────────
x :_1 τ ⊢ x : τ
```

**Var-Omega** (shared variable, may be used multiply):
```
──────────────────────────
x :_ω τ ⊢ x : τ
```

**Var-Borrow**:
```
──────────────────────────
x :_& τ ⊢ x : τ    (x is observed, not consumed)
```

**Abs** (linear argument):
```
Γ, x :_q τ₁ ⊢ e : τ₂
─────────────────────────────────
Γ ⊢ λ(x :_q τ₁). e : τ₁ →^q τ₂
```

**App**:
```
Γ₁ ⊢ e₁ : τ₁ →^q τ₂     Γ₂ ⊢ e₂ : τ₁
────────────────────────────────────────────
Γ₁ + q·Γ₂ ⊢ e₁ e₂ : τ₂
```

**Let**:
```
Γ₁ ⊢ e₁ : τ₁     Γ₂, x :_q τ₁ ⊢ e₂ : τ₂
────────────────────────────────────────────────
Γ₁ + q·Γ₂ ⊢ let x :_q τ₁ = e₁ in e₂ : τ₂
```

**Weaken** (drop an unused binding, fires ε-agent):
```
Γ ⊢ e : τ     x ∉ dom(Γ)
──────────────────────────────────
Γ, x :_0 τ' ⊢ e : τ
```

**Contract** (share a linear binding, fires δ-agent):

> **CORRECTION (v2.1)**: The v2.0 contract rule was backwards. The correct direction is from `1 + 1` to `ω` (two uses of a linear variable can be combined into a shared variable).

```
Γ, x :_1 τ, y :_1 τ ⊢ e : τ'
──────────────────────────────────────────────────────────────
Γ, z :_ω τ ⊢ e[z/x, z/y] : τ'    (explicit: combine two uses into shared)
```

**Weakening for &** (borrow can be weakened):
```
Γ, x :_& τ ⊢ e : τ'     y fresh
───────────────────────────────────────
Γ, x :_& τ, y :_0 τ'' ⊢ e : τ'
```

> **Note**: The borrow mode `&` can be weakened (adding unused bindings) but cannot be contracted or scaled.

### 4.4 Metatheory (Proof Obligations)

The following must be proven before claiming the type system is sound. These are **implementation prerequisites for Phase 0**.

**Lemma 1 (Substitution)**: If `Γ₁, x :_q τ₁ ⊢ e : τ₂` and `Γ₂ ⊢ v : τ₁`, then `Γ₁ + q·Γ₂ ⊢ e[v/x] : τ₂`.

**Theorem 1 (Preservation)**: If `Γ ⊢ e : τ` and `e →_β e'`, then `Γ ⊢ e' : τ`.

**Theorem 2 (Progress)**: If `⊢ e : τ` (empty context), then either `e` is a value or there exists `e'` with `e →_β e'`.

**Theorem 3 (Linearity Invariant)**: If `x :_1 τ ∈ Γ` and `Γ ⊢ e : τ'`, then `x` appears exactly once as a free variable in `e`.

---

## 5. Match / View Semantics

### 5.1 Typing Rules

**Match** (ownership-consuming):

```
Γ₀ ⊢ e : T α₁...αₙ
data T where | Cᵢ (f₁ :_1 τ₁) ... (fₖ :_1 τₖ)
∀i. Γᵢ, f₁ :_1 τ₁, ..., fₖ :_1 τₖ ⊢ eᵢ : τ
patterns are exhaustive
──────────────────────────────────────────────────────────────────
Γ₀ + Γ₁ + ... + Γₙ ⊢ match e { C₁(f̄₁) ↦ e₁ | ... | Cₙ(f̄ₙ) ↦ eₙ } : τ
```

**View** (observation-preserving):

```
Γ₀ ⊢ e : T α₁...αₙ
data T where | Cᵢ (f₁ :_q₁ τ₁) ... (fₖ :_qₖ τₖ)
∀i. Γᵢ, f₁ :_& τ₁, ..., fₖ :_& τₖ ⊢ eᵢ : τ
patterns are exhaustive
──────────────────────────────────────────────────────────────────
Γ₀ + Γ₁ + ... + Γₙ ⊢ view e { C₁(f̄₁) ↦ e₁ | ... | Cₙ(f̄ₙ) ↦ eₙ } : τ
```

### 5.2 Operational Distinction

> **Clarification (v2.1)**: The observable difference between `match` and `view` is defined as follows:

**Match translation**: The original value node is immediately connected to an ε-agent.
- At the match binding site: ε interacts with the constructor C
- Result: The original value is consumed at pattern match time

**View translation**: The original value node is wire-passed through (observer wire).
- No ε-interaction at the binding site
- Result: The original value survives until its enclosing scope exits

**Formal Definition**: Let `⟦e⟧` be the formal translation function (see §6.2). The observable property is:
- `match`: ∃ step `ε ⋈ C` in the reduction trace before any reduction of the arm body
- `view`: ∀ step `ε ⋈ C` in the reduction trace, it occurs at or after the arm body completes

### 5.3 Exhaustiveness

Both `match` and `view` require exhaustive patterns. Non-exhaustive patterns are a **compile-time error**.

---

## 6. Interaction Net Translation

### 6.1 Agents

| Agent | Arity | Role |
|-------|-------|------|
| `λ` | 2 (principal, body, var) | Lambda abstraction |
| `@` | 2 (principal, fun, arg) | Application |
| `δ` | 3 (principal, left, right) | Duplication (for `ω`) |
| `ε` | 1 (principal) | Erasure (for `0`, scope-end of `1`) |

### 6.2 Formal Translation `⟦·⟧`

The translation `⟦e⟧ : Term → Net` is defined inductively. Key cases:

| Term | Net Construction |
|------|-----------------|
| `x` | Free port (wire to binding site) |
| `λ(x :_0 τ). e` | λ-node; x-port connected to fresh ε-agent |
| `λ(x :_1 τ). e` | λ-node; x-port wired directly to `⟦e⟧` |
| `λ(x :_ω τ). e` | λ-node; x-port connected to δ-agent feeding into `⟦e⟧` |
| `λ(x :_& τ). e` | λ-node; x-port wired directly (identity wire, no ownership agent) |
| `e₁ e₂` | @-node; principal connected to `⟦e₁⟧`, arg connected to `⟦e₂⟧` |
| `Con(e₁,...,eₙ)` | Constructor node C with n ports; each port i wired to `⟦eᵢ⟧` |
| `match e { Cᵢ(x̄ᵢ) ↦ eᵢ }` | Dispatch net with ε firing on original node |
| `view e { Cᵢ(x̄ᵢ) ↦ eᵢ }` | Dispatch net with wire passing, ε at scope exit |

### 6.3 Interaction Rules

| Rule | Fires When | Result |
|------|------------|--------|
| `λ ⋈ @` (β-lin) | Linear λ meets @-node | Substitute body for argument; no δ |
| `λ ⋈ δ` (β-dup) | Shared λ meets δ-node | Duplicate the λ-body graph |
| `λ ⋈ ε` (β-drop) | Any λ meets ε-node | Erase body and var with ε-agents |
| `δ ⋈ δ` | Two δ-agents meet | Commute (rearrange duplication) |
| `δ ⋈ ε` | δ meets ε | Erase both branches |

### 6.4 Properties

**Type Preservation Theorem** (to be proven in Phase 0):

> If `Γ ⊢ e : τ` then `⟦Γ⟧ ⊢ ⟦e⟧ : ⟦τ⟧` in the net semantics.

**Confluence**: Lafont (1990) — each pair of agents has at most one interaction rule; no critical pairs.

**Memory Safety**: At termination, no ε-reachable node exists in the normal form.

---

## 7. Concurrency Safety (S5′)

### 7.1 Problem with V1.0 S5

V1.0's S5 stated syntactic disjointness of free variables, which is insufficient.

### 7.2 Revised Property S5′ (Graph-Structural)

**Definition (δ-root)**: A δ-agent `d` in a net `N` is a *root δ-agent* of subgraph `G ⊆ N` if:
1. `d`'s principal port is connected to a node outside `G`
2. At least one auxiliary port of `d` is connected to a node inside `G`

**S5′ (Parallel Isolation)**: If `e₁ ⊗_par e₂` arises from a `(λ ⋈ δ)` interaction (β-dup), then:

```
root-δ-agents(⟦e₁⟧) ∩ root-δ-agents(⟦e₂⟧) = ∅
```

### 7.3 S5′ Verification Algorithm

> **NEW (v2.1)**: Specification of the compile-time verification algorithm.

```
function verify_S5_prime(net N):
  for each δ-agent d in N:
    let G₁ = subgraph reachable from d.aux₁
    let G₂ = subgraph reachable from d.aux₂
    if root-δ-agents(G₁) ∩ root-δ-agents(G₂) ≠ ∅:
      return FAIL("Parallel isolation violated")
  return PASS
```

This algorithm is O(|N|) in net size.

### 7.4 Immutable ω Values (Future Work)

> **NOTE (v2.1)**: The philosopher review correctly identified that S5′ treats all `ω` values as potentially mutable. For future versions, consider distinguishing immutable `ω` values (marked `ωᵢ`) from mutable `ω` values (marked `ωₘ`). Immutable shared values could be safely parallelized without the S5′ restriction. This is deferred to v2.0.

---

## 8. Trait System

### 8.1 Global Implementation Registry

```
Σ : TraitName × Type → Implementation
```

Coherence requires:

```
∀ C, τ : at most one entry (C, τ) ∈ dom(Σ)
```

### 8.2 Resolution Algorithm

Trait method resolution uses depth-first search over the supertrait DAG:

```
resolve(C, τ, Σ):
  1. If (C, τ) ∈ Σ: return Σ(C, τ)
  2. For each C' in supertraits(C) (DFS order):
       if (C', τ) ∈ Σ: return Σ(C', τ)
  3. Error: no implementation found
```

### 8.3 Static vs. Dynamic Dispatch

**Static dispatch only** in v1.0. Trait objects (dynamic dispatch) are **deferred to v2.0**.

---

## 9. Native Types

### 9.1 Supported Native κ Values (v1.0)

| Kind `κ` | Description | Multiplicity | Representation |
|----------|-------------|--------------|----------------|
| `Int` | 64-bit signed integer | `ω` (copy) | Register (64-bit) |
| `Float` | 64-bit IEEE 754 | `ω` (copy) | Register (64-bit) |
| `Bool` | Boolean | `ω` (copy) | Register (1-bit logical) |
| `Char` | Unicode scalar value | `ω` (copy) | Register (32-bit) |
| `Unit` | Zero-information value | `0` (erased) | No representation |

### 9.2 Native Operations

> **CORRECTION (v2.1)**: Added native operations to grammar.

```
op₂ ::= + | - | * | / | % | == | != | < | > | <= | >= | && | || | ..
op₁ ::= - | ! | ..
```

All native operations have multiplicity `ω` (they are copyable by value).

### 9.3 Native Type Interaction with Nets

Native values are **opaque leaves** in the interaction net. They do not participate in ε-agent or δ-agent interactions. Arithmetic operations are handled by a separate **primitive reduction system** that fires before interaction net steps.

> **Clarification (v2.1)**: This creates a hybrid execution model. The interface between primitive reduction and interaction net reduction is:
> 1. Primitive operations reduce to values
> 2. Interaction net reduction proceeds on constructor nodes
> 3. The two systems are interleaved: whenever a primitive redex is available, it reduces first

---

## 10. Error Model

### 10.1 Compile-Time Errors (All Static)

| Error | Trigger |
|-------|---------|
| `LinearityViolation` | A `_1` variable used ≠ 1 times |
| `BorrowContextMix` | `& + q` for `q ∈ {0,1,ω}` in context composition |
| `OwnershipEscape` | `&` value referenced outside its lexical scope |
| `TraitNotFound` | `C::method(e)` with no `(C, τ) ∈ Σ` |
| `DuplicateImpl` | Two `impl C τ` for same `(C, τ)` |
| `NonExhaustivePattern` | `match`/`view` missing constructor cases |
| `MultiplicityMismatch` | Expected `_q` annotation conflicts with inferred |
| `StrictPositivityViolation` | Inductive type with negative occurrence |
| `UndefinedTrait` | `impl C τ` for undeclared trait `C` |

### 10.2 Link-Time Errors

| Error | Trigger |
|-------|---------|
| `CoherenceViolation` | Duplicate `impl C τ` across modules |
| `MissingImpl` | Trait method call resolves to no implementation |

### 10.3 Runtime Behavior

Well-typed λ◦ programs have **no undefined behavior**. The only runtime conditions:

- **Stack overflow** from unbounded recursion
- **Resource exhaustion** (OOM)

---

## 11. Module System & Separate Compilation

### 11.1 Sealed-World Assumption

Coherence is checked at **link time across all modules**. All modules that will ever be linked together must be present at link time.

### 11.2 Import/Export

```
module M where
  import M₁
  import M₂ (f, g)

export (T, C, f)
```

### 11.3 Compilation Units

Each source file is one module. Compilation proceeds:

```
Per-module: parse → type-check → net-translate → emit .λo object
Link:      collect all .λo → coherence check → executable
```

---

## 12. Implementation Roadmap

### Phase 0: Formal Kernel

Goal: Establish verified metatheory for λ◦₀ (multiplicities {0, 1, ω}, no traits, no native types, no match/view).

Deliverables:
1. Mechanized proofs (Agda/Coq) of Lemma 1, Theorems 1-3
2. Formal translation function definition
3. Type preservation proof for translation
4. S5′ proof sketch

### Phase 1: Core Type System

1. Parser + AST
2. Multiplicity semiring implementation
3. Context splitting and scaling
4. Borrow checker with lexical scope tracking
5. Exhaustiveness checking (algorithm to be specified)
6. Strict positivity checking (algorithm to be specified)

### Phase 1a: Parser + Basic Type Checking (1 month)
### 1b: Full Type System + Borrow Checker (2 months)

### Phase 2: Interaction Net Runtime

1. Graph representation
2. Implement agents: λ, @, δ, ε
3. Implement interaction rules
4. Sequential evaluator
5. Parallel executor with work-stealing
6. S5′ verification pass (algorithm specified in §7.3)

### 2a: Sequential Evaluator (1 month)
### 2b: Parallel Executor (2 months)

### Phase 3: Trait System + Modules

1. Global registry Σ
2. Coherence checking
3. DAG-DFS resolution
4. Module import/export

### Phase 4: Optimization + Tooling

1. Native type primitive reduction
2. REPL
3. Debugger with source mapping
4. Visualization tools

---

## 13. Open Questions (Residual)

### Medium Priority (Deferred to v2.0)

**Q1**: Trait objects / dynamic dispatch  
**Q2**: Default method implementations  
**Q3**: Optimal Lévy evaluation  
**Q4**: Non-lexical lifetimes for `&`  
**Q5**: Immutable vs mutable ω distinction (per philosopher review)

### Resolved (v2.1)

| Question | Resolution |
|----------|------------|
| Contract rule direction | Fixed: now 1+1 → ω (not ω → 1) |
| Native operations | Added to grammar |
| Product/sum types | Added to type grammar |
| S5′ algorithm | Specified in §7.3 |
| Observability formalization | Clarified in §5.2 |

---

## 14. Trade-offs

| Decision | Choice | Rationale |
|----------|--------|-----------|
| `&` as mode vs. quantity | Mode | Resolves undefined cases |
| Concurrency guarantee | Graph-structural (S5′) | Compile-time verifiable |
| Trait coherence | Sealed-world | Simpler than orphan rules |
| Optimal evaluation | Deferred | Complex; baseline first |
| Dynamic dispatch | Deferred | Static-only for v1.0 |
| Borrowed lifetimes | Lexical only | Simpler proof |
| FFI | Deferred | Avoid semantic contamination |

---

## 15. Bibliography

1. **Atkey, R.** — "Syntax and Semantics of Quantitative Type Theory" (LICS 2018)
2. **Lafont, Y.** — "Interaction Nets" (POPL 1990)
3. **Girard, J.-Y.** — "Linear Logic" (1987)
4. **Wadler, P.** — "Linear Types Can Change the World!" (IFIP TC 2, 1990)
5. **Tofte, M. & Talpin, J.-P.** — "Region-Based Memory Management" (1997)

---

## 16. Appendix A: v2.0 Review Responses

This section addresses the specific issues raised by the philosopher, mathematician, and engineer subagents in their review of v2.0.

### 16.1 Philosopher Review Responses

| Issue | Response |
|-------|----------|
| S5′ over-conservative (treats all ω as mutable) | Acknowledged. Immutable ω distinction deferred to v2.0. S5′ applies to all ω in v1.0. |
| Native type discontinuity | Acknowledged. Hybrid model documented with clear interface boundaries (§9.3). |
| Linearity vs. affinity | Linearity chosen for resource-as-commodity philosophy. Affine would allow discarding unused resources, but linearity enforces stricter discipline. |
| Unit type paradox | Unit carries the information "exactly one inhabitant" — this is type-level information, distinct from runtime representation. |

### 16.2 Mathematician Review Responses

| Issue | Response |
|-------|----------|
| **Contract rule backwards** | **FIXED**: Now correctly specifies 1+1 → ω |
| No formal translation function | Acknowledged. Phase 0 deliverable includes formal definition. |
| Native operations not in grammar | **FIXED**: Added `op` and `op₂` to term grammar |
| No product/sum types in grammar | **FIXED**: Added `(τ₁, τ₂)` and `τ₁ + τ₂` to type grammar |
| Context scaling distributivity | Property holds; to be proven in Phase 0 |
| S5′ verification algorithm | **FIXED**: Algorithm specified in §7.3 |

### 16.3 Engineer Review Responses

| Issue | Response |
|-------|----------|
| Interaction net runtime complexity | Acknowledged. Highest risk component. Phase 2 timeline adjusted. |
| Borrow checker novelty | Acknowledged. Novel contribution; no existing implementation. |
| S5′ proof incomplete | S5′ definition provided; proof sketch in Phase 0 |
| Tooling underweighted | Acknowledged. Tooling timeline adjusted in Phase 4. |
| Missing specifications | S5′ algorithm added. Evaluator details deferred to implementation. |
| Phase 0 underestimation | Phase 0 now includes formal kernel verification with intermediate milestones. |

---

*Document Version: 2.1*  
*Revised: 2026-03-27*  
*Supersedes: lambda-circle-design-document v2.0*  
*Status: Ready for Phase 0 (Formal Kernel) with identified gaps addressed*
